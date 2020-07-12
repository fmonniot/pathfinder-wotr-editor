// TODO Rename this module to character_widget (and its main type to CharacterWidget)

use iced::{text_input, Align, Column, Command, Element, Length, Row, Text, TextInput};
use crate::data::{Character, Stat};

#[derive(Debug, Clone)]
pub struct Msg(Message);

#[derive(Debug, Clone)]
enum Message {
    StatisticModified {
        entity_id: Field,
        value: String, // TODO Add a way to find out which stat has been modified
    },
}

#[derive(Debug, Clone)]
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

impl Field {
    fn label(&self) -> &'static str {
        match self {
            Field::Strength => "Strength",
            Field::Dexterity => "Dexterity",
            Field::Constitution => "Constitution",
            Field::Intelligence => "Intelligence",
            Field::Wisdom => "Wisdom",
            Field::Charisma => "Charisma",
            Field::AttackBonus => "Attack Bonus",
            Field::CMB => "CMB",
            Field::CMD => "CMD",
            Field::ArmorClass => "Armor Class",
            Field::BaseAttackBonus => "Base Attack Bonus",
            Field::HitPoints => "Hit Points",
            Field::Initiative => "Initiative",
            Field::SaveFortitude => "Save: Fortitude",
            Field::SaveReflex => "Save: Reflex",
            Field::SaveWill => "Save: Will",
            Field::Athletics => "Athletics",
            Field::Mobility => "Mobility",
            Field::Thievery => "Thievery",
            Field::Stealth => "Stealth",
            Field::KnowledgeArcana => "Knowledge: Arcana",
            Field::KnowledgeWorld => "Knowledge: World",
            Field::LoreNature => "Lore: Nature",
            Field::LoreReligion => "Lore: Religion",
            Field::Perception => "Perception",
            Field::Persuasion => "Persuasion",
            Field::UseMagicDevice => "Use Magic Device",
            Field::Experience => "",
            Field::MythicExperience => "",
        }
    }

    fn build_view(self, character: &Character) -> StatView {
        let key = match self {
            Field::Strength => "Strength",
            Field::Dexterity => "Dexterity",
            Field::Constitution => "Constitution",
            Field::Intelligence => "Intelligence",
            Field::Wisdom => "Wisdom",
            Field::Charisma => "Charisma",
            Field::AttackBonus => "AdditionalAttackBonus",
            Field::CMB => "AdditionalCMB",
            Field::CMD => "AdditionalCMD",
            Field::ArmorClass => "AC",
            Field::BaseAttackBonus => "BaseAttackBonus",
            Field::HitPoints => "HitPoints",
            Field::Initiative => "Initiative",
            Field::SaveFortitude => "SaveFortitude",
            Field::SaveReflex => "SaveReflex",
            Field::SaveWill => "SaveWill",
            Field::Athletics => "SkillAthletics",
            Field::Mobility => "SkillMobility",
            Field::Thievery => "SkillThievery",
            Field::Stealth => "SkillStealth",
            Field::KnowledgeArcana => "SkillKnowledgeArcana",
            Field::KnowledgeWorld => "SkillKnowledgeWorld",
            Field::LoreNature => "SkillLoreNature",
            Field::LoreReligion => "SkillLoreReligion",
            Field::Perception => "SkillPerception",
            Field::Persuasion => "SkillPersuasion",
            Field::UseMagicDevice => "SkillUseMagicDevice",
            Field::Experience => "",
            Field::MythicExperience => "",
        };

        let stat = character.find_stat(key).unwrap();

        StatView::new(self, stat)
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
pub struct CharacterView {
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
    experience: StatView,
    mythic_experience: StatView,
}

impl CharacterView {
    pub fn new(character: &Character) -> CharacterView {
        CharacterView {
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

    pub fn view(&mut self) -> Element<Msg> {
        let main_stats = Row::new()
            .width(Length::Fill)
            .height(Length::from(50))
            .align_items(Align::Center)
            // Money is actually part of the player.json and not party.json.
            .push(Text::new("Money: 38747G").width(Length::FillPortion(1)))
            .push(self.experience.view())
            .push(self.mythic_experience.view());

        let abilities_stats = Column::new()
            .height(Length::Fill)
            .width(Length::FillPortion(1))
            .push(self.strength.view())
            .push(self.dexterity.view())
            .push(self.constitution.view())
            .push(self.intelligence.view())
            .push(self.wisdom.view())
            .push(self.charisma.view());

        let combat_stats = Column::new()
            .width(Length::FillPortion(1))
            .push(self.attack_bonus.view())
            .push(self.cmb.view())
            .push(self.cmd.view())
            .push(self.ac.view())
            .push(self.bab.view())
            .push(self.hp.view())
            .push(self.initiative.view())
            .push(self.save_fortitude.view())
            .push(self.save_reflex.view())
            .push(self.save_will.view());

        let skills_stats = Column::new()
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

        let statistics = Row::new()
            .spacing(25)
            .push(abilities_stats)
            .push(combat_stats)
            .push(skills_stats);

        Column::new()
            .width(Length::Fill)
            .padding(10)
            .push(main_stats)
            .push(statistics)
            .into()
    }

    pub fn update(&mut self, message: Msg) -> Command<Msg> {
        match message {
            Msg(Message::StatisticModified { .. }) => {

                ()
            }
        };
        Command::none()
    }
}

struct StatView {
    id: Field,
    text_input: text_input::State,
    value: u64,
}

impl StatView {

    fn new(field: Field, stat: &Stat) -> StatView {
        StatView {
            id: field,
            text_input: text_input::State::new(),
            value: stat.base_value,
        }
    }

    fn view(&mut self) -> Element<Msg> {
        let entity_id = self.id.clone();
        let input = TextInput::new(
            &mut self.text_input,
            self.id.label(),
            &self.value.to_string(),
            move |value| {
                // Not sure why just moving the view's entity_id is not enough, but given how
                // cheap a Field is I can leave with that clone.
                let entity_id = entity_id.clone();
                Msg(Message::StatisticModified { entity_id, value })
            },
        );

        Row::new()
            .width(Length::FillPortion(1))
            .push(Text::new(format!("{}: ", self.id.label())))
            .push(input)
            .into()
    }
}
