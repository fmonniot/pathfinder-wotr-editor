use crate::json::{Id, JsonPatch, JsonPointer};
use crate::styles;
use iced::{text_input, Element, Length, Row, TextInput};
use serde::Serialize;
use std::fmt::Display;
use std::string::ToString;

#[derive(Debug, Clone)]
pub(super) enum InputChange<D> {
    Changed(D, String),
    NoOp,
}

impl<D> InputChange<D> {
    #[inline]
    pub fn map_or_else<U, D2: FnOnce() -> U, F: FnOnce(D, String) -> U>(
        self,
        default: D2,
        f: F,
    ) -> U {
        match self {
            InputChange::Changed(d, v) => f(d, v),
            InputChange::NoOp => default(),
        }
    }
}

pub(super) struct LabelledInputNumber<D, V> {
    id: Id,
    ptr: JsonPointer,
    pub discriminator: D,
    pub value: V,
    text_input: text_input::State,
    label_input: text_input::State,
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
            label_input: text_input::State::new(),
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
            label_input: text_input::State::new(),
            disabled: true,
        }
    }

    pub fn view(&mut self) -> Element<InputChange<D>> {
        let label = self.discriminator.to_string();
        let discriminator = self.discriminator.clone();
        let disabled = self.disabled;

        let label_widget = TextInput::new(
            &mut self.label_input,
            &label,
            &format!("{}:", label),
            |_| InputChange::NoOp,
        )
        .size(16)
        .style(styles::InputAsText)
        .width(Length::FillPortion(2));

        let input_widget = TextInput::new(
            &mut self.text_input,
            &label,
            &self.value.to_string(),
            move |value| {
                if disabled {
                    InputChange::NoOp
                } else {
                    InputChange::Changed(discriminator.clone(), value)
                }
            },
        )
        .style(styles::MainPane)
        .width(Length::FillPortion(1));

        Row::new()
            .width(Length::FillPortion(1))
            .push(label_widget)
            .push(input_widget)
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
