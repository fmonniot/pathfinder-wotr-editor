use crate::data::Player;
use crate::editor_widget::style;
use crate::labelled_input_number::LabelledInputNumber;
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
}

impl Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Field::Money => write!(f, "Money"),
        }
    }
}

pub struct PlayerWidget {
    money: LabelledInputNumber<Field>,
}

impl PlayerWidget {
    pub fn new(player: &Player) -> PlayerWidget {
        PlayerWidget {
            money: LabelledInputNumber::new(Field::Money, player.money),
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

    fn view_for_field(&mut self, field: &Field) -> &mut LabelledInputNumber<Field> {
        match field {
            Field::Money => &mut self.money,
        }
    }
}
