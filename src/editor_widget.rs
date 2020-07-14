use crate::character_view::{self, CharacterView};
use crate::data::{Party, Player};
use iced::{
    button, Align, Button, Column, Command, Container, Element, Font, HorizontalAlignment, Length,
    Row, Text, VerticalAlignment,
};

#[derive(Debug, Clone)]
pub struct Message(Msg);

#[derive(Debug, Clone)]
enum Msg {
    ChangeActivePane(Pane),
    SwitchCharacter(usize),
    CharacterMessage(character_view::Msg),
}

pub struct EditorWidget {
    pane_selector: PaneSelector,
    party: Party,
    secondary_menu_buttons: Vec<button::State>,
    active_character: CharacterView,
    active_character_index: usize,
}

impl EditorWidget {
    pub fn new(party: Party, _player: Player) -> EditorWidget {
        let characters_len = party.characters.len() as usize; // Pretty sure you can't more characters than that
        let mut secondary_menu_buttons = Vec::with_capacity(characters_len);

        for _ in 0..characters_len {
            secondary_menu_buttons.push(button::State::new());
        }

        let active_character = CharacterView::new(&party.characters.first().unwrap());

        EditorWidget {
            pane_selector: PaneSelector::new(),
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
            },
            Message(Msg::ChangeActivePane(new_pane)) => {
                self.pane_selector.active = new_pane;
                Command::none()
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
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
            .push(self.pane_selector.view())
            .push(characters)
            .push(character)
            .into()
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Pane {
    Party,
    Crusade
}

#[derive(Debug, Clone, PartialEq)]
struct PaneSelector {
    party_button: button::State,
    crusade_button: button::State,
    active: Pane
}

impl PaneSelector {

    fn new() -> PaneSelector {
        PaneSelector {
            party_button: button::State::new(),
            crusade_button: button::State::new(),
            active: Pane::Party,
        }
    }

    fn view(&mut self) -> Element<Message> {        
        let item = |pane, state, active| {
            let label = match pane {
                Pane::Party => "Party",
                Pane::Crusade => "Crusade"
            };

            let is_active = &pane == active;

            let txt = Text::new(label)
                .font(CALIGHRAPHIC_FONT)
                .size(30)
                .horizontal_alignment(HorizontalAlignment::Center)
                .vertical_alignment(VerticalAlignment::Center);

            Button::new(state, txt)
                .on_press(Message(Msg::ChangeActivePane(pane)))
                .width(Length::from(100))
                .height(Length::from(80))
                .padding(1)
                .style(if is_active {
                    style::PaneSelectorButton::Selected
                } else {
                    style::PaneSelectorButton::Inactive
                })
        };

        let layout = Column::new()
            .align_items(Align::Start)
            .push(item(Pane::Party, &mut self.party_button, &self.active))
            .push(item(Pane::Crusade, &mut self.crusade_button, &self.active));

        Container::new(layout)
            .height(Length::Fill)
            .style(style::PaneSelectorSurface)
            .into()
    }


}

fn character_item(
    text: String,
    idx: usize,
    active: bool,
    state: &mut button::State,
) -> Element<Message> {
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
    use iced::{button, container, Background, Color};


    /// style marker for the widgets composing the left hand side menu structure.
    /// This part of the UI is basically a list of buttons which can be in two states:
    /// - inactive: a button is inactive when its pane isn't the one displayed or the user
    ///             doesn't have the focus on it,
    /// - selected: the opposite, when the button's pane is displayed or the user has
    ///             put focus on this button (hover or press).
    pub struct PaneSelectorSurface;

    impl container::StyleSheet for PaneSelectorSurface {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(Color::from_rgb8(0x25, 0x27, 0x29))),
                ..container::Style::default()
            }
        }
    }

    pub enum PaneSelectorButton {
        Selected,
        Inactive
    }

    impl PaneSelectorButton {
        fn inactive(&self) -> button::Style {
            button::Style {
                border_radius: 0,
                text_color: Color::from_rgb8(0xdd, 0xdd, 0xdd),
                ..button::Style::default()
            }
        }

        fn selected(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(Color::from_rgb8(0x2e, 0x31, 0x36))),
                text_color: Color::WHITE,
                ..self.inactive()
            }
        }
    }

    impl button::StyleSheet for PaneSelectorButton {

        // Strangely enough, the active() method return a style used when the button is not active :)
        fn active(&self) -> button::Style {
            match self {
                PaneSelectorButton::Inactive => self.inactive(),
                PaneSelectorButton::Selected => self.selected(),
            }
        }

        fn hovered(&self) -> button::Style {
            self.selected()
        }

        fn pressed(&self) -> button::Style {
            self.selected()
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
