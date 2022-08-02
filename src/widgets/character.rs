use super::input::InputChange;
use super::{alignment, LabelledInputNumber};
use crate::data::Character;
use crate::json::{Id, JsonPatch, JsonPointer};
use crate::widgets::AlignmentWidget;
use iced::{
    pure::{self, Pure},
    Alignment, Column, Command, Container, Element, Length, Row,
};

#[derive(Debug, Clone)]
pub struct Message(Msg);

#[derive(Debug, Clone)]
enum Msg {
    /// Emitted when a Msg is required but nothing should be done
    Nothing,
    StatisticModified {
        entity_id: Field,
        value: String,
    },
    StatisticModifiedNew {
        field: Field,
        value: u64,
    },
    AlignmentWheel(alignment::Message),
}

impl Msg {
    fn statistic_modified(change: InputChange<Field>) -> Message {
        Message(change.map_or_else(
            || Msg::Nothing,
            |entity_id, value| Msg::StatisticModified { entity_id, value },
        ))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(clippy::upper_case_acronyms)]
enum Field {
    // Abilities
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
    // Combat stats
    CMB,
    CMD,
    ArmorClass,
    BaseAttackBonus,
    HitPoints,
    Initiative,
    // Saves
    SaveFortitude,
    SaveReflex,
    SaveWill,
    // Skills
    Athletics,
    Mobility,
    Thievery,
    Stealth,
    KnowledgeArcana,
    KnowledgeWorld,
    LoreNature,
    LoreReligion,
    Perception,
    Persuasion,
    UseMagicDevice,
    // Experience points
    Experience,
    MythicExperience,
}

impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::Strength => write!(f, "Strength"),
            Field::Dexterity => write!(f, "Dexterity"),
            Field::Constitution => write!(f, "Constitution"),
            Field::Intelligence => write!(f, "Intelligence"),
            Field::Wisdom => write!(f, "Wisdom"),
            Field::Charisma => write!(f, "Charisma"),
            Field::CMB => write!(f, "CMB"),
            Field::CMD => write!(f, "CMD"),
            Field::ArmorClass => write!(f, "Armor Class"),
            Field::BaseAttackBonus => write!(f, "Base Attack Bonus"),
            Field::HitPoints => write!(f, "Hit Points"),
            Field::Initiative => write!(f, "Initiative"),
            Field::SaveFortitude => write!(f, "Save: Fortitude"),
            Field::SaveReflex => write!(f, "Save: Reflex"),
            Field::SaveWill => write!(f, "Save: Will"),
            Field::Athletics => write!(f, "Athletics"),
            Field::Mobility => write!(f, "Mobility"),
            Field::Thievery => write!(f, "Thievery"),
            Field::Stealth => write!(f, "Stealth"),
            Field::KnowledgeArcana => write!(f, "Knowledge: Arcana"),
            Field::KnowledgeWorld => write!(f, "Knowledge: World"),
            Field::LoreNature => write!(f, "Lore: Nature"),
            Field::LoreReligion => write!(f, "Lore: Religion"),
            Field::Perception => write!(f, "Perception"),
            Field::Persuasion => write!(f, "Persuasion"),
            Field::UseMagicDevice => write!(f, "Use Magic Device"),
            Field::Experience => write!(f, "Experience"),
            Field::MythicExperience => write!(f, "Mythic Experience"),
        }
    }
}

impl Field {
    fn stat_key(&self) -> Option<&str> {
        match self {
            Field::Strength => Some("Strength"),
            Field::Dexterity => Some("Dexterity"),
            Field::Constitution => Some("Constitution"),
            Field::Intelligence => Some("Intelligence"),
            Field::Wisdom => Some("Wisdom"),
            Field::Charisma => Some("Charisma"),
            Field::CMB => Some("AdditionalCMB"),
            Field::CMD => Some("AdditionalCMD"),
            Field::ArmorClass => Some("AC"),
            Field::BaseAttackBonus => Some("BaseAttackBonus"),
            Field::HitPoints => Some("HitPoints"),
            Field::Initiative => Some("Initiative"),
            Field::SaveFortitude => Some("SaveFortitude"),
            Field::SaveReflex => Some("SaveReflex"),
            Field::SaveWill => Some("SaveWill"),
            Field::Athletics => Some("SkillAthletics"),
            Field::Mobility => Some("SkillMobility"),
            Field::Thievery => Some("SkillThievery"),
            Field::Stealth => Some("SkillStealth"),
            Field::KnowledgeArcana => Some("SkillKnowledgeArcana"),
            Field::KnowledgeWorld => Some("SkillKnowledgeWorld"),
            Field::LoreNature => Some("SkillLoreNature"),
            Field::LoreReligion => Some("SkillLoreReligion"),
            Field::Perception => Some("SkillPerception"),
            Field::Persuasion => Some("SkillPersuasion"),
            Field::UseMagicDevice => Some("SkillUseMagicDevice"),
            Field::Experience => None,
            Field::MythicExperience => None,
        }
    }

