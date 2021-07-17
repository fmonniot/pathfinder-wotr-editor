use crate::json::{Id, JsonPatch, JsonPointer};
use crate::styles;
use iced::{text_input, Element, Length, Row, Text, TextInput};
use serde::Serialize;
use std::fmt::Display;
use std::string::ToString;

pub(super) struct LabelledInputNumber<D, V> {
    id: Id,
    ptr: JsonPointer,
    pub discriminator: D,
    pub value: V,
    text_input: text_input::State,
    disabled: bool,
}

impl<D, V> LabelledInputNumber<D, V>
where
    D: 'static + Clone + Display,
    V: Copy + ToString + Serialize,
{
    pub fn new(discriminator: D, value: V, id: Id, ptr: JsonPointer) -> LabelledInputNumber<D, V> {
        LabelledInputNumber {
            id,
            ptr,
            discriminator,
            value,
            text_input: text_input::State::new(),
            disabled: false,
        }
    }

    pub fn disabled(
        discriminator: D,
        value: V,
        id: Id,
        ptr: JsonPointer,
    ) -> LabelledInputNumber<D, V> {
        LabelledInputNumber {
            id,
            ptr,
            discriminator,
            value,
            text_input: text_input::State::new(),
            disabled: true,
        }
    }

    pub fn view<'a, Msg, F>(&'a mut self, make_message: F) -> Element<'a, Msg>
    where
        F: 'static + Fn(D, String, bool) -> Msg,
        Msg: 'a + Clone,
    {
        let label = format!("{}", self.discriminator);
        let discriminator = self.discriminator.clone();
        let disabled = self.disabled;

        let input = TextInput::new(
            &mut self.text_input,
            &label,
            &self.value.to_string(),
            move |value| {
                // Not sure why just moving the view's discriminator is not enough, but given how
                // cheap a Field is I can live with that clone.
                let discriminator = discriminator.clone();
                make_message(discriminator, value, disabled)
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
        if self.disabled {
            JsonPatch::None
        } else {
            JsonPatch::id_at_pointer(
                self.id.clone(),
                self.ptr.clone(),
                serde_json::to_value(self.value).unwrap(),
            )
        }
    }
}
