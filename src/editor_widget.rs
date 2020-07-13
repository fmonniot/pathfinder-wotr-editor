use crate::character_view::{self, CharacterView};
use crate::data::Party;
use iced::{
    button, Align, Button, Column, Command, Container, Element, Font, HorizontalAlignment, Length,
    Row, Text, VerticalAlignment,
};

pub struct EditorWidget {
    party: Party,
    secondary_menu_buttons: Vec<button::State>,
    active_character: CharacterView,
    active_character_index: usize,
}

#[derive(Debug, Clone)]
pub struct Message(Msg);

#[derive(Debug, Clone)]
enum Msg {
    SwitchCharacter(usize),
    CharacterMessage(character_view::Msg),
}

impl EditorWidget {
    pub fn new(party: Party) -> EditorWidget {
        let characters_len = party.characters.len() as usize; // Pretty sure you can't more characters than that
        let mut secondary_menu_buttons = Vec::with_capacity(characters_len);

        for _ in 0..characters_len {
            secondary_menu_buttons.push(button::State::new());
        }

        let active_character = CharacterView::new(&party.characters.first().unwrap());

        EditorWidget {
            party,
            secondary_menu_buttons,
            active_character,
            active_character_index: 0,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message(Msg::SwitchCharacter(active_character)) => {
                let character = self.party.characters.get(active_character).unwrap();
                self.active_character = CharacterView::new(character);
                self.active_character_index = active_character;

                Command::none()
            }
            Message(Msg::CharacterMessage(msg)) => {
                self.active_character.update(msg);

                Command::none()
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let menu = Column::new()
            .align_items(Align::Start)
            .push(menu_item("Party"))
            .push(menu_item("Crusade"));
        let menu = Container::new(menu)
            .style(style::MenuSurface)
            .height(Length::Fill);

        let mut characters = Column::new().width(Length::from(150)).height(Length::Fill);

        for (idx, (c, m)) in self
            .party
            .characters
            .iter()
            .zip(&mut self.secondary_menu_buttons)
            .enumerate()
        {
            let active = idx == self.active_character_index;

            characters = characters.push(character_item(c.name(), idx, active, m));
        }

        let characters = Container::new(characters)
            .style(style::SecondaryMenuSurface)
            .height(Length::Fill);

        let character: Element<Message> = self
            .active_character
            .view()
            .map(|msg| Message(Msg::CharacterMessage(msg)));

        Row::new()
            .push(menu)
            .push(characters)
            .push(character)
            .into()
    }
}

fn menu_item(text: &str) -> Element<Message> {
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
) -> Element<'a, Message> {
    let text = Text::new(text)
        .font(BOOKLETTER_1911)
        .size(30)
        .vertical_alignment(VerticalAlignment::Center)
        .horizontal_alignment(HorizontalAlignment::Left);

    let b = Button::new(state, text)
        .on_press(Message(Msg::SwitchCharacter(idx)))
        .width(Length::Fill)
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
