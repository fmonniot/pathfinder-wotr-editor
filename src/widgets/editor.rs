use crate::data::{Character, Party, Player};
use crate::json::Id;
use crate::save::{SaveError, SaveNotifications, SavingSaveGame, SavingStep};
use crate::theme::{self, BECKETT_FONT, BOOKLETTER_1911};
use crate::widgets::{CharacterMessage, CharacterWidget, Element, PlayerMessage, PlayerWidget};
use iced::{
    alignment,
    widget::{button, column, container, progress_bar, row, text},
    Alignment, Command, Length, Subscription,
};
use std::path::PathBuf;
use std::vec;

#[derive(Debug, Clone)]
pub struct Message(Msg);

#[derive(Debug, Clone, PartialEq, Copy)]
enum Pane {
    Party,
    Crusade,
}

#[derive(Debug, Clone)]
enum Msg {
    Save,
    ChangeActivePane(Pane),
    SwitchCharacter(Id),
    CharacterMessage(CharacterMessage),
    Player(PlayerMessage),
    SavingChange(SavingStep),
    SavingResult(Box<Result<(), SaveError>>),
}

pub struct EditorWidget {
    archive_path: PathBuf,
    characters: Vec<Character>,
    active_character: Id,
    active_pane: Pane,
    saving: Option<SaveNotifications>,
    save_progress: Option<SavingStep>,

    character_widgets: Vec<CharacterWidget>,
    player_widget: PlayerWidget,
}

impl EditorWidget {
    pub fn new(archive_path: PathBuf, party: Party, player: Player) -> EditorWidget {
        let active_character = party.characters.first().unwrap().id.clone();
        let character_widgets = party.characters.iter().map(CharacterWidget::new).collect();

        EditorWidget {
            archive_path,
            characters: party.characters,
            active_character,
            active_pane: Pane::Party,
            saving: None,
            save_progress: None,

            character_widgets,
            player_widget: PlayerWidget::new(&player),
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        log::debug!("Message received: {:?}", message);
        match message {
            Message(Msg::Save) => {
                let (saving, receiver) = SavingSaveGame::new(
                    self.player_widget.patches(),
                    self.character_widgets
                        .iter()
                        .flat_map(|c| c.patches())
                        .collect(),
                    self.archive_path.clone(),
                );
                self.saving = Some(receiver);

                Command::perform(saving.save(), |res| {
                    Message(Msg::SavingResult(Box::new(res)))
                })
            }
            Message(Msg::SavingChange(step)) => {
                self.save_progress = Some(step);
                Command::none()
            }
            Message(Msg::SavingResult(res)) => {
                // TODO Keep the progress bar on fail (and change its color) ?
                match *res {
                    Ok(()) => log::debug!("Save Game modified successfully"),
                    Err(err) => log::error!("Saving save game failed: {:?}", err),
                };

                // File saved (or failed to), reset the progress bar
                self.save_progress = None;
                self.saving = None;

                Command::none()
            }

            Message(Msg::ChangeActivePane(new_pane)) => {
                self.active_pane = new_pane;
                Command::none()
            }
            Message(Msg::SwitchCharacter(active_character_id)) => {
                self.active_character = active_character_id;

                Command::none()
            }

            Message(Msg::CharacterMessage(msg)) => self
                .active_character_widget_mut()
                .update(msg)
                .map(|msg| Message(Msg::CharacterMessage(msg))),
            Message(Msg::Player(msg)) => self
                .player_widget
                .update(msg)
                .map(|msg| Message(Msg::Player(msg))),
        }
    }

    fn active_character_widget_mut(&mut self) -> &mut CharacterWidget {
        let a = self.active_character.clone();

        self.character_widgets
            .iter_mut()
            .find(|c| c.id == a)
            .unwrap()
    }

    pub fn view(&self) -> Element<Message> {
        let mut container = row(vec![]).push(pane_selector(self.active_pane, self.save_progress));

        match self.active_pane {
            Pane::Party => {
                let a = self.active_character.clone();

                // we unfortunately cannot use `active_character_mut` here because
                // if we do we would borrow self multiple time (for some reason)
                let character = self
                    .character_widgets
                    .iter()
                    .find(|c| c.id == a)
                    .unwrap()
                    .view()
                    .map(|msg| Message(Msg::CharacterMessage(msg)));

                container = container
                    .push(character_selector(&self.characters, &self.active_character))
                    .push(character);
            }

            Pane::Crusade => {
                container = container.push(
                    self.player_widget
                        .view()
                        .map(|msg| Message(Msg::Player(msg))),
                )
            }
        };

        container.into()
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

fn pane_selector(active: Pane, save_progress: Option<SavingStep>) -> Element<'static, Message> {
    let build_tile = |label: &'static str, message: Message, is_active| {
        let txt = text(label)
            .font(BECKETT_FONT)
            .size(30)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center);

        button(txt)
            .on_press(message)
            .width(Length::from(100))
            .height(Length::from(80))
            .padding(1)
            .style(if is_active {
                theme::Button::PaneSelectorActive
            } else {
                theme::Button::PaneSelectorInactive
            })
    };

    let go_to_pane = |target| {
        let label = match target {
            Pane::Party => "Party",
            Pane::Crusade => "Crusade",
        };

        let is_active = target == active;

        build_tile(label, Message(Msg::ChangeActivePane(target)), is_active)
    };

    let mut layout = column(vec![])
        .align_items(Alignment::Start)
        .push(go_to_pane(Pane::Party))
        .push(go_to_pane(Pane::Crusade))
        .push(build_tile("Save", Message(Msg::Save), false));

    if let Some(step) = save_progress {
        let bar = progress_bar(SavingStep::steps_range(), step.number())
            .width(Length::from(100))
            .height(Length::from(10));

        layout = layout.push(bar);
    }

    container(layout)
        .height(Length::Fill)
        .style(theme::Container::PaneSelectorSurface)
        .into()
}

fn character_selector<'a>(
    characters: &[Character],
    active_character_id: &Id,
) -> Element<'a, Message> {
    let mut col = column(vec![]).width(Length::from(150)).height(Length::Fill);

    for character in characters {
        let active = character.id == *active_character_id;

        let text = text(character.name())
            .font(BOOKLETTER_1911)
            .size(30)
            .vertical_alignment(alignment::Vertical::Center)
            .horizontal_alignment(alignment::Horizontal::Left);

        let button = button(text)
            .on_press(Message(Msg::SwitchCharacter(character.id.clone())))
            .width(Length::Fill)
            .style(if active {
                theme::Button::SecondaryMenuItemActive
            } else {
                theme::Button::SecondaryMenuItemInactive
            })
            .padding(10);

        col = col.push(button);
    }

    container(col)
        .style(theme::Container::SecondaryMenuSurface)
        .height(Length::Fill)
        .into()
}