    fn build_view(self, character: &Character) -> LabelledInputNumber<Field, u64> {
        let stat_key = self.stat_key();

        match stat_key {
            Some(key) => {
                let stat = character.find_stat(key).unwrap();

                let id = stat.id.clone();
                let ptr = "/m_BaseValue".into();

                if let Some(base_value) = stat.base_value {
                    LabelledInputNumber::new(self, base_value, id, ptr)
                } else {
                    LabelledInputNumber::disabled(self, 0, id, ptr)
                }
            }
            None => match self {
                Field::Experience => LabelledInputNumber::new(
                    self,
                    character.experience,
                    character.id.clone(),
                    "/Descriptor/Progression/Experience".into(),
                ),
                Field::MythicExperience => {
                    let id = character.id.clone();
                    let ptr = "/Descriptor/Progression/MythicExperience".into();

                    if let Some(xp) = character.mythic_experience {
                        LabelledInputNumber::new(self, xp, id, ptr)
                    } else {
                        LabelledInputNumber::disabled(self, 0, id, ptr)
                    }
                }
                _ => panic!(
                    "A field ({:?}) was not matched when building its view, please report",
                    self
                ),
            },
        }
    }
}

struct FieldValue {
    field: Field,
    value: u64,
    disabled: bool,
    id: Id,
    ptr: JsonPointer,
}

impl FieldValue {
    fn from_field(character: &Character, field: Field) -> FieldValue {
        let (value, disabled, id, ptr) = match field.stat_key() {
            Some(key) => {
                let stat = character.find_stat(key).unwrap();

                let id = stat.id.clone();
                let ptr = "/m_BaseValue".into();

                if let Some(base_value) = stat.base_value {
                    (base_value, false, id, ptr)
                } else {
                    (0, true, id, ptr)
                }
            }
            None => match field {
                Field::Experience => (
                    character.experience,
                    false,
                    character.id.clone(),
                    "/Descriptor/Progression/Experience".into(),
                ),
                Field::MythicExperience => {
                    let id = character.id.clone();
                    let ptr = "/Descriptor/Progression/MythicExperience".into();

                    if let Some(xp) = character.mythic_experience {
                        (xp, false, id, ptr)
                    } else {
                        (0, true, id, ptr)
                    }
                }
                _ => panic!(
                    "A field ({:?}) was not matched when building its view, please report",
                    field
                ),
            },
        };

        FieldValue {
            field,
            value,
            disabled,
            id,
            ptr,
        }
    }

    fn change(&self) -> JsonPatch {
        if self.disabled {
            JsonPatch::None
        } else {
            JsonPatch::id_at_pointer(
                self.id.clone(),
                self.ptr.clone(),
                serde_json::to_value(self.value).unwrap(),
            )
        }
    }

    fn view(&self) -> super::input::pure::LabelledInputNumber<u64, Message> {
        let field = self.field;

        super::input::pure::labelled_input_number(
            self.field.to_string(),
            self.value,
            self.id.clone(),
            self.ptr.clone(),
            move |new_value| {
                Message(Msg::StatisticModifiedNew {
                    field: field.clone(),
                    value: new_value,
                })
            },
        )
    }
}

