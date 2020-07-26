use crate::character_widget::{self, CharacterWidget};
use crate::data::{Party, Player};
use crate::json::Id;
use crate::player_widget::{self, PlayerWidget};
use crate::save::{SaveError, SaveNotifications, SavingSaveGame, SavingStep};
use iced::{
    button, Align, Button, Column, Command, Container, Element, Font, HorizontalAlignment, Length,
    Row, Subscription, Text, VerticalAlignment,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Message(Msg);

#[derive(Debug, Clone)]
enum Msg {
    ChangeActivePane(Pane),
    SwitchCharacter(Id),
    CharacterMessage(character_widget::Message),
    Player(player_widget::Message),
    SavingChange(SavingStep),
    SavingResult(Box<Result<(), SaveError>>),
}

pub struct EditorWidget {
    archive_path: PathBuf,
    party: Party,
    pane_selector: PaneSelector,
    character_selector: CharacterSelector,
    active_character: CharacterWidget,
    player_widget: PlayerWidget,
    saving: Option<SaveNotifications>,
}

impl EditorWidget {
    pub fn new(archive_path: PathBuf, party: Party, player: Player) -> EditorWidget {
        let character_selector = CharacterSelector::new(&party.characters);
        let active_character = CharacterWidget::new(&party.characters.first().unwrap());

        EditorWidget {
            archive_path,
            pane_selector: PaneSelector::new(),
            character_selector,
            party,
            active_character,
            player_widget: PlayerWidget::new(&player),
            saving: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        log::debug!("Message received: {:?}", message);
        match message {
            Message(Msg::ChangeActivePane(Pane::Save)) => {
                // TODO How does it work for multiple characters ? At the moment I'd say it doesn't work :(
                let (saving, receiver) = SavingSaveGame::new(
                    self.player_widget.patches(),
                    self.active_character.patches(),
                    self.archive_path.clone(),
                );
                self.saving = Some(receiver);

                Command::perform(saving.save(), |res| {
                    Message(Msg::SavingResult(Box::new(res)))
                })
            }
            Message(Msg::ChangeActivePane(new_pane)) => {
                self.pane_selector.active = new_pane;
                Command::none()
            }
            Message(Msg::SwitchCharacter(active_character_id)) => {
                let character = self
                    .party
                    .characters
                    .iter()
                    .find(|c| c.id == active_character_id)
                    .unwrap();
                self.active_character = CharacterWidget::new(character);
                self.character_selector.active_character_id = active_character_id;

                Command::none()
            }
            Message(Msg::CharacterMessage(msg)) => self
                .active_character
                .update(msg)
                .map(|msg| Message(Msg::CharacterMessage(msg))),
            Message(Msg::Player(msg)) => self
                .player_widget
                .update(msg)
                .map(|msg| Message(Msg::Player(msg))),
            Message(Msg::SavingChange(_)) => Command::none(),
            Message(Msg::SavingResult(res)) => {
                match *res {
                    Ok(()) => log::debug!("Save Game modified successfully"),
                    Err(err) => log::error!("Saving save game failed: {:?}", err),
                };
                Command::none()
            }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        match self.pane_selector.active {
            Pane::Party => Row::new()
                .push(self.pane_selector.view())
                .push(self.character_selector.view())
                .push(
                    self.active_character
                        .view()
                        .map(|msg| Message(Msg::CharacterMessage(msg))),
                )
                .into(),

            Pane::Crusade => Row::new()
                .push(self.pane_selector.view())
                .push(
                    self.player_widget
                        .view()
                        .map(|msg| Message(Msg::Player(msg))),
                )
                .into(),

            Pane::Save => {
                unreachable!("Save is a hack for the time being and should not be reached")
            }
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        match &self.saving {
            Some(s) => {
                iced::Subscription::from_recipe(s.clone()).map(|s| Message(Msg::SavingChange(s)))
            }
            None => Subscription::none(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Pane {
    Party,
    Crusade,
    Save,
}

#[derive(Debug, Clone, PartialEq)]
struct PaneSelector {
    party_button: button::State,
    crusade_button: button::State,
    save_button: button::State,
    active: Pane,
}

impl PaneSelector {
    fn new() -> PaneSelector {
        PaneSelector {
            party_button: button::State::new(),
            crusade_button: button::State::new(),
            save_button: button::State::new(),
            active: Pane::Party,
        }
    }

    fn view(&mut self) -> Element<Message> {
        let item = |pane, state, active| {
            let label = match pane {
                Pane::Party => "Party",
                Pane::Crusade => "Crusade",
                Pane::Save => "Save",
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
            .push(item(Pane::Crusade, &mut self.crusade_button, &self.active))
            .push(item(Pane::Save, &mut self.save_button, &self.active));

        Container::new(layout)
            .height(Length::Fill)
            .style(style::PaneSelectorSurface)
            .into()
    }
}

struct CharacterSelector {
    characters: Vec<(button::State, String, Id)>, // (state, name, id)
    active_character_id: Id,
}

impl CharacterSelector {
    fn new(characters: &[crate::data::Character]) -> CharacterSelector {
        let characters = characters
            .iter()
            .map(|c| (button::State::new(), c.name(), c.id.clone()))
            .collect::<Vec<_>>();

        let active_character_id = characters.first().unwrap().2.clone();

        CharacterSelector {
            characters,
            active_character_id,
        }
    }

    fn view(&mut self) -> Element<Message> {
        let mut characters = Column::new().width(Length::from(150)).height(Length::Fill);

        for (ref mut state, ref name, id) in &mut self.characters {
            let active = id == &self.active_character_id;

            let text = Text::new(name)
                .font(BOOKLETTER_1911)
                .size(30)
                .vertical_alignment(VerticalAlignment::Center)
                .horizontal_alignment(HorizontalAlignment::Left);

            let button = Button::new(state, text)
                .on_press(Message(Msg::SwitchCharacter(id.clone())))
                .width(Length::Fill)
                .style(if active {
                    style::SecondaryMenuItem::Selected
                } else {
                    style::SecondaryMenuItem::Inactive
                })
                .padding(10);

            characters = characters.push(button);
        }

        Container::new(characters)
            .style(style::SecondaryMenuSurface)
            .height(Length::Fill)
            .into()
    }
}

// Fonts (might need to move to style.rs if I create that file)
const CALIGHRAPHIC_FONT: Font = Font::External {
    name: "Caligraphic",
    bytes: include_bytes!("../fonts/beckett/BECKETT.TTF"),
};

// Should probably be used by main pane widgets too
const BOOKLETTER_1911: Font = Font::External {
    name: "Goudy_Bookletter_1911",
    bytes: include_bytes!("../fonts/Goudy_Bookletter_1911/GoudyBookletter1911-Regular.ttf"),
};

// Might need to move to another file, as its now used by character_view (and probably crusade_widget when it comes up)
pub mod style {
    use iced::{button, container, progress_bar, radio, text_input, Background, Color};

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
        Inactive,
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

    pub struct SecondaryMenuSurface;

    impl container::StyleSheet for SecondaryMenuSurface {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(Color::from_rgb8(0x2d, 0x30, 0x34))),
                ..container::Style::default()
            }
        }
    }

    pub enum SecondaryMenuItem {
        Selected,
        Inactive,
    }

    impl SecondaryMenuItem {
        fn inactive(&self) -> button::Style {
            button::Style {
                border_radius: 0,
                text_color: Color::from_rgb8(0x87, 0x90, 0x9c),
                ..button::Style::default()
            }
        }

        fn selected(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(Color::from_rgb8(0x36, 0x39, 0x3F))),
                text_color: Color::WHITE,
                ..self.inactive()
            }
        }
    }

    impl button::StyleSheet for SecondaryMenuItem {
        // Strangely enough, the active() method return a style used when the button is not active :)
        fn active(&self) -> button::Style {
            match self {
                SecondaryMenuItem::Inactive => self.inactive(),
                SecondaryMenuItem::Selected => self.selected(),
            }
        }

        fn hovered(&self) -> button::Style {
            self.selected()
        }

        fn pressed(&self) -> button::Style {
            self.selected()
        }
    }

    // Can't use from_rgb8 because the constructor needs to be const fn
    const SURFACE: Color = Color::from_rgb(
        0x40 as f32 / 255.0,
        0x44 as f32 / 255.0,
        0x4B as f32 / 255.0,
    );

    const ACCENT: Color = Color::from_rgb(
        0x6F as f32 / 255.0,
        0xFF as f32 / 255.0,
        0xE9 as f32 / 255.0,
    );

    const ACTIVE: Color = Color::from_rgb(
        0x72 as f32 / 255.0,
        0x89 as f32 / 255.0,
        0xDA as f32 / 255.0,
    );

    const HOVERED: Color = Color::from_rgb(
        0x67 as f32 / 255.0,
        0x7B as f32 / 255.0,
        0xC4 as f32 / 255.0,
    );

    pub struct MainPane;

    impl container::StyleSheet for MainPane {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color(Color::from_rgb8(0x36, 0x39, 0x3F))),
                text_color: Some(Color::WHITE),
                ..container::Style::default()
            }
        }
    }

    impl radio::StyleSheet for MainPane {
        fn active(&self) -> radio::Style {
            radio::Style {
                background: Background::Color(SURFACE),
                dot_color: ACTIVE,
                border_width: 1,
                border_color: ACTIVE,
            }
        }

        fn hovered(&self) -> radio::Style {
            radio::Style {
                background: Background::Color(Color { a: 0.5, ..SURFACE }),
                ..self.active()
            }
        }
    }

    impl text_input::StyleSheet for MainPane {
        fn active(&self) -> text_input::Style {
            text_input::Style {
                background: Background::Color(SURFACE),
                border_radius: 2,
                border_width: 0,
                border_color: Color::TRANSPARENT,
            }
        }

        fn focused(&self) -> text_input::Style {
            text_input::Style {
                border_width: 1,
                border_color: ACCENT,
                ..self.active()
            }
        }

        fn hovered(&self) -> text_input::Style {
            text_input::Style {
                border_width: 1,
                border_color: Color { a: 0.3, ..ACCENT },
                ..self.focused()
            }
        }

        fn placeholder_color(&self) -> Color {
            Color::from_rgb(0.4, 0.4, 0.4)
        }

        fn value_color(&self) -> Color {
            Color::WHITE
        }

        fn selection_color(&self) -> Color {
            ACTIVE
        }
    }

    impl button::StyleSheet for MainPane {
        fn active(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(ACTIVE)),
                border_radius: 3,
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        }

        fn hovered(&self) -> button::Style {
            button::Style {
                background: Some(Background::Color(HOVERED)),
                text_color: Color::WHITE,
                ..self.active()
            }
        }

        fn pressed(&self) -> button::Style {
            button::Style {
                border_width: 1,
                border_color: Color::WHITE,
                ..self.hovered()
            }
        }
    }

    impl progress_bar::StyleSheet for MainPane {
        fn style(&self) -> progress_bar::Style {
            progress_bar::Style {
                background: Background::Color(SURFACE),
                bar: Background::Color(ACTIVE),
                border_radius: 10,
            }
        }
    }
}
