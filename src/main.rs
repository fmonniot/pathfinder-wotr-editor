use iced::{
    button, text_input, Align, Application, Button, Column, Command, Container, Element, Font,
    HorizontalAlignment, Length, Row, Settings, Subscription, Text, TextInput, VerticalAlignment,
};
use std::path::PathBuf;

mod data;
mod dialog;
mod loader;

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
}

struct CharacterView {
    index: usize,
    statistics: StatisticsView,
}

impl CharacterView {
    fn new(character: &data::Character, index: usize) -> CharacterView {
        CharacterView {
            index,
            statistics: StatisticsView::new(character),
        }
    }
}

struct StatisticsView {
    // Abilities
    strength: StatView,
    dexterity: StatView,
    constitution: StatView,
    intelligence: StatView,
    wisdom: StatView,
    charisma: StatView,
    // Combat stats
    attack_bonus: StatView,
    cmb: StatView,
    cmd: StatView,
    ac: StatView,
    bab: StatView,
    hp: StatView,
    initiative: StatView,
    // Saves
    save_fortitude: StatView,
    save_reflex: StatView,
    save_will: StatView,
    // Skills
    athletics: StatView,
    mobility: StatView,
    thievery: StatView,
    stealth: StatView,
    arcana: StatView,
    world: StatView,
    nature: StatView,
    religion: StatView,
    perception: StatView,
    persuasion: StatView,
    magic_device: StatView,
    // Money & Experience should also goes here
}

/*
  We have a few more skills we don't display on the UI at the moment
  - "AdditionalDamage",
  - "AttackOfOpportunityCount",
  - "CheckBluff",
  - "CheckDiplomacy",
  - "CheckIntimidate",
  - "DamageNonLethal",
  - "Reach",
  - "SneakAttack",
  - "Speed",
  - "TemporaryHitPoints",
*/
impl StatisticsView {
    fn new(character: &data::Character) -> StatisticsView {
        StatisticsView {
            strength: StatView::new("STR", character.find_stat("Strength").unwrap()),
            dexterity: StatView::new("DEX", character.find_stat("Dexterity").unwrap()),
            constitution: StatView::new("CON", character.find_stat("Constitution").unwrap()),
            intelligence: StatView::new("INT", character.find_stat("Intelligence").unwrap()),
            wisdom: StatView::new("WIS", character.find_stat("Wisdom").unwrap()),
            charisma: StatView::new("CHA", character.find_stat("Charisma").unwrap()),
            attack_bonus: StatView::new(
                "Additional Attack Bonus",
                character.find_stat("AdditionalAttackBonus").unwrap(),
            ),
            cmb: StatView::new("CMB", character.find_stat("AdditionalCMB").unwrap()),
            cmd: StatView::new("CMD", character.find_stat("AdditionalCMD").unwrap()),
            ac: StatView::new("AC", character.find_stat("AC").unwrap()),
            bab: StatView::new("BAB", character.find_stat("BaseAttackBonus").unwrap()),
            hp: StatView::new("HP", character.find_stat("HitPoints").unwrap()),
            initiative: StatView::new("Initiative", character.find_stat("Initiative").unwrap()),
            save_fortitude: StatView::new(
                "Save: Fortitude",
                character.find_stat("SaveFortitude").unwrap(),
            ),
            save_reflex: StatView::new("Save: Reflex", character.find_stat("SaveReflex").unwrap()),
            save_will: StatView::new("Save: Will", character.find_stat("SaveWill").unwrap()),
            athletics: StatView::new("Athletics", character.find_stat("SkillAthletics").unwrap()),
            mobility: StatView::new("Mobility", character.find_stat("SkillMobility").unwrap()),
            thievery: StatView::new("Thievery", character.find_stat("SkillThievery").unwrap()),
            stealth: StatView::new("Stealth", character.find_stat("SkillStealth").unwrap()),
            arcana: StatView::new(
                "Knowledge: Arcana",
                character.find_stat("SkillKnowledgeArcana").unwrap(),
            ),
            world: StatView::new(
                "Knowledge: World",
                character.find_stat("SkillKnowledgeWorld").unwrap(),
            ),
            nature: StatView::new(
                "Lore: Nature",
                character.find_stat("SkillLoreNature").unwrap(),
            ),
            religion: StatView::new(
                "Lore: Religion",
                character.find_stat("SkillLoreReligion").unwrap(),
            ),
            perception: StatView::new(
                "Perception",
                character.find_stat("SkillPerception").unwrap(),
            ),
            persuasion: StatView::new(
                "Persuasion",
                character.find_stat("SkillPersuasion").unwrap(),
            ),
            magic_device: StatView::new(
                "Use Magic Device",
                character.find_stat("SkillUseMagicDevice").unwrap(),
            ),
        }
    }
}