/*
  We have a few more fields we don't display on the UI at the moment
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
pub struct CharacterWidget {
    pub id: Id,

    // tmp state, while we move to iced::pure
    state_1: pure::State,
    state_2: pure::State,

    // Abilities
    strength: LabelledInputNumber<Field, u64>,
    dexterity: LabelledInputNumber<Field, u64>,
    constitution: LabelledInputNumber<Field, u64>,
    intelligence: LabelledInputNumber<Field, u64>,
    wisdom: LabelledInputNumber<Field, u64>,
    charisma: LabelledInputNumber<Field, u64>,
    // Combat stats
    cmb: LabelledInputNumber<Field, u64>,
    cmd: LabelledInputNumber<Field, u64>,
    ac: LabelledInputNumber<Field, u64>,
    bab: LabelledInputNumber<Field, u64>,
    hp: LabelledInputNumber<Field, u64>,
    initiative: LabelledInputNumber<Field, u64>,
    // Saves
    save_fortitude: LabelledInputNumber<Field, u64>,
    save_reflex: LabelledInputNumber<Field, u64>,
    save_will: LabelledInputNumber<Field, u64>,
    // Skills
    athletics: LabelledInputNumber<Field, u64>,
    mobility: LabelledInputNumber<Field, u64>,
    thievery: LabelledInputNumber<Field, u64>,
    stealth: LabelledInputNumber<Field, u64>,
    arcana: LabelledInputNumber<Field, u64>,
    world: LabelledInputNumber<Field, u64>,
    nature: LabelledInputNumber<Field, u64>,
    religion: LabelledInputNumber<Field, u64>,
    perception: LabelledInputNumber<Field, u64>,
    persuasion: LabelledInputNumber<Field, u64>,
    magic_device: LabelledInputNumber<Field, u64>,

    // Experience points
    experience: FieldValue,
    mythic_experience: LabelledInputNumber<Field, u64>,

    // Alignment
    alignment: AlignmentWidget,
}

impl CharacterWidget {
    pub fn new(character: &Character) -> CharacterWidget {
        CharacterWidget {
            id: character.id.clone(),
            state_1: pure::State::new(),
            state_2: pure::State::new(),
            experience: FieldValue::from_field(&character, Field::Experience),
            mythic_experience: Field::MythicExperience.build_view(character),
            strength: Field::Strength.build_view(character),
            dexterity: Field::Dexterity.build_view(character),
            constitution: Field::Constitution.build_view(character),
            intelligence: Field::Intelligence.build_view(character),
            wisdom: Field::Wisdom.build_view(character),
            charisma: Field::Charisma.build_view(character),
            cmb: Field::CMB.build_view(character),
            cmd: Field::CMD.build_view(character),
            ac: Field::ArmorClass.build_view(character),
            bab: Field::BaseAttackBonus.build_view(character),
            hp: Field::HitPoints.build_view(character),
            initiative: Field::Initiative.build_view(character),
            save_fortitude: Field::SaveFortitude.build_view(character),
            save_reflex: Field::SaveReflex.build_view(character),
            save_will: Field::SaveWill.build_view(character),
            athletics: Field::Athletics.build_view(character),
            mobility: Field::Mobility.build_view(character),
            thievery: Field::Thievery.build_view(character),
            stealth: Field::Stealth.build_view(character),
            arcana: Field::KnowledgeArcana.build_view(character),
            world: Field::KnowledgeWorld.build_view(character),
            nature: Field::LoreNature.build_view(character),
            religion: Field::LoreReligion.build_view(character),
            perception: Field::Perception.build_view(character),
            persuasion: Field::Persuasion.build_view(character),
            magic_device: Field::UseMagicDevice.build_view(character),
            alignment: AlignmentWidget::new(character.alignment.clone(), false),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let char_xp: Element<_> = Pure::new(
            &mut self.state_1,
            iced_lazy::pure::component(self.experience.view()),
        )
        .into();

        let myth_xp: Element<_> = self.mythic_experience.view().map(Msg::statistic_modified);

        let main_stats = Row::new()
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .push(myth_xp)
            .push(char_xp);

        let abilities_stats = Column::new()
            .width(Length::FillPortion(1))
            .push(self.strength.view().map(Msg::statistic_modified))
            .push(self.dexterity.view().map(Msg::statistic_modified))
            .push(self.constitution.view().map(Msg::statistic_modified))
            .push(self.intelligence.view().map(Msg::statistic_modified))
            .push(self.wisdom.view().map(Msg::statistic_modified))
            .push(self.charisma.view().map(Msg::statistic_modified));

        let combat_stats = Column::new()
            .width(Length::FillPortion(1))
            .push(self.cmb.view().map(Msg::statistic_modified))
            .push(self.cmd.view().map(Msg::statistic_modified))
            .push(self.ac.view().map(Msg::statistic_modified))
            .push(self.bab.view().map(Msg::statistic_modified))
            .push(self.hp.view().map(Msg::statistic_modified))
            .push(self.initiative.view().map(Msg::statistic_modified))
            .push(self.save_fortitude.view().map(Msg::statistic_modified))
            .push(self.save_reflex.view().map(Msg::statistic_modified))
            .push(self.save_will.view().map(Msg::statistic_modified));

        let skills_stats = Column::new()
            .width(Length::FillPortion(1))
            .push(self.athletics.view().map(Msg::statistic_modified))
            .push(self.mobility.view().map(Msg::statistic_modified))
            .push(self.thievery.view().map(Msg::statistic_modified))
            .push(self.stealth.view().map(Msg::statistic_modified))
            .push(self.arcana.view().map(Msg::statistic_modified))
            .push(self.world.view().map(Msg::statistic_modified))
            .push(self.nature.view().map(Msg::statistic_modified))
            .push(self.religion.view().map(Msg::statistic_modified))
            .push(self.perception.view().map(Msg::statistic_modified))
            .push(self.persuasion.view().map(Msg::statistic_modified))
            .push(self.magic_device.view().map(Msg::statistic_modified));

        let statistics = Row::new()
            .spacing(25)
            .push(abilities_stats)
            .push(combat_stats)
            .push(skills_stats);

        let alignment_wheel: Element<alignment::Message> =
            Pure::new(&mut self.state_2, self.alignment.view()).into();

        Container::new(
            Column::new()
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
                .push(main_stats)
                .push(statistics)
                .push(alignment_wheel.map(|m| Message(Msg::AlignmentWheel(m))))
                .push(iced::widget::Space::new(Length::Fill, Length::Fill)),
        )
        .style(crate::styles::MainPane)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message(Msg::Nothing) => (),
            Message(Msg::StatisticModified { entity_id, value }) => {
                if let Ok(n) = value.parse::<u64>() {
                    self.stat_view_for_field(&entity_id).value = n;
                }
            }
            Message(Msg::StatisticModifiedNew { field, value }) => {
                self.stat_view_for_field(&field).value = value;
            }
            Message(Msg::AlignmentWheel(_m)) => {
                // TODO Will be used when integrating drag & drop for the alignment pin
            }
        };
        Command::none()
    }

    pub fn patches(&self) -> Vec<JsonPatch> {
        vec![
            self.strength.change(),
            self.dexterity.change(),
            self.constitution.change(),
            self.intelligence.change(),
            self.wisdom.change(),
            self.charisma.change(),
            self.cmb.change(),
            self.cmd.change(),
            self.ac.change(),
            self.bab.change(),
            self.hp.change(),
            self.initiative.change(),
            self.save_fortitude.change(),
            self.save_reflex.change(),
            self.save_will.change(),
            self.athletics.change(),
            self.mobility.change(),
            self.thievery.change(),
            self.stealth.change(),
            self.arcana.change(),
            self.world.change(),
            self.nature.change(),
            self.religion.change(),
            self.perception.change(),
            self.persuasion.change(),
            self.magic_device.change(),
            self.experience.change(),
            self.mythic_experience.change(),
        ]
    }

    fn stat_view_for_field(&mut self, field: &Field) -> &mut LabelledInputNumber<Field, u64> {
        match field {
            Field::Strength => &mut self.strength,
            Field::Dexterity => &mut self.dexterity,
            Field::Constitution => &mut self.constitution,
            Field::Intelligence => &mut self.intelligence,
            Field::Wisdom => &mut self.wisdom,
            Field::Charisma => &mut self.charisma,
            Field::CMB => &mut self.cmb,
            Field::CMD => &mut self.cmd,
            Field::ArmorClass => &mut self.ac,
            Field::BaseAttackBonus => &mut self.bab,
            Field::HitPoints => &mut self.hp,
            Field::Initiative => &mut self.initiative,
            Field::SaveFortitude => &mut self.save_fortitude,
            Field::SaveReflex => &mut self.save_reflex,
            Field::SaveWill => &mut self.save_will,
            Field::Athletics => &mut self.athletics,
            Field::Mobility => &mut self.mobility,
            Field::Thievery => &mut self.thievery,
            Field::Stealth => &mut self.stealth,
            Field::KnowledgeArcana => &mut self.arcana,
            Field::KnowledgeWorld => &mut self.world,
            Field::LoreNature => &mut self.nature,
            Field::LoreReligion => &mut self.religion,
            Field::Perception => &mut self.perception,
            Field::Persuasion => &mut self.persuasion,
            Field::UseMagicDevice => &mut self.magic_device,
            Field::Experience => &mut self.experience,
            Field::MythicExperience => &mut self.mythic_experience,
        }
    }
}
