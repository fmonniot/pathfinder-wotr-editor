use iced::{
    button, Align, Application, Button, Column, Command, Container, Element, Length, ProgressBar,
    Settings, Subscription, Text,
};
use std::path::PathBuf;

mod data;
mod dialog;
mod json;
mod save;
mod styles;
mod widgets;

use save::{LoadNotifications, LoadingDone, LoadingStep, SaveError, SaveLoader};
use styles::CALIGHRAPHIC_FONT;
use widgets::{EditorMessage, EditorWidget};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() {
    env_logger::init();
    log::debug!("Running with version {}", VERSION);

    let flags: Option<PathBuf> = std::env::args().nth(1).map(|s| s.into());

    Main::run(Settings {
        flags,
        antialiasing: true,
        ..Settings::default()
    })
}

struct LoadingState {
    notifications: LoadNotifications,
    file_path: PathBuf,
    current_step: LoadingStep,
    failed: Option<SaveError>,
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
    LoadDone(Box<Result<LoadingDone, SaveError>>),
    EditorMessage(EditorMessage),
}

impl Application for Main {
    type Executor = iced::executor::Default;
    type Message = MainMessage;
    type Flags = Option<PathBuf>;

    fn new(save_path: Self::Flags) -> (Self, Command<Self::Message>) {
        let (component, command) = match save_path {
            None => (
                Main::Loader {
                    open_button_state: button::State::new(),
                    open_failed: None,
                },
                Command::none(),
            ),
            Some(file_path) => {
                let (loader, notifications) = SaveLoader::new(file_path.clone());

                let component = Main::Loading(Box::new(LoadingState {
                    notifications,
                    file_path,
                    current_step: LoadingStep::Initialized,
                    failed: None,
                }));
                let command =
                    Command::perform(loader.load(), |r| MainMessage::LoadDone(Box::new(r)));

                (component, command)
            }
        };

        (component, command)
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

                Command::perform(loader.load(), |r| MainMessage::LoadDone(Box::new(r)))
            }
            MainMessage::FileChosen(Err(error)) => {
                *self = Main::Loader {
                    open_button_state: button::State::new(),
                    open_failed: Some(error),
                };
                Command::none()
            }
            MainMessage::LoadProgressed(step) => {
                if let Main::Loading(ref mut state) = self {
                    state.current_step = step;
                }
                Command::none()
            }
            MainMessage::LoadDone(result) => match *result {
                Ok(done) => {
                    *self = Main::Loaded(Box::new(EditorWidget::new(
                        done.archive_path,
                        done.party,
                        done.player,
                    )));
                    Command::none()
                }
                Err(error) => {
                    if let Main::Loading(ref mut state) = self {
                        state.failed = Some(error);
                    }
                    Command::none()
                }
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
                    .spacing(8)
                    .push(
                        Text::new("Pathfinder Editor")
                            .size(60)
                            .font(CALIGHRAPHIC_FONT),
                    )
                    .push(
                        Text::new("Wrath of the Righteous Edition")
                            .size(45)
                            .font(CALIGHRAPHIC_FONT),
                    )
                    .push(
                        Button::new(open_button_state, Text::new("Load a save game"))
                            .on_press(MainMessage::OpenFileDialog)
                            .padding(10),
                    );

                if let Some(error) = open_failed {
                    layout = layout.push(Text::new(format!("Loading file failed: {}", error)));
                };

                let content = Container::new(layout).max_width(640).max_height(480);

                Container::new(content)
                    .center_x()
                    .center_y()
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            Main::Loading(state) => {
                let layout = match &state.failed {
                    Some(error) => Column::new()
                        .push(Text::new("Loading failed"))
                        .push(Text::new(format!("{:?}", error))),
                    None => Column::new()
                        .push(Text::new(format!(
                            "Loading {:?}",
                            state.file_path.file_name().expect(
                                "File name must be present, otherwise we couldn't be loading it"
                            )
                        )))
                        .push(ProgressBar::new(
                            0.0..=LoadingStep::total_steps(),
                            state.current_step.step_number(),
                        ))
                        .push(Text::new(state.current_step.description())),
                };

                let content = Container::new(layout.spacing(8).align_items(Align::Center))
                    .max_width(640)
                    .max_height(480);

                Container::new(content)
                    .center_x()
                    .center_y()
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
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
