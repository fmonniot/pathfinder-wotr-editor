// compat layer, will change once pure is the only way
pub(super) use impure::InputChange;
pub(super) use impure::LabelledInputNumber;

mod impure {
    use crate::json::{Id, JsonPatch, JsonPointer};
    use crate::styles;
    use iced::{text_input, Element, Length, Row, TextInput};
    use serde::Serialize;
    use std::fmt::Display;
    use std::string::ToString;

    #[derive(Debug, Clone)]
    pub enum InputChange<D> {
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

    pub struct LabelledInputNumber<D, V> {
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
        pub fn new(
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
                disabled: false,
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
}

pub mod pure {
    use crate::styles;
    use iced::pure::{row, text_input};
    use iced::Length;
    use iced_lazy::pure::Component;
    use iced_pure::Element;
    use serde::Serialize;
    use std::str::FromStr;
    use std::string::ToString;

    // TODO Should we find a better name ?
    pub fn labelled_input_number<V, Message>(
        label: String,
        value: V,
        on_change: impl Fn(V) -> Message + 'static,
    ) -> LabelledInputNumber<V, Message>
    where
        V: Copy + ToString + Serialize,
    {
        LabelledInputNumber {
            label,
            value,
            on_change: Box::new(on_change),
            disabled: false,
        }
    }

    pub struct LabelledInputNumber<V, Message> {
        label: String,
        pub value: V,
        on_change: Box<dyn Fn(V) -> Message>,
        disabled: bool,
    }

    impl<V, Message> LabelledInputNumber<V, Message> {
        pub fn disabled(mut self) -> Self {
            self.disabled = true;
            self
        }
    }

    #[derive(Clone)]
    pub enum Event {
        NoOp,
        InputChanged(String),
    }

    impl<V, Message, Renderer> Component<Message, Renderer> for LabelledInputNumber<V, Message>
    where
        Renderer: iced_native::text::Renderer + 'static,
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
        fn view(&self, _state: &Self::State) -> Element<Self::Event, Renderer> {
            let label_widget = text_input(&self.label, &self.label, |_| Event::NoOp)
                .size(16)
                .style(styles::InputAsText)
                .width(Length::FillPortion(2));

            let mut input_widget =
                text_input(&self.label, &self.value.to_string(), Event::InputChanged)
                    .style(styles::MainPane)
                    .width(Length::FillPortion(1));

            if self.disabled {
                input_widget = input_widget.style(styles::InputAsText);
            }

            row()
                .push(label_widget)
                .push(input_widget)
                .width(Length::FillPortion(1))
                .into()
        }
    }
}
