use crate::json::{Id, JsonPatch, JsonPointer};
use crate::styles;
use iced::{text_input, Element, Length, Row, Text, TextInput};
use std::fmt::Display;

pub struct LabelledInputNumber<D> {
    id: Id,
    ptr: JsonPointer,
    pub discriminator: D,
    pub value: u64,
    text_input: text_input::State,
}

impl<D: 'static + Clone + Display> LabelledInputNumber<D> {
    pub fn new(discriminator: D, value: u64, id: Id, ptr: JsonPointer) -> LabelledInputNumber<D> {
        LabelledInputNumber {
            id,
            ptr,
            discriminator,
            value,
            text_input: text_input::State::new(),
        }
    }

    pub fn view<'a, Msg, F>(&'a mut self, make_message: F) -> Element<'a, Msg>
    where
        F: 'static + Fn(D, String) -> Msg,
        Msg: 'a + Clone,
    {
        let label = format!("{}", self.discriminator);
        let discriminator = self.discriminator.clone();

        let input = TextInput::new(
            &mut self.text_input,
            &label,
            &self.value.to_string(),
            move |value| {
                // Not sure why just moving the view's discriminator is not enough, but given how
                // cheap a Field is I can leave with that clone.
                let discriminator = discriminator.clone();
                make_message(discriminator, value)
            },
        )
        .style(styles::MainPane);

        Row::new()
            .width(Length::FillPortion(1))
            .push(Text::new(format!("{}: ", label)))
            .push(input)
            .into()
    }

    pub fn change(&self) -> JsonPatch {
        JsonPatch::id_at_pointer(
            self.id.clone(),
            self.ptr.clone(),
            serde_json::to_value(self.value).unwrap(),
        )
    }
}
