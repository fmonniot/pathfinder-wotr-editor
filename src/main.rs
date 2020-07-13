use iced::{
    button, Align, Application, Button, Column, Command, Container, Element, Length, Settings,
    Subscription, Text,
};
use std::path::PathBuf;

mod character_view;
mod data;
mod dialog;
mod editor_widget;
mod loader;

use editor_widget::EditorWidget;
use loader::{Loader, LoaderError, LoadingStep};

pub fn main() {
    Main::run(Settings::default())
}

struct LoadingState {
    loader: Loader,
    current_step: LoadingStep,
    failed: Option<LoaderError>,
    // TODO Add progress bar
}

enum Main {
    Loader {
        open_button_state: button::State,
        open_failed: Option<dialog::OpenError>,
    },
    Loading(LoadingState),
    Loaded(EditorWidget),
}

#[derive(Debug, Clone)]
enum MainMessage {
    OpenFileDialog,
    FileChosen(Result<PathBuf, dialog::OpenError>),
    LoadProgressed(LoadingStep),
    EditorMessage(editor_widget::Message),
}

impl Application for Main {
    type Executor = iced::executor::Default;
    type Message = MainMessage;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        // Normal running condition
        /*
        let component = Main::Loader {
            open_button_state: button::State::new(),
            open_failed: None
        };
        */

        // Hack to speed up development, should probably be behind a flag
        let party = data::read_entity_from_path("samples/party.json").unwrap();
        let party = data::IndexedJson::new(party);
        let party = data::read_party(party).unwrap();

        (Main::Loaded(EditorWidget::new(party)), Command::none())
    }

    fn title(&self) -> String {
        match self {
            Main::Loader { .. } => format!("Pathfinder WotR Editor"),
            Main::Loading(LoadingState { loader, .. }) => {
                format!("Loading file {:?}", loader.file_path())
            }
            Main::Loaded { .. } => format!("Pathfinder WotR Editor"),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            MainMessage::OpenFileDialog => {
                println!("open file dialog");
                Command::perform(dialog::open_file(), MainMessage::FileChosen)
            }
            MainMessage::FileChosen(Ok(path)) => {
                println!("Let's open {:?}", path);
                *self = Main::Loading(LoadingState {
                    loader: Loader::new(path),
                    current_step: LoadingStep::Initialized,
                    failed: None,
                });
                Command::none()
            }
            MainMessage::FileChosen(Err(error)) => {
                *self = Main::Loader {
                    open_button_state: button::State::new(),
                    open_failed: Some(error),
                };
                Command::none()
            }
            MainMessage::LoadProgressed(step) => match self {
                Main::Loading(ref mut state) => match step {
                    LoadingStep::Done { party } => {
                        *self = Main::Loaded(EditorWidget::new(party));
                        Command::none()
                    }
                    _ => {
                        state.current_step = step;
                        Command::none()
                    }
                },
                _ => Command::none(),
            },
            MainMessage::EditorMessage(msg) => {
                match self {
                    Main::Loaded(ref mut state) => {
                        state.update(msg);
                    }
                    _ => (),
                };
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<MainMessage> {
        match self {
            Main::Loader {
                open_button_state,
                open_failed,
            } => {
                let mut layout = Column::new()
                    .align_items(Align::Center)
                    .push(
                        Text::new("Pathfinder Editor - Wrath of the Righteous Edition")
                            .width(Length::Fill),
                    )
                    .push(
                        Button::new(open_button_state, Text::new("Load a save file"))
                            .on_press(MainMessage::OpenFileDialog)
                            .padding(10),
                    );

                if let Some(error) = open_failed {
                    layout = layout.push(Text::new(format!("Loading file failed: {:?}", error)));
                };

                layout.into()
            }
            Main::Loading(LoadingState {
                loader,
                failed,
                current_step,
            }) => {
                let layout = match failed {
                    Some(error) => Column::new()
                        .push(Text::new("Loading failed"))
                        .push(Text::new(format!("{:?}", error))),
                    None => Column::new()
                        .push(Text::new(format!("Loading {:?}", loader.file_path())))
                        .push(Text::new(format!(
                            "Completion: {}/100",
                            current_step.completion_percentage()
                        )))
                        .push(Text::new(format!("{}", current_step.next_description()))),
                };

                Container::new(layout).into()
            }
            Main::Loaded(editor) => editor.view().map(MainMessage::EditorMessage),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match self {
            Main::Loading(LoadingState { loader, .. }) => {
                let l = loader.clone();
                iced::Subscription::from_recipe(l).map(Self::Message::LoadProgressed)
            }
            _ => Subscription::none(),
        }
    }
}
