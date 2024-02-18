use crate::{
    theme::{self, Theme},
    widgets::Element,
};
use iced::widget::Component;
use iced::widget::{row, text_input};
use iced::Length;
use serde::Serialize;
use std::str::FromStr;
use std::string::ToString;

// TODO Should we find a better name ?
pub fn labelled_input_number<V, Message>(
    label: impl ToString,
    value: V,
    on_change: impl Fn(V) -> Message + 'static,
) -> LabelledInputNumber<V, Message>
where
    V: Copy + ToString + Serialize,
{
    LabelledInputNumber {
        label: label.to_string(),
        value,
        on_change: Box::new(on_change),
        disabled: false,
    }
}

pub struct LabelledInputNumber<V, Message> {
    label: String,
    value: V,
    on_change: Box<dyn Fn(V) -> Message>,
    disabled: bool,
}

impl<V, Message> LabelledInputNumber<V, Message> {
    pub fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
}

// We still need this hack so that label can be selectable, having disabled text_input
// isn't enough to get rid of it entirely.
#[derive(Clone)]
pub enum Event {
    NoOp,
    InputChanged(String),
}

impl<V, Message> Component<Message, Theme> for LabelledInputNumber<V, Message>
where
    V: ToString + FromStr,
{
    /// The internal state of this [`Component`].
    type State = ();
    /// The type of event this [`Component`] handles internally.
    type Event = Event;

    /// Processes an [`Event`](Component::Event) and updates the [`Component`] state accordingly.
    ///
    /// It can produce a `Message` for the parent application.
    fn update(&mut self, _state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            Event::InputChanged(value) if !self.disabled && !value.is_empty() => {
                V::from_str(&value).ok().map(self.on_change.as_ref())
            }
            _ => None,
        }
    }

    /// Produces the widgets of the [`Component`], which may trigger an [`Event`](Component::Event)
    /// on user interaction.
    fn view(&self, _state: &Self::State) -> Element<Self::Event> {
        // We still have to rely on this hack so that people can select and copy the labels.
        // Follow https://github.com/iced-rs/iced/issues/36 to know when we can use regular text block.
        // Moreover we do need to set an event, otherwise the input is disabled which also disable selecting and
        // copying the content… sigh, can't have good things can we?
        let label_widget = text_input(&self.label, &self.label)
            .on_input(|_| Event::NoOp)
            .size(16)
            .style(theme::TextInput::InputAsText)
            .width(Length::FillPortion(2));

        let mut input_widget =
            text_input(&self.label, &self.value.to_string()).width(Length::FillPortion(1));

        if !self.disabled {
            input_widget = input_widget.on_input(Event::InputChanged);
        }

        row(vec![])
            .push(label_widget)
            .push(input_widget)
            .width(Length::FillPortion(1))
            .into()
    }
}
