use iced::{
    button, Align, Application, Button, Column, Command, Container, Element, Length, Settings,
    Subscription, Text,
};
use std::path::PathBuf;

mod character_view;
mod data;
mod dialog;
mod editor_widget;
mod json;
mod labelled_input_number;
mod player_widget;
mod save;

use editor_widget::EditorWidget;
use save::{LoadNotifications, LoadingStep, SaveError, SaveLoader};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() {
    env_logger::init();
    log::debug!("Running with version {}", VERSION);

    Main::run(Settings::default())
}

struct LoadingState {
    notifications: LoadNotifications,
    file_path: PathBuf,
    current_step: LoadingStep,
    failed: Option<SaveError>,
    // TODO Add progress bar
}

enum Main {
    Loader {
        open_button_state: button::State,
        open_failed: Option<dialog::OpenError>,
    },
    Loading(Box<LoadingState>),
    Loaded(Box<EditorWidget>),
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
        let file_path: PathBuf = "samples/Manual_17____27_Gozran__IV__4710___11_43_08.zks".into();
        let (loader, notifications) = SaveLoader::new(file_path.clone());

        // Hack to speed up development, should probably be behind a flag (open via cli)
        let component = Main::Loading(Box::new(LoadingState {
            notifications,
            file_path,
            current_step: LoadingStep::Initialized,
            failed: None,
        }));

        (component, Command::perform(loader.load(), |_| todo!()))
    }

    fn title(&self) -> String {
        match self {
            Main::Loader { .. } => "Pathfinder WotR Editor".to_string(),
            Main::Loading(state) => format!("Loading file {:?}", state.file_path),
            Main::Loaded { .. } => "Pathfinder WotR Editor".to_string(),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            MainMessage::OpenFileDialog => {
                Command::perform(dialog::open_file(), MainMessage::FileChosen)
            }
            MainMessage::FileChosen(Ok(file_path)) => {
                let (loader, notifications) = SaveLoader::new(file_path.clone());

                *self = Main::Loading(Box::new(LoadingState {
                    notifications,
                    file_path,
                    current_step: LoadingStep::Initialized,
                    failed: None,
                }));
                Command::perform(loader.load(), |_| todo!())
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
                    LoadingStep::Done(done) => {
                        *self = Main::Loaded(Box::new(EditorWidget::new(
                            done.archive_path,
                            done.party,
                            done.player,
                        )));
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
                if let Main::Loaded(ref mut state) = self {
                    state.update(msg).map(Self::Message::EditorMessage)
                } else {
                    Command::none()
                }
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
            Main::Loading(state) => {
                let layout = match &state.failed {
                    Some(error) => Column::new()
                        .push(Text::new("Loading failed"))
                        .push(Text::new(format!("{:?}", error))),
                    None => Column::new()
                        .push(Text::new(format!("Loading {:?}", state.file_path)))
                        .push(Text::new(format!(
                            "Completion: {}/100",
                            state.current_step.completion_percentage()
                        )))
                        .push(Text::new(state.current_step.next_description())),
                };

                Container::new(layout).into()
            }
            Main::Loaded(editor) => editor.view().map(MainMessage::EditorMessage),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match self {
            Main::Loading(state) => iced::Subscription::from_recipe(state.notifications.clone())
                .map(Self::Message::LoadProgressed),
            Main::Loaded(state) => state.subscription().map(Self::Message::EditorMessage),
            _ => Subscription::none(),
        }
    }
}
