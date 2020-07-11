use iced::{button, Align, Button, Column, Element, Settings, Text, Row, Length, Container, Application, Command};
use std::path::{PathBuf};

mod data;
mod dialog;

pub fn main() {
    Main::run(Settings::default())
}

enum Main {
    Loader {
        open_button_state: button::State,
        loading_failed: Option<dialog::OpenError>
    },
    Loading(PathBuf),
    Loaded(data::Entity),
}

#[derive(Debug, Clone)]
enum MainMessage {
    OpenFileDialog,
    FileChosen(Result<PathBuf, dialog::OpenError>)
}

impl Application for Main {
    type Executor = iced::executor::Default;
    type Message = MainMessage;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        /*
        let component = Main::Loader {
            open_button_state: button::State::new(),
            loading_failed: None
        }
        */

        let kaylin = data::read_entity_from_path("samples/kaylin.json").unwrap();
        //println!("Reading Kaylin data:\n{:#?}", kaylin);
        let component = Main::Loaded(kaylin);

        (component, Command::none())
    }

    fn title(&self) -> String {
        match self {
            Main::Loader{..} => format!("Pathfinder WotR Editor"),
            Main::Loading(path) => format!("Loading file {:?}", path),
            Main::Loaded(..) => format!("Pathfinder WotR Editor"),
        }
    }

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
            MainMessage::OpenFileDialog => {
                println!("open file dialog");
                Command::perform(dialog::open_file(), MainMessage::FileChosen)
            },
            MainMessage::FileChosen(Ok(path)) => {
                println!("Let's open {:?}", path);
                *self = Main::Loading(path);
                Command::none()
            },
            MainMessage::FileChosen(Err(error)) => {
                *self = Main::Loader {
                    open_button_state: button::State::new(),
                    loading_failed: Some(error)
                };
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<MainMessage> {
        match self {
            Main::Loader { open_button_state, loading_failed } => {
                let mut layout = Column::new()
                    .align_items(Align::Center)
                    .push(
                        Text::new("Pathfinder Editor - Wrath of the Righteous Edition").width(Length::Fill)
                    )
                    .push(
                        Button::new(open_button_state, Text::new("Load a save file"))
                            .on_press(MainMessage::OpenFileDialog)
                            .padding(10)
                    );
                
                    if let Some(error) = loading_failed {
                        layout = layout.push(
                            Text::new(format!("Loading file failed: {:?}", error))
                        );
                    };

                layout.into()
            },
            Main::Loading(path) => Container::new(Text::new(format!("Loading {:?}", path))).into(),
            Main::Loaded(..) => {
                
                let menu = Column::new()
                    .align_items(Align::Start)
                    .width(Length::from(50))
                    .push(Text::new("Party"));

                let characters = Column::new()
                    .width(Length::from(150))
                    .push(Text::new("Kaylin"))
                    .push(Text::new("Solace"))
                    .push(Text::new("Amiri"))
                    .push(Text::new("Ember"));

                // Statistics

                let main_stats = Row::new()
                    .width(Length::Fill)
                    .height(Length::from(50))
                    .push(Text::new("Money: 38747G"))
                    .push(Text::new("Experience: 38747"))
                    .push(Text::new("Alignment: Neutral Good"));

                let abilities_stats = Column::new()
                    .height(Length::Fill)
                    .push(Text::new("STR 10"))
                    .push(Text::new("DEX 15"));

                let combat_stats = Column::new()
                    .push(Text::new("CMB 5"))
                    .push(Text::new("CMD 7"));

                let skills_stats = Column::new()
                    .push(Text::new("Athletics 7"))
                    .push(Text::new("Mobility 13"));

                let statistics = Row::new()
                    .push(abilities_stats)
                    .push(combat_stats)
                    .push(skills_stats);

                let character = Column::new()
                    .width(Length::Fill)
                    .push(main_stats)
                    .push(statistics);

                Row::new()
                    .push(menu)
                    .push(characters)
                    .push(character)
                    .into()
            },
        }
    }
}
