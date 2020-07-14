use crate::editor_widget::style;
use iced::{text_input, Element, Length, Row, Text, TextInput};
use std::fmt::Display;

pub struct LabelledInputNumber<Id> {
    pub id: Id,
    pub value: u64,
    text_input: text_input::State,
}

impl<Id: 'static + Clone + Display> LabelledInputNumber<Id> {
    pub fn new(id: Id, value: u64) -> LabelledInputNumber<Id> {
        LabelledInputNumber {
            id,
            value,
            text_input: text_input::State::new(),
        }
    }

    pub fn view<'a, Msg, F>(&'a mut self, make_message: F) -> Element<'a, Msg>
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
        .style(style::MainPane);

        Row::new()
            .width(Length::FillPortion(1))
            .push(Text::new(format!("{}: ", label)))
            .push(input)
            .into()
    }
}
