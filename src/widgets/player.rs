use super::LabelledInputNumber;
use crate::data::{Army, KingdomResources, Player, Squad};
use crate::json::{Id, JsonPatch};
use crate::styles;
use iced::{Column, Command, Container, Element, Length, Row, Space, Text};
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
    Materials,
    Favors,
    Mana,
}

impl Display for KingdomResourcesField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KingdomResourcesField::Finances => write!(f, "Finances"),
            KingdomResourcesField::Materials => write!(f, "Materials"),
            KingdomResourcesField::Favors => write!(f, "Divine Favors"),
            KingdomResourcesField::Mana => write!(f, "Mana"),
        }
    }
}

pub struct PlayerWidget {
    money: LabelledInputNumber<Field, u64>,
    resources: Option<KingdomResourcesWidget>,
    resources_per_turn: Option<KingdomResourcesWidget>,
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
            resources: player
                .kingdom
                .as_ref()
                .map(|k| KingdomResourcesWidget::new(&k.resources)),
            resources_per_turn: player
                .kingdom
                .as_ref()
                .map(|k| KingdomResourcesWidget::new(&k.resources_per_turn)),
            armies,
        }
    }

    // TODO We are missing recruits panels
    // TODO We might need a scrollable widget to account for many army blocks
    pub fn view(&mut self) -> Element<Message> {
        let mut resources = vec![];
        match &mut self.resources {
            Some(res) => {
                let v = res.view("Resources", Field::Resources);
                resources.push(v);
            }
            None => (),
        };
        match &mut self.resources_per_turn {
            Some(res) => {
                let v = res.view("Resources/turn", Field::ResourcesPerTurn);
                resources.push(v);
            }
            None => (),
        };

        let armies = two_columns_layout(self.armies.iter_mut().map(|a| a.view()));

        // TODO Make a dedicated function for title (and separator) with nicer style
        let layout = Column::new()
            .push(
                self.money
                    .view(|id, value| Message(Msg::FieldUpdate(id, value))),
            )
            .push(Text::new("Resources"))
            .push(Row::with_children(resources))
            .push(Text::new("Armies"))
            .push(armies);

        Container::new(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::MainPane)
            .into()
    }

    fn update_resource(
        resources: &mut Option<KingdomResourcesWidget>,
        field: &KingdomResourcesField,
        value: u64,
    ) {
        if let Some(ref mut resources) = resources.as_mut() {
            match field {
                KingdomResourcesField::Finances => resources.finances.value = value,
                KingdomResourcesField::Materials => resources.materials.value = value,
                KingdomResourcesField::Favors => resources.favors.value = value,
                KingdomResourcesField::Mana => resources.mana.value = value,
            };
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message(Msg::FieldUpdate(field, value)) => {
                if let Ok(value) = value.parse::<u64>() {
                    match &field {
                        Field::Money => self.money.value = value,
                        Field::Resources(res) => {
                            PlayerWidget::update_resource(&mut self.resources, res, value)
                        }
                        Field::ResourcesPerTurn(res) => {
                            PlayerWidget::update_resource(&mut self.resources_per_turn, res, value)
                        }
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
        if let Some(res) = self.resources.as_ref() {
            patches.append(&mut res.patches());
        };
        if let Some(res) = self.resources_per_turn.as_ref() {
            patches.append(&mut res.patches());
        };
        patches.append(&mut self.armies.iter().flat_map(|a| a.patches()).collect());

        patches
    }
}

/// Organize `children` elements in a column containing two elements
/// per row. On the event that children is odd, the latest element will
/// take half the space of the layout, not the entire width.
// Should this be moved to a module for generic layouts ?
fn two_columns_layout<'a, Msg: 'a, I>(children: I) -> Element<'a, Msg>
where
    I: Iterator<Item = Element<'a, Msg>>,
{
    // TODO Make some optimization in the layout used based on the iterator size
    // e.g. if one/two elements, remove the outer Column
    // let (lower_bound, upper_bound) = children.size_hint(); // how many elements we have to layout

    let mut columns = Column::new();

    let mut c = children;
    loop {
        let first = c.next();
        let second = c.next();

        match first {
            None => break,
            Some(el1) => {
                let el2 =
                    second.unwrap_or_else(|| Space::new(Length::Fill, Length::from(1)).into());
                columns = columns.push(Row::with_children(vec![el1, el2]))
            }
        }
    }

    columns.into()
}

struct KingdomResourcesWidget {
    finances: LabelledInputNumber<KingdomResourcesField, u64>,
    materials: LabelledInputNumber<KingdomResourcesField, u64>,
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
            materials: LabelledInputNumber::new(
                KingdomResourcesField::Materials,
                resources.materials,
                resources.id.clone(),
                "m_Materials".into(),
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
            .push(self.materials.view(update.clone()))
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
            self.materials.change(),
            self.favors.change(),
            self.mana.change(),
        ]
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ArmyField {
    MovementPoints,
    Squad(String), // unit id
}

impl Display for ArmyField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
                    "Count".into(),
                )
            })
            .collect();

        ArmyWidget {
            id: army.id.clone(),
            squads,
            movement_points: LabelledInputNumber::new(
                ArmyField::MovementPoints,
                army.movement_points,
                army.id.clone(),
                "MovementPoints".into(),
            ),
        }
    }

    fn view(&mut self) -> Element<Message> {
        // I don't know of a cleaner pattern to share the same immutable variable with multiple closures.
        // I feel there should be a simpler pattern than having multiple named variable but that will have
        // to do for now.
        let id_for_mp = self.id.clone();
        let common = Row::with_children(vec![
            // We currently don't have an army name, so we should do like the game and use their order of
            // apparition to number them. And monitor the game if that changes with further releases.
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
        let mut patches = vec![self.movement_points.change()];

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
            border_width: 1.,
            ..container::Style::default()
        }
    }
}
