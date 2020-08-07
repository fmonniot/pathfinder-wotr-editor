use crate::data::{KingdomResources, Player};
use crate::json::JsonPatch;
use crate::labelled_input_number::LabelledInputNumber;
use crate::styles;
use iced::{Column, Command, Container, Element, Length, Row, Text};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Message(Msg);

#[derive(Debug, Clone)]
enum Msg {
    FieldUpdate(Field, String),
}

#[derive(Debug, Clone)]
enum Field {
    Money,
    Resources(KingdomResourcesField),
    ResourcesPerTurn(KingdomResourcesField),
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::Money => write!(f, "Money"),
            Field::Resources(res) => write!(f, "{}", res),
            Field::ResourcesPerTurn(res) => write!(f, "{}", res),
        }
    }
}

#[derive(Debug, Clone)]
enum KingdomResourcesField {
    Finances,
    Basics,
    Favors,
    Mana,
}

impl Display for KingdomResourcesField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KingdomResourcesField::Finances => write!(f, "Finances"),
            KingdomResourcesField::Basics => write!(f, "Basics"),
            KingdomResourcesField::Favors => write!(f, "Divine Favors"),
            KingdomResourcesField::Mana => write!(f, "Mana"),
        }
    }
}

pub struct PlayerWidget {
    money: LabelledInputNumber<Field>,
    resources: KingdomResourcesWidget,
    resources_per_turn: KingdomResourcesWidget,
}

impl PlayerWidget {
    pub fn new(player: &Player) -> PlayerWidget {
        PlayerWidget {
            money: LabelledInputNumber::new(
                Field::Money,
                player.money,
                player.id.clone(),
                "/Money".into(),
            ),
            resources: KingdomResourcesWidget::new(&player.resources),
            resources_per_turn: KingdomResourcesWidget::new(&player.resources_per_turn),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let resources = Row::with_children(vec![
            self.resources.view("Resources", Field::Resources),
            self.resources_per_turn
                .view("Resources/turn", Field::ResourcesPerTurn),
        ]);

        let layout = Column::new()
            .push(Text::new("Crusade pane tbd"))
            .push(
                self.money
                    .view(|id, value| Message(Msg::FieldUpdate(id, value))),
            )
            .push(resources);

        Container::new(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::MainPane)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message(Msg::FieldUpdate(field, value)) => {
                if let Ok(value) = value.parse::<u64>() {
                    match field {
                        Field::Money => self.money.value = value,
                        Field::Resources(res) => match res {
                            KingdomResourcesField::Finances => {
                                self.resources.finances.value = value
                            }
                            KingdomResourcesField::Basics => self.resources.basics.value = value,
                            KingdomResourcesField::Favors => self.resources.favors.value = value,
                            KingdomResourcesField::Mana => self.resources.mana.value = value,
                        },
                        Field::ResourcesPerTurn(res) => match res {
                            KingdomResourcesField::Finances => {
                                self.resources_per_turn.finances.value = value
                            }
                            KingdomResourcesField::Basics => self.resources_per_turn.basics.value = value,
                            KingdomResourcesField::Favors => self.resources_per_turn.favors.value = value,
                            KingdomResourcesField::Mana => self.resources_per_turn.mana.value = value,
                        },
                    }
                }
            }
        }
        Command::none()
    }

    pub fn patches(&self) -> Vec<JsonPatch> {
        let mut patches = vec![];

        patches.push(self.money.change());
        patches.append(&mut self.resources.patches());
        patches.append(&mut self.resources_per_turn.patches());

        patches
    }
}

struct KingdomResourcesWidget {
    finances: LabelledInputNumber<KingdomResourcesField>,
    basics: LabelledInputNumber<KingdomResourcesField>,
    favors: LabelledInputNumber<KingdomResourcesField>,
    mana: LabelledInputNumber<KingdomResourcesField>,
}

impl KingdomResourcesWidget {
    fn new(resources: &KingdomResources) -> KingdomResourcesWidget {
        KingdomResourcesWidget {
            finances: LabelledInputNumber::new(
                KingdomResourcesField::Finances,
                resources.finances,
                resources.id.clone(),
                "m_Finances".into(),
            ),
            basics: LabelledInputNumber::new(
                KingdomResourcesField::Basics,
                resources.basics,
                resources.id.clone(),
                "m_Basics".into(),
            ),
            favors: LabelledInputNumber::new(
                KingdomResourcesField::Favors,
                resources.favors,
                resources.id.clone(),
                "m_Favors".into(),
            ),
            mana: LabelledInputNumber::new(
                KingdomResourcesField::Mana,
                resources.mana,
                resources.id.clone(),
                "m_Mana".into(),
            ),
        }
    }

    fn view<F>(&mut self, title: &str, f: F) -> Element<Message>
    where
        F: 'static + Clone + Fn(KingdomResourcesField) -> Field,
    {
        let update = move |id, value| Message(Msg::FieldUpdate(f(id), value));

        let layout = Column::new()
            .push(Text::new(title))
            .push(self.finances.view(update.clone()))
            .push(self.basics.view(update.clone()))
            .push(self.favors.view(update.clone()))
            .push(self.mana.view(update));

        Container::new(layout)
            .width(Length::Fill)
            .style(styles::MainPane)
            .into()
    }

    fn patches(&self) -> Vec<JsonPatch> {
        vec![
            self.finances.change(),
            self.basics.change(),
            self.favors.change(),
            self.mana.change(),
        ]
    }
}
