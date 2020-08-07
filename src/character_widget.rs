use crate::data::Character;
use crate::json::{Id, JsonPatch};
use crate::labelled_input_number::LabelledInputNumber;
use iced::{Align, Column, Command, Container, Element, Length, Row, Text};

#[derive(Debug, Clone)]
pub struct Message(Msg);

#[derive(Debug, Clone)]
enum Msg {
    StatisticModified { entity_id: Field, value: String },
}

impl Msg {
    fn statistic_modified(field: Field, value: String) -> Message {
        Message(Msg::StatisticModified {
            entity_id: field,
            value,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Field {
    // Abilities
    Strength,
    Dexterity,
    Constitution,
    Intelligence,
    Wisdom,
    Charisma,
    // Combat stats
    AttackBonus,
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
    // Money & Experience should also goes here
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
            Field::AttackBonus => write!(f, "Attack Bonus"),
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
    fn build_view(self, character: &Character) -> LabelledInputNumber<Field, u64> {
        let stat_key = match self {
            Field::Strength => Some("Strength"),
            Field::Dexterity => Some("Dexterity"),
            Field::Constitution => Some("Constitution"),
            Field::Intelligence => Some("Intelligence"),
            Field::Wisdom => Some("Wisdom"),
            Field::Charisma => Some("Charisma"),
            Field::AttackBonus => Some("AdditionalAttackBonus"),
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
        };

        let (id, ptr, value) = match stat_key {
            Some(key) => {
                let stat = character.find_stat(key).unwrap();

                let id = stat.id.clone();
                let ptr = "/m_BaseValue".into();

                (id, ptr, stat.base_value)
            }
            None => match self {
                Field::Experience => (
                    character.id.clone(),
                    "/Descriptor/Progression/Experience".into(),
                    character.experience,
                ),
                Field::MythicExperience => (
                    character.id.clone(),
                    "/Descriptor/Progression/MythicExperience".into(),
                    character.mythic_experience,
                ),
                _ => panic!(
                    "A field ({:?}) was not matched when building its view, please report",
                    self
                ),
            },
        };

        LabelledInputNumber::new(self, value, id, ptr)
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
    strength: LabelledInputNumber<Field, u64>,
    dexterity: LabelledInputNumber<Field, u64>,
    constitution: LabelledInputNumber<Field, u64>,
    intelligence: LabelledInputNumber<Field, u64>,
    wisdom: LabelledInputNumber<Field, u64>,
    charisma: LabelledInputNumber<Field, u64>,
    // Combat stats
    attack_bonus: LabelledInputNumber<Field, u64>,
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
    // Money & Experience should also goes here
    experience: LabelledInputNumber<Field, u64>,
    mythic_experience: LabelledInputNumber<Field, u64>,
}

impl CharacterWidget {
    pub fn new(character: &Character) -> CharacterWidget {
        CharacterWidget {
            id: character.id.clone(),
            experience: Field::Experience.build_view(character),
            mythic_experience: Field::MythicExperience.build_view(character),
            strength: Field::Strength.build_view(character),
            dexterity: Field::Dexterity.build_view(character),
            constitution: Field::Constitution.build_view(character),
            intelligence: Field::Intelligence.build_view(character),
            wisdom: Field::Wisdom.build_view(character),
            charisma: Field::Charisma.build_view(character),
            attack_bonus: Field::AttackBonus.build_view(character),
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
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let main_stats = Row::new()
            .width(Length::Fill)
            .height(Length::from(50))
            .align_items(Align::Center)
            // Money is actually part of the player.json and not party.json.
            .push(Text::new("Money: 38747G").width(Length::FillPortion(1)))
            .push(self.experience.view(Msg::statistic_modified))
            .push(self.mythic_experience.view(Msg::statistic_modified));

        let abilities_stats = Column::new()
            .height(Length::Fill)
            .width(Length::FillPortion(1))
            .push(self.strength.view(Msg::statistic_modified))
            .push(self.dexterity.view(Msg::statistic_modified))
            .push(self.constitution.view(Msg::statistic_modified))
            .push(self.intelligence.view(Msg::statistic_modified))
            .push(self.wisdom.view(Msg::statistic_modified))
            .push(self.charisma.view(Msg::statistic_modified));

        let combat_stats = Column::new()
            .width(Length::FillPortion(1))
            .push(self.attack_bonus.view(Msg::statistic_modified))
            .push(self.cmb.view(Msg::statistic_modified))
            .push(self.cmd.view(Msg::statistic_modified))
            .push(self.ac.view(Msg::statistic_modified))
            .push(self.bab.view(Msg::statistic_modified))
            .push(self.hp.view(Msg::statistic_modified))
            .push(self.initiative.view(Msg::statistic_modified))
            .push(self.save_fortitude.view(Msg::statistic_modified))
            .push(self.save_reflex.view(Msg::statistic_modified))
            .push(self.save_will.view(Msg::statistic_modified));

        let skills_stats = Column::new()
            .width(Length::FillPortion(1))
            .push(self.athletics.view(Msg::statistic_modified))
            .push(self.mobility.view(Msg::statistic_modified))
            .push(self.thievery.view(Msg::statistic_modified))
            .push(self.stealth.view(Msg::statistic_modified))
            .push(self.arcana.view(Msg::statistic_modified))
            .push(self.world.view(Msg::statistic_modified))
            .push(self.nature.view(Msg::statistic_modified))
            .push(self.religion.view(Msg::statistic_modified))
            .push(self.perception.view(Msg::statistic_modified))
            .push(self.persuasion.view(Msg::statistic_modified))
            .push(self.magic_device.view(Msg::statistic_modified));

        let statistics = Row::new()
            .spacing(25)
            .push(abilities_stats)
            .push(combat_stats)
            .push(skills_stats);

        Container::new(
            Column::new()
                .width(Length::Fill)
                .padding(10)
                .push(main_stats)
                .push(statistics),
        )
        .style(crate::styles::MainPane)
        .into()
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message(Msg::StatisticModified { entity_id, value }) => {
                if let Ok(n) = value.parse::<u64>() {
                    self.stat_view_for_field(&entity_id).value = n;
                }
            }
        };
        Command::none()
    }

    pub fn patches(&self) -> Vec<JsonPatch> {
        let mut patches = Vec::with_capacity(28);

        patches.push(self.strength.change());
        patches.push(self.dexterity.change());
        patches.push(self.constitution.change());
        patches.push(self.intelligence.change());
        patches.push(self.wisdom.change());
        patches.push(self.charisma.change());
        patches.push(self.attack_bonus.change());
        patches.push(self.cmb.change());
        patches.push(self.cmd.change());
        patches.push(self.ac.change());
        patches.push(self.bab.change());
        patches.push(self.hp.change());
        patches.push(self.initiative.change());
        patches.push(self.save_fortitude.change());
        patches.push(self.save_reflex.change());
        patches.push(self.save_will.change());
        patches.push(self.athletics.change());
        patches.push(self.mobility.change());
        patches.push(self.thievery.change());
        patches.push(self.stealth.change());
        patches.push(self.arcana.change());
        patches.push(self.world.change());
        patches.push(self.nature.change());
        patches.push(self.religion.change());
        patches.push(self.perception.change());
        patches.push(self.persuasion.change());
        patches.push(self.magic_device.change());
        patches.push(self.experience.change());
        patches.push(self.mythic_experience.change());

        patches
    }

    fn stat_view_for_field(&mut self, field: &Field) -> &mut LabelledInputNumber<Field, u64> {
        match field {
            Field::Strength => &mut self.strength,
            Field::Dexterity => &mut self.dexterity,
            Field::Constitution => &mut self.constitution,
            Field::Intelligence => &mut self.intelligence,
            Field::Wisdom => &mut self.wisdom,
            Field::Charisma => &mut self.charisma,
            Field::AttackBonus => &mut self.attack_bonus,
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
