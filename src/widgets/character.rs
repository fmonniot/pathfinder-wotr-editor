use super::alignment;
use super::input::labelled_input_number;
use crate::data::Character;
use crate::json::{Id, JsonPatch, JsonPointer};
use crate::theme;
use crate::widgets::{AlignmentWidget, Element};
use iced::{
    widget::{column, container, row},
    Alignment, Command, Length,
};

#[derive(Debug, Clone)]
pub struct Message(Msg);

#[derive(Debug, Clone)]
enum Msg {
    /// Emitted when a Msg is required but nothing should be done
    StatisticModified {
        field: Field,
        value: u64,
    },
    AlignmentWheel(alignment::Message),
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
            Field::LoreNature => write!(f, "Lore: Nature"),
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

    fn view(&self) -> Element<'_, Message> {
        let field = self.field;

        let mut input = labelled_input_number(self.field.to_string(), self.value, move |value| {
            Message(Msg::StatisticModified { field, value })
        });

        if self.disabled {
            input = input.disabled();
        }

        iced_lazy::component(input)
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

    // Abilities
    strength: FieldValue,
    dexterity: FieldValue,
    constitution: FieldValue,
    intelligence: FieldValue,
    wisdom: FieldValue,
    charisma: FieldValue,
    // Combat stats
    cmb: FieldValue,
    cmd: FieldValue,
    ac: FieldValue,
    bab: FieldValue,
    hp: FieldValue,
    initiative: FieldValue,
    // Saves
    save_fortitude: FieldValue,
    save_reflex: FieldValue,
    save_will: FieldValue,
    // Skills
    athletics: FieldValue,
    mobility: FieldValue,
    thievery: FieldValue,
    stealth: FieldValue,
    arcana: FieldValue,
    world: FieldValue,
    nature: FieldValue,
    religion: FieldValue,
    perception: FieldValue,
    persuasion: FieldValue,
    magic_device: FieldValue,

    // Experience points
    experience: FieldValue,
    mythic_experience: FieldValue,

    // Alignment
    alignment: AlignmentWidget,
}

impl CharacterWidget {
    pub fn new(character: &Character) -> CharacterWidget {
        CharacterWidget {
            id: character.id.clone(),
            experience: FieldValue::from_field(character, Field::Experience),
            mythic_experience: FieldValue::from_field(character, Field::MythicExperience),
            strength: FieldValue::from_field(character, Field::Strength),
            dexterity: FieldValue::from_field(character, Field::Dexterity),
            constitution: FieldValue::from_field(character, Field::Constitution),
            intelligence: FieldValue::from_field(character, Field::Intelligence),
            wisdom: FieldValue::from_field(character, Field::Wisdom),
            charisma: FieldValue::from_field(character, Field::Charisma),
            cmb: FieldValue::from_field(character, Field::CMB),
            cmd: FieldValue::from_field(character, Field::CMD),
            ac: FieldValue::from_field(character, Field::ArmorClass),
            bab: FieldValue::from_field(character, Field::BaseAttackBonus),
            hp: FieldValue::from_field(character, Field::HitPoints),
            initiative: FieldValue::from_field(character, Field::Initiative),
            save_fortitude: FieldValue::from_field(character, Field::SaveFortitude),
            save_reflex: FieldValue::from_field(character, Field::SaveReflex),
            save_will: FieldValue::from_field(character, Field::SaveWill),
            athletics: FieldValue::from_field(character, Field::Athletics),
            mobility: FieldValue::from_field(character, Field::Mobility),
            thievery: FieldValue::from_field(character, Field::Thievery),
            stealth: FieldValue::from_field(character, Field::Stealth),
            arcana: FieldValue::from_field(character, Field::KnowledgeArcana),
            world: FieldValue::from_field(character, Field::KnowledgeWorld),
            nature: FieldValue::from_field(character, Field::LoreNature),
            religion: FieldValue::from_field(character, Field::LoreReligion),
            perception: FieldValue::from_field(character, Field::Perception),
            persuasion: FieldValue::from_field(character, Field::Persuasion),
            magic_device: FieldValue::from_field(character, Field::UseMagicDevice),
            alignment: AlignmentWidget::new(character.alignment.clone(), false),
        }
    }

    pub fn view(&self) -> Element<Message> {
        let main_stats = row(vec![])
            .width(Length::Fill)
            .align_items(Alignment::Center)
            .push(self.mythic_experience.view())
            .push(self.experience.view());

        let abilities_stats = column(vec![])
            .width(Length::FillPortion(1))
            .push(self.strength.view())
            .push(self.dexterity.view())
            .push(self.constitution.view())
            .push(self.intelligence.view())
            .push(self.wisdom.view())
            .push(self.charisma.view());

        let combat_stats = column(vec![])
            .width(Length::FillPortion(1))
            .push(self.cmb.view())
            .push(self.cmd.view())
            .push(self.ac.view())
            .push(self.bab.view())
            .push(self.hp.view())
            .push(self.initiative.view())
            .push(self.save_fortitude.view())
            .push(self.save_reflex.view())
            .push(self.save_will.view());

        let skills_stats = column(vec![])
            .width(Length::FillPortion(1))
            .push(self.athletics.view())
            .push(self.mobility.view())
            .push(self.thievery.view())
            .push(self.stealth.view())
            .push(self.arcana.view())
            .push(self.world.view())
            .push(self.nature.view())
            .push(self.religion.view())
            .push(self.perception.view())
            .push(self.persuasion.view())
            .push(self.magic_device.view());

        let statistics = row(vec![])
            .spacing(25)
            .push(abilities_stats)
            .push(combat_stats)
            .push(skills_stats);

        container(
            column(vec![])
                .width(Length::Fill)
                .height(Length::Fill)
                .padding(10)
                .push(main_stats)
                .push(statistics)
                .push(
                    self.alignment
                        .view()
                        .map(|m| Message(Msg::AlignmentWheel(m))),
                )
                .push(iced::widget::Space::new(Length::Fill, Length::Fill)),
        )
        .style(theme::Container::MainPane)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message(Msg::StatisticModified { field, value }) => {
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

    fn stat_view_for_field(&mut self, field: &Field) -> &mut FieldValue {
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
