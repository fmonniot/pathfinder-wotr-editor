use super::input::labelled_input_number;
use crate::data::{Army, KingdomResources, Player, Squad};
use crate::json::{Id, JsonPatch};
use crate::styles;
use iced::{
    pure::{column, container, row, text, widget::Row, widget::Space, Element},
    Command, Length,
};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub struct Message(Msg);

// The clippy warning would be useful if the enum itself would
// have been named Update something. As it stands, the overall
// Msg could in theory be something else than a field update.
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone)]
enum Msg {
    FieldUpdate(Field, u64),
    ArmyMovementPointsUpdate(Id, f64), // (army id, new value)
    ArmySquadUpdate(Id, Id, u64),      // (army id, squad id, new value)
}

#[derive(Debug, Clone)]
enum Field {
    Money,
    Resources(KingdomResourcesField),
    ResourcesPerTurn(KingdomResourcesField),
}

#[derive(Debug, Clone, Copy)]
enum KingdomResourcesField {
    Finances,
    Materials,
    Favors,
}

impl Display for KingdomResourcesField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KingdomResourcesField::Finances => write!(f, "Finances"),
            KingdomResourcesField::Materials => write!(f, "Materials"),
            KingdomResourcesField::Favors => write!(f, "Divine Favors"),
        }
    }
}

pub struct PlayerWidget {
    player_id: Id,
    money: u64,
    resources: Option<KingdomResourcesState>,
    resources_per_turn: Option<KingdomResourcesState>,
    armies: Vec<ArmyState>,
}

impl PlayerWidget {
    pub fn new(player: &Player) -> PlayerWidget {
        let armies = player.armies.iter().map(ArmyState::from).collect();

        PlayerWidget {
            player_id: player.id.clone(),
            money: player.money,
            resources: player
                .kingdom
                .as_ref()
                .map(|k| KingdomResourcesState::from(&k.resources)),
            resources_per_turn: player
                .kingdom
                .as_ref()
                .map(|k| KingdomResourcesState::from(&k.resources_per_turn)),
            armies,
        }
    }

    // TODO We are missing recruits panels
    // TODO We might need a scrollable widget to account for many army blocks
    pub fn view(&mut self) -> Element<Message> {
        let money = iced_lazy::pure::component(labelled_input_number(
            "Money",
            self.money,
            move |new_value| Message(Msg::FieldUpdate(Field::Money, new_value)),
        ));

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
        let layout = column()
            .push(money)
            .push(text("Resources"))
            //.push(PureRow::with_children(resources))
            .push(text("Armies"))
            .push(armies);

        container(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(styles::MainPane)
            .into()
    }

    fn update_resource(
        resources: &mut Option<KingdomResourcesState>,
        field: &KingdomResourcesField,
        value: u64,
    ) {
        if let Some(ref mut resources) = resources.as_mut() {
            match field {
                KingdomResourcesField::Finances => resources.finances = value,
                KingdomResourcesField::Materials => resources.materials = value,
                KingdomResourcesField::Favors => resources.favors = value,
            };
        }
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message(Msg::FieldUpdate(field, value)) => {
                match &field {
                    Field::Money => self.money = value,
                    Field::Resources(res) => {
                        PlayerWidget::update_resource(&mut self.resources, res, value)
                    }
                    Field::ResourcesPerTurn(res) => {
                        PlayerWidget::update_resource(&mut self.resources_per_turn, res, value)
                    }
                };
            }
            Message(Msg::ArmyMovementPointsUpdate(army_id, new_value)) => {
                if let Some(army) = self.find_army_state(army_id) {
                    army.movement_points = new_value;
                };
            }
            Message(Msg::ArmySquadUpdate(army_id, squad_id, new_value)) => {
                if let Some(army) = self.find_army_state(army_id) {
                    army.update_squad(squad_id, new_value);
                };
            }
        }
        Command::none()
    }

    fn find_army_state(&mut self, army_id: Id) -> Option<&mut ArmyState> {
        self.armies.iter_mut().find(|a| a.army_id == army_id)
    }

