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

// Re-define some iced type with our theme predefined.
pub type Element<'a, Message> = iced::Element<'a, Message, crate::theme::Theme>;
