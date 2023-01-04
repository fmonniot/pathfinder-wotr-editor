mod alignment;
mod character;
mod editor;
mod input;
mod player;

// module building blocks
use alignment::AlignmentWidget;
use character::{CharacterWidget, Message as CharacterMessage};
use player::{Message as PlayerMessage, PlayerWidget};

// exposed components
pub use editor::{EditorWidget, Message as EditorMessage};

// Always import widget types from this module since it
// uses our custom theme instead of the built-in iced::Theme.
// Otherwise you will get compilation errors since iced::Element
// expects use of iced::Theme by default.
// The same apply to other built-in types like iced::widget::Container.
// Note that it is only true for the annotated types, usually the rust
// compiler can infer the proper theme by itself.
use crate::theme::Theme;

pub type Renderer = iced::Renderer<Theme>;
pub type Element<'a, Message> = iced::Element<'a, Message, Renderer>;
