use crate::data::{Party, Player};
use crate::json::Id;
use crate::save::{SaveError, SaveNotifications, SavingSaveGame, SavingStep};
use crate::styles::{self, BOOKLETTER_1911, CALIGHRAPHIC_FONT};
use crate::widgets::{CharacterMessage, CharacterWidget, PlayerMessage, PlayerWidget};
use iced::{
    button, Align, Button, Column, Command, Container, Element, HorizontalAlignment, Length, Row,
    Subscription, Text, VerticalAlignment,
};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Message(Msg);

#[derive(Debug, Clone)]
enum Msg {
    ChangeActivePane(Pane),
    SwitchCharacter(Id),
    CharacterMessage(CharacterMessage),
    Player(PlayerMessage),
    SavingChange(SavingStep),
    SavingResult(Box<Result<(), SaveError>>),
}

pub struct EditorWidget {
    archive_path: PathBuf,
    pane_selector: PaneSelector,
    character_selector: CharacterSelector,
    active_character: Id,
    characters: Vec<CharacterWidget>,
    player_widget: PlayerWidget,
    saving: Option<SaveNotifications>,
}

impl EditorWidget {
    pub fn new(archive_path: PathBuf, party: Party, player: Player) -> EditorWidget {
        let character_selector = CharacterSelector::new(&party.characters);
        let active_character = party.characters.first().unwrap().id.clone();
        let characters = party.characters.iter().map(CharacterWidget::new).collect();

        EditorWidget {
            archive_path,
            pane_selector: PaneSelector::new(),
            character_selector,
            active_character,
            characters,
            player_widget: PlayerWidget::new(&player),
            saving: None,
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        log::debug!("Message received: {:?}", message);
        match message {
            Message(Msg::ChangeActivePane(Pane::Save)) => {
                let (saving, receiver) = SavingSaveGame::new(
                    self.player_widget.patches(),
                    self.characters.iter().flat_map(|c| c.patches()).collect(),
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
                self.active_character = active_character_id.clone();
                self.character_selector.active_character_id = active_character_id;

                Command::none()
            }
            Message(Msg::CharacterMessage(msg)) => self
                .active_character_mut()
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

    fn active_character_mut(&mut self) -> &mut CharacterWidget {
        let a = self.active_character.clone();

        self.characters.iter_mut().find(|c| c.id == a).unwrap()
    }

    pub fn view(&mut self) -> Element<Message> {
        match self.pane_selector.active {
            Pane::Party => {
                let a = self.active_character.clone();

                // we unfortunately cannot use `active_character_mut` here because
                // if we do we would borrow self multiple time (for some reason)
                let character = self
                    .characters
                    .iter_mut()
                    .find(|c| c.id == a)
                    .unwrap()
                    .view()
                    .map(|msg| Message(Msg::CharacterMessage(msg)));

                Row::new()
                    .push(self.pane_selector.view())
                    .push(self.character_selector.view())
                    .push(character)
                    .into()
            }

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
                    styles::PaneSelectorButton::Selected
                } else {
                    styles::PaneSelectorButton::Inactive
                })
        };

        let layout = Column::new()
            .align_items(Align::Start)
            .push(item(Pane::Party, &mut self.party_button, &self.active))
            .push(item(Pane::Crusade, &mut self.crusade_button, &self.active))
            .push(item(Pane::Save, &mut self.save_button, &self.active));

        Container::new(layout)
            .height(Length::Fill)
            .style(styles::PaneSelectorSurface)
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
                    styles::SecondaryMenuItem::Selected
                } else {
                    styles::SecondaryMenuItem::Inactive
                })
                .padding(10);

            characters = characters.push(button);
        }

        Container::new(characters)
            .style(styles::SecondaryMenuSurface)
            .height(Length::Fill)
            .into()
    }
}
