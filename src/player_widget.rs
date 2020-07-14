use crate::data::Player;
use crate::editor_widget::style;
use iced::{text_input, Column, Command, Container, Element, Length, Row, Text, TextInput};
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
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::Money => write!(f, "Money"),
        }
    }
}

pub struct PlayerWidget {
    money: LabelledInputText<Field>,
}

impl PlayerWidget {
    pub fn new(player: &Player) -> PlayerWidget {
        PlayerWidget {
            money: LabelledInputText::new(Field::Money, player.money),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let layout = Column::new().push(Text::new("Crusade pane tbd")).push(
            self.money
                .view(|id, value| Message(Msg::FieldUpdate(id, value))),
        );

        Container::new(layout)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::MainPane)
            .into()
    }

    pub fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message(Msg::FieldUpdate(field, value)) => {
                if let Ok(value) = value.parse::<u64>() {
                    self.view_for_field(&field).value = value;
                }
            }
        }
        Command::none()
    }

    fn view_for_field(&mut self, field: &Field) -> &mut LabelledInputText<Field> {
        match field {
            Field::Money => &mut self.money,
        }
    }
}

struct LabelledInputText<Id> {
    id: Id,
    text_input: text_input::State,
    value: u64,
}

impl<Id: 'static + Clone + Display> LabelledInputText<Id> {
    fn new(id: Id, value: u64) -> LabelledInputText<Id> {
        LabelledInputText {
            id,
            text_input: text_input::State::new(),
            value,
        }
    }

    fn view<'a, Msg, F>(&'a mut self, make_message: F) -> Element<'a, Msg>
    where
        F: 'static + Fn(Id, String) -> Msg,
        Msg: 'a + Clone,
    {
        let label = format!("{}", self.id);
        let entity_id = self.id.clone();

        let input = TextInput::new(
            &mut self.text_input,
            &label,
            &self.value.to_string(),
            move |value| {
                // Not sure why just moving the view's entity_id is not enough, but given how
                // cheap a Field is I can leave with that clone.
                let entity_id = entity_id.clone();
                make_message(entity_id, value)
            },
        )
        .style(crate::editor_widget::style::MainPane);

        Row::new()
            .width(Length::FillPortion(1))
            .push(Text::new(format!("{}: ", label)))
            .push(input)
            .into()
    }
}
