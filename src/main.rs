#![windows_subsystem = "windows"]

use iced::{
    widget::{button, column, container, progress_bar, text},
    Alignment, Element, Length, Subscription, Task,
};
use std::path::PathBuf;

mod data;
mod dialog;
mod json;
mod save;
mod theme;
mod widgets;

use save::{LoadNotifications, LoadingDone, LoadingStep, SaveError, SaveLoader};
use theme::BECKETT_FONT;
use widgets::{EditorMessage, EditorWidget};

const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn main() {
    env_logger::init();
    log::debug!("Running with version {}", VERSION);

    let window = icon_window_settings();
    let save_file: Option<PathBuf> = std::env::args().nth(1).map(|s| s.into());

    iced::application(Main::title, Main::update, Main::view)
        .subscription(Main::subscription)
        .window(window)
        .antialiasing(true)
        .theme(|_state| theme::app_theme())
        .run_with(|| Main::new(save_file))
        .expect("UI runloop couldn't be started")
}

// No error handling as the app.ico file is injected at compile time.
fn icon_window_settings() -> iced::window::Settings {
    let bytes = std::io::Cursor::new(include_bytes!("../assets/app.ico"));
    let icon_dir = ico::IconDir::read(bytes).unwrap();

    let idx = icon_dir.entries().iter().find(|e| e.width() == 48);

    if let Some(entry) = idx {
        let img = entry.decode().unwrap().rgba_data().to_vec();
        let icon = iced::window::icon::from_rgba(img, 48, 48).unwrap();

        iced::window::Settings {
            icon: Some(icon),
            ..iced::window::Settings::default()
        }
    } else {
        iced::window::Settings::default()
    }
}

#[derive(Debug, Clone)]
enum MainMessage {
    OpenFileDialog,
    FileChosen(Result<PathBuf, dialog::OpenError>),
    LoadProgressed(LoadingStep),
    LoadDone(Box<Result<LoadingDone, SaveError>>),
    EditorMessage(EditorMessage),
    FontLoaded(Result<(), iced::font::Error>),
}

enum Main {
    Loader {
        open_failed: Option<dialog::OpenError>,
    },
    Loading {
        notifications: LoadNotifications,
        file_path: PathBuf,
        current_step: LoadingStep,
        failed: Option<SaveError>,
    },
    Loaded(Box<EditorWidget>),
}

impl Main {
    fn new(save_path: Option<PathBuf>) -> (Self, Task<MainMessage>) {
        let (component, command) = match save_path {
            None => (Main::Loader { open_failed: None }, Task::none()),
            Some(file_path) => {
                let (loader, notifications) = SaveLoader::new(file_path.clone());

                let component = Main::Loading {
                    notifications,
                    file_path,
                    current_step: LoadingStep::Initialized,
                    failed: None,
                };
                let command = Task::perform(loader.load(), |r| MainMessage::LoadDone(Box::new(r)));

                (component, command)
            }
        };

        let command = Task::batch(vec![
            command,
            iced::font::load(include_bytes!("../assets/beckett/Beckett-Regular.otf").as_slice())
                .map(MainMessage::FontLoaded),
            iced::font::load(
                include_bytes!("../assets/Goudy_Bookletter_1911/GoudyBookletter1911-Regular.ttf")
                    .as_slice(),
            )
            .map(MainMessage::FontLoaded),
        ]);

        (component, command)
    }

    fn title(&self) -> String {
        match self {
            Main::Loader { .. } => "Pathfinder WotR Editor".to_string(),
            Main::Loading { file_path, .. } => format!("Loading file {:?}", file_path),
            Main::Loaded { .. } => "Pathfinder WotR Editor".to_string(),
        }
    }

    fn update(&mut self, message: MainMessage) -> Task<MainMessage> {
        match message {
            MainMessage::OpenFileDialog => {
                Task::perform(dialog::open_file(), MainMessage::FileChosen)
            }
            MainMessage::FileChosen(Ok(file_path)) => {
                let (loader, notifications) = SaveLoader::new(file_path.clone());

                *self = Main::Loading {
                    notifications,
                    file_path,
                    current_step: LoadingStep::Initialized,
                    failed: None,
                };

                Task::perform(loader.load(), |r| MainMessage::LoadDone(Box::new(r)))
            }
            MainMessage::FileChosen(Err(error)) => {
                *self = Main::Loader {
                    open_failed: Some(error),
                };
                Task::none()
            }
            MainMessage::LoadProgressed(step) => {
                if let Main::Loading {
                    ref mut current_step,
                    ..
                } = self
                {
                    *current_step = step;
                }
                Task::none()
            }
            MainMessage::LoadDone(result) => match *result {
                Ok(done) => {
                    *self = Main::Loaded(Box::new(EditorWidget::new(
                        done.archive_path,
                        done.party,
                        done.player,
                    )));
                    Task::none()
                }
                Err(error) => {
                    if let Main::Loading { ref mut failed, .. } = self {
                        *failed = Some(error);
                    }
                    Task::none()
                }
            },
            MainMessage::EditorMessage(msg) => {
                if let Main::Loaded(ref mut state) = self {
                    state.update(msg).map(MainMessage::EditorMessage)
                } else {
                    Task::none()
                }
            }
            MainMessage::FontLoaded(result) => {
                if let Err(error) = result {
                    log::error!("Couldn't load font. error={error:?}");
                }

                Task::none()
            }
        }
    }

    fn view(&self) -> Element<MainMessage> {
        match self {
            Main::Loader { open_failed } => {
                let mut layout = column(vec![])
                    .align_x(Alignment::Center)
                    .spacing(8)
                    .push(text("Pathfinder Editor").size(60).font(BECKETT_FONT))
                    .push(
                        text("Wrath of the Righteous Edition")
                            .size(45)
                            .font(BECKETT_FONT),
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
                    .center_x(Length::Fill)
                    .center_y(Length::Fill);

                container.into()
            }
            Main::Loading {
                failed,
                file_path,
                current_step,
                ..
            } => {
                let layout = match &failed {
                    Some(error) => column(vec![])
                        .push(text("Loading failed"))
                        .push(text(format!("{:?}", error))),
                    None => column(vec![])
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

                let content = container(layout.spacing(8).align_x(Alignment::Center))
                    .max_width(640)
                    .max_height(480);

                let container = container(content)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill);

                container.into()
            }
            Main::Loaded(editor) => editor.view().map(MainMessage::EditorMessage),
        }
    }

    fn subscription(&self) -> Subscription<MainMessage> {
        match self {
            Main::Loading { notifications, .. } => {
                iced::advanced::subscription::from_recipe(notifications.clone())
                    .map(MainMessage::LoadProgressed)
            }
            Main::Loaded(state) => state.subscription().map(MainMessage::EditorMessage),
            _ => Subscription::none(),
        }
    }
}