struct StatView {
    label: &'static str,
    id: String,
    text_input: text_input::State,
    value: i16,
}

impl StatView {
    fn new(label: &'static str, stat: &data::Stat) -> StatView {
        StatView {
            label,
            id: stat.id.clone(),
            text_input: text_input::State::new(),
            value: stat.base_value,
        }
    }

    fn view(&mut self) -> Element<MainMessage> {
        let stat_id = self.id.clone();
        let input = TextInput::new(
            &mut self.text_input,
            self.label,
            &self.value.to_string(),
            move |value| {
                let stat_id = stat_id.clone();
                MainMessage::StatisticModified { stat_id, value }
            },
        );

        Row::new()
            .push(Text::new(format!("{}: ", self.label)))
            .push(input)
            .into()
    }
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

        let active_character = CharacterView::new(&party.characters.first().unwrap(), 0);

        Main::Loaded(LoadedState {
            party,
            secondary_menu_buttons,
            active_character,
        })
    }
}

#[derive(Debug, Clone)]
enum MainMessage {
    OpenFileDialog,
    FileChosen(Result<PathBuf, dialog::OpenError>),
    LoadProgressed(LoadingStep),
    SwitchCharacter(usize),
    StatisticModified {
        stat_id: String,
        value: String, // TODO Add a way to find out which stat has been modified
    },
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
                        state.active_character = CharacterView::new(character, active_character);
                    }
                    _ => (),
                };
                Command::none()
            }
            MainMessage::StatisticModified { .. } => Command::none(),
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
                    let mut name = &c.name;
                    if name.is_empty() {
                        name = &c.blueprint;
                    }

                    let active = idx == active_character.index;

                    characters = characters.push(character_item(name, idx, active, m));
                }

                let characters = Container::new(characters)
                    .style(style::SecondaryMenuSurface)
                    .height(Length::Fill);

                // Statistics

                let main_stats = Row::new()
                    .width(Length::Fill)
                    .height(Length::from(50))
                    .align_items(Align::Center)
                    .push(Text::new("Money: 38747G"))
                    .push(Text::new("Experience: 38747"))
                    .push(Text::new("Alignment: Neutral Good"));

                let abilities_stats = Column::new()
                    .height(Length::Fill)
                    .width(Length::FillPortion(1))
                    .push(active_character.statistics.strength.view())
                    .push(active_character.statistics.dexterity.view())
                    .push(active_character.statistics.constitution.view())
                    .push(active_character.statistics.intelligence.view())
                    .push(active_character.statistics.wisdom.view())
                    .push(active_character.statistics.charisma.view());

                let combat_stats = Column::new()
                    .width(Length::FillPortion(1))
                    .push(active_character.statistics.attack_bonus.view())
                    .push(active_character.statistics.cmb.view())
                    .push(active_character.statistics.cmd.view())
                    .push(active_character.statistics.ac.view())
                    .push(active_character.statistics.bab.view())
                    .push(active_character.statistics.hp.view())
                    .push(active_character.statistics.initiative.view())
                    .push(active_character.statistics.save_fortitude.view())
                    .push(active_character.statistics.save_reflex.view())
                    .push(active_character.statistics.save_will.view());

                let skills_stats = Column::new()
                    .width(Length::FillPortion(1))
                    .push(active_character.statistics.athletics.view())
                    .push(active_character.statistics.mobility.view())
                    .push(active_character.statistics.thievery.view())
                    .push(active_character.statistics.stealth.view())
                    .push(active_character.statistics.arcana.view())
                    .push(active_character.statistics.world.view())
                    .push(active_character.statistics.nature.view())
                    .push(active_character.statistics.religion.view())
                    .push(active_character.statistics.perception.view())
                    .push(active_character.statistics.persuasion.view())
                    .push(active_character.statistics.magic_device.view());

                let statistics = Row::new()
                    .spacing(25)
                    .push(abilities_stats)
                    .push(combat_stats)
                    .push(skills_stats);

                let character = Column::new()
                    .width(Length::Fill)
                    .padding(10)
                    .push(main_stats)
                    .push(statistics);

                Row::new()
                    .push(menu)
                    .push(characters)
                    .push(character)
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
    text: &'a str,
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
