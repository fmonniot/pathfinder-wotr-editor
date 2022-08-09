#![windows_subsystem = "windows"]

use iced::{
    pure::{self, button, column, container, progress_bar, text, Pure},
    Alignment, Application, Command, Element, Length, Settings, Subscription, 
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

    let window = icon_window_settings();
    let flags: Option<PathBuf> = std::env::args().nth(1).map(|s| s.into());

    Main::run(Settings {
        window,
        flags,
        antialiasing: true,
        ..Settings::default()
    })
    .expect("UI runloop couldn't be started")
}

// No error handling as the app.ico file is injected at compile time.
fn icon_window_settings() -> iced::window::Settings {
    let bytes = std::io::Cursor::new(include_bytes!("../assets/app.ico"));
    let icon_dir = ico::IconDir::read(bytes).unwrap();

    let idx = icon_dir.entries().iter().find(|e| e.width() == 48);

    if let Some(entry) = idx {
        let img = entry.decode().unwrap().rgba_data().to_vec();
        let icon = iced::window::Icon::from_rgba(img, 48, 48).unwrap();

        iced::window::Settings {
            icon: Some(icon),
            ..iced::window::Settings::default()
        }
    } else {
        iced::window::Settings::default()
    }
}

enum Main {
    Loader {
        pure_state: pure::State,
        open_failed: Option<dialog::OpenError>,
    },
    Loading {
        pure_state: pure::State,
        notifications: LoadNotifications,
        file_path: PathBuf,
        current_step: LoadingStep,
        failed: Option<SaveError>,
    },
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
                    pure_state: pure::State::new(),
                    open_failed: None,
                },
                Command::none(),
            ),
            Some(file_path) => {
                let (loader, notifications) = SaveLoader::new(file_path.clone());

                let component = Main::Loading {
                    pure_state: pure::State::new(),
                    notifications,
                    file_path,
                    current_step: LoadingStep::Initialized,
                    failed: None,
                };
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
            Main::Loading { file_path, .. } => format!("Loading file {:?}", file_path),
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

                *self = Main::Loading {
                    pure_state: pure::State::new(),
                    notifications,
                    file_path,
                    current_step: LoadingStep::Initialized,
                    failed: None,
                };

                Command::perform(loader.load(), |r| MainMessage::LoadDone(Box::new(r)))
            }
            MainMessage::FileChosen(Err(error)) => {
                *self = Main::Loader {
                    pure_state: pure::State::new(),
                    open_failed: Some(error),
                };
                Command::none()
            }
            MainMessage::LoadProgressed(step) => {
                if let Main::Loading {
                    ref mut current_step,
                    ..
                } = self
                {
                    *current_step = step;
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
                    if let Main::Loading { ref mut failed, .. } = self {
                        *failed = Some(error);
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
                pure_state,
                open_failed,
            } => {
                let mut layout = column()
                    .align_items(Alignment::Center)
                    .spacing(8)
                    .push(text("Pathfinder Editor").size(60).font(CALIGHRAPHIC_FONT))
                    .push(
                        text("Wrath of the Righteous Edition")
                            .size(45)
                            .font(CALIGHRAPHIC_FONT),
                    )
                    .push(
                        button(text("Load a save game"))
                            .on_press(MainMessage::OpenFileDialog)
                            .padding(10),
                    );

                if let Some(error) = open_failed {
                    layout = layout.push(text(format!("Loading file failed: {}", error)));
                };

                let content = container(layout).max_width(640).max_height(480);

                let container = container(content)
                    .center_x()
                    .center_y()
                    .width(Length::Fill)
                    .height(Length::Fill);

                Pure::new(pure_state, container).into()
            }
            Main::Loading {
                pure_state,
                failed,
                file_path,
                current_step,
                ..
            } => {
                let layout = match &failed {
                    Some(error) => column()
                        .push(text("Loading failed"))
                        .push(text(format!("{:?}", error))),
                    None => column()
                        .push(text(format!(
                            "Loading {:?}",
                            file_path.file_name().expect(
                                "File name must be present, otherwise we couldn't be loading it"
                            )
                        )))
                        .push(progress_bar(
                            0.0..=LoadingStep::total_steps(),
                            current_step.step_number(),
                        ))
                        .push(text(current_step.description())),
                };

                let content = container(layout.spacing(8).align_items(Alignment::Center))
                    .max_width(640)
                    .max_height(480);

                let container = container(content)
                    .center_x()
                    .center_y()
                    .width(Length::Fill)
                    .height(Length::Fill);

                Pure::new(pure_state, container).into()
            }
            Main::Loaded(editor) => editor.view().map(MainMessage::EditorMessage),
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        match self {
            Main::Loading { notifications, .. } => {
                iced::Subscription::from_recipe(notifications.clone())
                    .map(Self::Message::LoadProgressed)
            }
            Main::Loaded(state) => state.subscription().map(Self::Message::EditorMessage),
            _ => Subscription::none(),
        }
    }
}