    pub fn patches(&self) -> Vec<JsonPatch> {
        let mut patches = vec![JsonPatch::id_at_pointer(
            self.player_id.clone(),
            "/Money".into(),
            serde_json::to_value(self.money).unwrap(),
        )];

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
fn two_columns_layout<'a, Msg: 'a, I>(children: I) -> iced::pure::Element<'a, Msg>
where
    I: Iterator<Item = iced::pure::Element<'a, Msg>>,
{
    // TODO Make some optimization in the layout used based on the iterator size
    // e.g. if one/two elements, remove the outer Column
    // let (lower_bound, upper_bound) = children.size_hint(); // how many elements we have to layout

    let mut columns = column();

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

struct KingdomResourcesState {
    resources_id: Id,
    finances: u64,
    materials: u64,
    favors: u64,
}

impl KingdomResourcesState {
    fn from(resources: &KingdomResources) -> KingdomResourcesState {
        KingdomResourcesState {
            resources_id: resources.id.clone(),
            finances: resources.finances,
            materials: resources.materials,
            favors: resources.favors,
        }
    }

    fn patches(&self) -> Vec<JsonPatch> {
        let base = |pointer: &str, value| {
            JsonPatch::id_at_pointer(
                self.resources_id.clone(),
                pointer.into(),
                serde_json::to_value(value).unwrap(),
            )
        };

        let patches = vec![
            base("m_Finances", self.finances),
            base("m_Materials", self.finances),
            base("m_Favors", self.finances),
        ];

        patches
    }

    fn view<F>(&self, title: impl Into<String>, build_field: F) -> iced::pure::Element<Message>
    where
        F: 'static + Clone + Fn(KingdomResourcesField) -> Field, // TODO is 'static and Clone still required ?
    {
        let view = move |field: KingdomResourcesField, value| {
            let build_field = build_field.clone();

            iced_lazy::pure::component(labelled_input_number(
                field.to_string(),
                value,
                move |new_value| Message(Msg::FieldUpdate(build_field(field), new_value)),
            ))
        };

        let layout = column()
            .push(text(title))
            .push(view(KingdomResourcesField::Finances, self.finances))
            .push(view(KingdomResourcesField::Materials, self.materials))
            .push(view(KingdomResourcesField::Favors, self.favors));

        container(layout)
            .width(Length::Fill)
            .style(styles::MainPane)
            .into()
    }
}

struct ArmyState {
    army_id: Id,
    movement_points: f64,
    /// A squad is composed of the unit id (a reference to the type
    /// of id) and a game id (a reference to the unique squad within
    /// the runtime). It is associated with the number of unit within
    /// the squad.
    ///
    /// TODO Should I introduce a SquadId newtype instead of using a
    /// string? It would at least make it clearer that
    /// [Squad::id_to_name()] needs to be used to get a human readable
    /// name (or rather, newtype could implement the correct [Display]
    /// for it)
    squads: Vec<Squad>,
}

impl ArmyState {
    fn from(army: &Army) -> ArmyState {
        let army_id = army.id.clone();
        let squads = army.squads.to_vec();
        let movement_points = army.movement_points;

        ArmyState {
            army_id,
            movement_points,
            squads,
        }
    }

    fn view(&self) -> iced::pure::Element<Message> {
        // I don't know of a cleaner pattern to share the same immutable variable with multiple closures.
        // I feel there should be a simpler pattern than having multiple named variable but that will have
        // to do for now.
        let army_id = self.army_id.clone();
        let common = row().push(iced_lazy::pure::component(labelled_input_number(
            "Movement Points",
            self.movement_points,
            move |v| Message(Msg::ArmyMovementPointsUpdate(army_id.clone(), v)),
        )));

        let mut layout = column().push(common);

        for Squad { id, unit, count } in self.squads.iter() {
            let army_id = self.army_id.clone();
            let squad_id = id.clone();

            let label = match Squad::id_to_name(unit) {
                Some(s) => s,
                None => unit,
            };

            layout = layout.push(iced_lazy::pure::component(labelled_input_number(
                label,
                *count,
                move |v| Message(Msg::ArmySquadUpdate(army_id.clone(), squad_id.clone(), v)),
            )));
        }
        let inner = container(layout).padding(5).style(ArmyWidgetContainerStyle);

        // Outer container, simulating a margin on inner
        container(inner).padding(10).width(Length::Fill).into()
    }

    fn update_squad(&mut self, new_squad_id: Id, new_squad_count: u64) {
        for mut squad in &mut self.squads {
            // Looking at the id is enough, as they are unique per
            // game instance.
            if squad.id == new_squad_id {
                squad.count = new_squad_count
            }
        }
    }

    fn patches(&self) -> Vec<JsonPatch> {
        let mut patches: Vec<_> = self
            .squads
            .iter()
            .map(|s| {
                JsonPatch::id_at_pointer(
                    s.id.clone(),
                    "Count".into(),
                    serde_json::to_value(s.count).unwrap(),
                )
            })
            .collect();

        patches.push(JsonPatch::id_at_pointer(
            self.army_id.clone(),
            "MovementPoints".into(),
            serde_json::to_value(self.movement_points).unwrap(),
        ));

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
