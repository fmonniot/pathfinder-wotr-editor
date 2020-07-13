use iced::{
    button, Align, Application, Button, Column, Command, Container, Element, Font,
    HorizontalAlignment, Length, Row, Settings, Subscription, Text, VerticalAlignment,
};
use std::path::PathBuf;

mod character_view;
mod data;
mod dialog;
mod loader;

use character_view::CharacterView;
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

struct LoadedState {
    party: data::Party,
    secondary_menu_buttons: Vec<button::State>,
    active_character: CharacterView,
    active_character_index: usize,
}

enum Main {
    Loader {
        open_button_state: button::State,
        open_failed: Option<dialog::OpenError>,
    },
    Loading(LoadingState),
    Loaded(LoadedState),
}

impl Main {
    fn new_loaded(party: data::Party) -> Main {
        let characters_len = party.characters.len() as usize; // Pretty sure you can't more characters than that
        let mut secondary_menu_buttons = Vec::with_capacity(characters_len);

        for _ in 0..characters_len {
            secondary_menu_buttons.push(button::State::new());
        }

        let active_character = CharacterView::new(&party.characters.first().unwrap());

        Main::Loaded(LoadedState {
            party,
            secondary_menu_buttons,
            active_character,
            active_character_index: 0,
        })
    }
}

#[derive(Debug, Clone)]
enum MainMessage {
    OpenFileDialog,
    FileChosen(Result<PathBuf, dialog::OpenError>),
    LoadProgressed(LoadingStep),
    SwitchCharacter(usize),
    CharacterMessage(character_view::Msg),
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

        let component = Main::new_loaded(party);

        (component, Command::none())
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
                        *self = Main::new_loaded(party);
                        Command::none()
                    }
                    _ => {
                        state.current_step = step;
                        Command::none()
                    }
                },
                _ => Command::none(),
            },
            MainMessage::SwitchCharacter(active_character) => {
                match self {
                    Main::Loaded(ref mut state) => {
                        let character = state.party.characters.get(active_character).unwrap();
                        state.active_character = CharacterView::new(character);
                        state.active_character_index = active_character;
                    }
                    _ => (),
                };
                Command::none()
            }
            MainMessage::CharacterMessage(msg) => {
                match self {
                    Main::Loaded(ref mut state) => {
                        state.active_character.update(msg);
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
            Main::Loaded(LoadedState {
                party,
                secondary_menu_buttons,
                active_character,
                active_character_index,
            }) => {
                let menu = Column::new()
                    .align_items(Align::Start)
                    .push(menu_item("Party"));
                let menu = Container::new(menu)
                    .style(style::MenuSurface)
                    .height(Length::Fill);

                let mut characters = Column::new().width(Length::from(150)).height(Length::Fill);

                for (idx, (c, m)) in party
                    .characters
                    .iter()
                    .zip(secondary_menu_buttons)
                    .enumerate()
                {
                    let active = &idx == active_character_index;

                    characters = characters.push(character_item(c.name(), idx, active, m));
                }

                let characters = Container::new(characters)
                    .style(style::SecondaryMenuSurface)
                    .height(Length::Fill);

                // Statistics

                Row::new()
                    .push(menu)
                    .push(characters)
                    .push(active_character.view().map(MainMessage::CharacterMessage))
                    .into()
            }
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

fn menu_item(text: &str) -> Element<MainMessage> {
    let length = Length::from(75);

    let text = Text::new(text)
        .font(CALIGHRAPHIC_FONT)
        .horizontal_alignment(HorizontalAlignment::Center)
        .vertical_alignment(VerticalAlignment::Center)
        .width(length)
        .height(length)
        .size(30);

    Container::new(text).style(style::MenuItem).into()
}

fn character_item<'a>(
    text: String,
    idx: usize,
    active: bool,
    state: &'a mut button::State,
) -> Element<'a, MainMessage> {
    let text = Text::new(text)
        .font(BOOKLETTER_1911)
        .size(30)
        .vertical_alignment(VerticalAlignment::Center)
        .horizontal_alignment(HorizontalAlignment::Left);

    let b = Button::new(state, text)
        .on_press(MainMessage::SwitchCharacter(idx))
        .padding(10);

    Container::new(b)
        .padding(10)
        .style(if active {
            style::SecondaryMenuItem::Active
        } else {
            style::SecondaryMenuItem::Inactive
        })
        .width(Length::Fill)
        .into()
}

// Fonts
const CALIGHRAPHIC_FONT: Font = Font::External {
    name: "Caligraphic",
    bytes: include_bytes!("../fonts/beckett/BECKETT.TTF"),
};

const BOOKLETTER_1911: Font = Font::External {
    name: "Goudy_Bookletter_1911",
    bytes: include_bytes!("../fonts/Goudy_Bookletter_1911/GoudyBookletter1911-Regular.ttf"),
};

mod style {
    use iced::{container, Background, Color};

    pub struct MenuSurface;

    impl container::StyleSheet for MenuSurface {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(Color::from_rgb8(0x2e, 0x31, 0x36))),
                ..container::Style::default()
            }
        }
    }

    /*
    $layoutBackground: #2e3136;
    $layoutTabsBackground: #252729;
    $layoutTabsActiveColor: #ffffff;
    */
    pub struct MenuItem;

    impl container::StyleSheet for MenuItem {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(Color::from_rgb8(0x2e, 0x31, 0x36))),
                border_width: 1,
                border_color: Color::from_rgb8(0x4c, 0x50, 0x53),
                text_color: Some(Color::WHITE),
                ..container::Style::default()
            }
        }
    }

    /*
    // Secondary menu (eg. characters in a party)
    $sidebarBackground: #36393e;
    $sidebarActiveBackground: #414448;
    $sidebarSubmenuActiveColor: #ffffff;
    $sidebarSubmenuActiveBackground: #00796b;
    */

    pub struct SecondaryMenuSurface;

    impl container::StyleSheet for SecondaryMenuSurface {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(Color::from_rgb8(0x36, 0x39, 0x3e))),
                ..container::Style::default()
            }
        }
    }

    pub enum SecondaryMenuItem {
        Active,
        Inactive,
    }

    impl container::StyleSheet for SecondaryMenuItem {
        fn style(&self) -> container::Style {
            let (bg, text) = match self {
                SecondaryMenuItem::Inactive => (
                    Color::from_rgb8(0x36, 0x39, 0x3e),
                    Color::from_rgb8(0x87, 0x90, 0x9c),
                ),
                SecondaryMenuItem::Active => (Color::from_rgb8(0x41, 0x44, 0x48), Color::WHITE),
            };

            container::Style {
                background: Some(Background::Color(bg)),
                text_color: Some(text),
                ..container::Style::default()
            }
        }
    }
}
