use crate::data::{Army, KingdomResources, Player, Squad};
use crate::json::{Id, JsonPatch};
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
    Army(Id, ArmyField),
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::Money => write!(f, "Money"),
            Field::Resources(res) => write!(f, "{}", res),
            Field::ResourcesPerTurn(res) => write!(f, "{}", res),
            Field::Army(_, res) => write!(f, "{}", res),
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
    money: LabelledInputNumber<Field, u64>,
    resources: KingdomResourcesWidget,
    resources_per_turn: KingdomResourcesWidget,
    armies: Vec<ArmyWidget>,
}

impl PlayerWidget {
    pub fn new(player: &Player) -> PlayerWidget {
        let armies = player.armies.iter().map(ArmyWidget::new).collect();
        PlayerWidget {
            money: LabelledInputNumber::new(
                Field::Money,
                player.money,
                player.id.clone(),
                "/Money".into(),
            ),
            resources: KingdomResourcesWidget::new(&player.resources),
            resources_per_turn: KingdomResourcesWidget::new(&player.resources_per_turn),
            armies,
        }
    }

    // TODO We are missing recruits and armies panels
    pub fn view(&mut self) -> Element<Message> {
        let resources = Row::with_children(vec![
            self.resources.view("Resources", Field::Resources),
            self.resources_per_turn
                .view("Resources/turn", Field::ResourcesPerTurn),
        ]);

        let armies = Row::with_children(self.armies.iter_mut().map(|a| a.view()).collect());

        // TODO Make a dedicated function for title (and separator) with nicer style
        let layout = Column::new()
            .push(
                self.money
                    .view(|id, value| Message(Msg::FieldUpdate(id, value))),
            )
            .push(Text::new("Resources"))
            .push(resources)
            .push(Text::new("Armies"))
            .push(armies);

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
                    match &field {
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
                            KingdomResourcesField::Basics => {
                                self.resources_per_turn.basics.value = value
                            }
                            KingdomResourcesField::Favors => {
                                self.resources_per_turn.favors.value = value
                            }
                            KingdomResourcesField::Mana => {
                                self.resources_per_turn.mana.value = value
                            }
                        },
                        _ => (),
                    }
                };

                if let Field::Army(id, res) = field {
                    if let Some(army) = self.armies.iter_mut().find(|a| a.id == id) {
                        army.update(res, value);
                    };
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
        patches.append(&mut self.armies.iter().flat_map(|a| a.patches()).collect());

        patches
    }
}

struct KingdomResourcesWidget {
    finances: LabelledInputNumber<KingdomResourcesField, u64>,
    basics: LabelledInputNumber<KingdomResourcesField, u64>,
    favors: LabelledInputNumber<KingdomResourcesField, u64>,
    mana: LabelledInputNumber<KingdomResourcesField, u64>,
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

#[derive(Debug, Clone, PartialEq)]
enum ArmyField {
    Experience,
    MovementPoints,
    Squad(String), // unit id
}

impl Display for ArmyField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArmyField::Experience => write!(f, "Experience"),
            ArmyField::MovementPoints => write!(f, "Movement Points"),
            ArmyField::Squad(unit) => match Squad::id_to_name(&unit) {
                Some(named) => write!(f, "{}", named),
                None => write!(f, "{}", unit),
            },
        }
    }
}

struct ArmyWidget {
    id: Id,
    experience: LabelledInputNumber<ArmyField, u64>,
    movement_points: LabelledInputNumber<ArmyField, f64>,
    squads: Vec<LabelledInputNumber<ArmyField, u64>>,
}

impl ArmyWidget {
    fn new(army: &Army) -> ArmyWidget {
        let squads = army
            .squads
            .iter()
            .map(|squad| {
                LabelledInputNumber::new(
                    ArmyField::Squad(squad.unit.clone()),
                    squad.count,
                    squad.id.clone(),
                    "count".into(),
                )
            })
            .collect();

        ArmyWidget {
            id: army.id.clone(),
            squads,
            experience: LabelledInputNumber::new(
                ArmyField::Experience,
                army.experience,
                army.id.clone(),
                "experience".into(),
            ),
            movement_points: LabelledInputNumber::new(
                ArmyField::MovementPoints,
                army.movement_points,
                army.id.clone(),
                "movement_points".into(),
            ),
        }
    }

    fn view(&mut self) -> Element<Message> {
        // I don't know of a cleaner pattern to share the same immutable variable with multiple closures.
        // I feel there should be a simpler pattern than having multiple named variable but that will have
        // to do for now.
        let id_for_xp = self.id.clone();
        let id_for_mp = self.id.clone();
        let common = Row::with_children(vec![
            // TODO Do we have the name of the army ?
            self.experience
                .view(move |d, v| Message(Msg::FieldUpdate(Field::Army(id_for_xp.clone(), d), v))),
            self.movement_points
                .view(move |d, v| Message(Msg::FieldUpdate(Field::Army(id_for_mp.clone(), d), v))),
        ]);

        let mut layout = Column::new().push(common);

        // Adding the members of the army to the layout
        // A "squad" is basically the number of a given unit type in the army
        for squad in &mut self.squads {
            let id_for_squads = self.id.clone();
            layout = layout.push(squad.view(move |d, v| {
                Message(Msg::FieldUpdate(Field::Army(id_for_squads.clone(), d), v))
            }));
        }

        let inner = Container::new(layout)
            .padding(5)
            .style(ArmyWidgetContainerStyle);

        // Outer container, simulating a margin on inner
        Container::new(inner).padding(10).width(Length::Fill).into()
    }

    fn update(&mut self, field: ArmyField, value: String) {
        match field {
            ArmyField::Experience => {
                if let Ok(value) = value.parse::<u64>() {
                    self.experience.value = value;
                }
            }
            ArmyField::MovementPoints => {
                if let Ok(value) = value.parse::<f64>() {
                    self.movement_points.value = value;
                }
            }
            ArmyField::Squad(squad_id) => {
                for mut squad in &mut self.squads {
                    if squad.discriminator == ArmyField::Squad(squad_id.clone()) {
                        if let Ok(value) = value.parse::<u64>() {
                            squad.value = value;
                        }
                    }
                }
            }
        }
    }

    fn patches(&self) -> Vec<JsonPatch> {
        let mut patches = vec![self.experience.change(), self.movement_points.change()];

        patches.append(&mut self.squads.iter().map(|s| s.change()).collect());

        patches
    }
}

struct ArmyWidgetContainerStyle;

use iced::{container, Color};

impl container::StyleSheet for ArmyWidgetContainerStyle {
    fn style(&self) -> container::Style {
        container::Style {
            border_color: Color::WHITE,
            border_width: 1,
            ..container::Style::default()
        }
    }
}
