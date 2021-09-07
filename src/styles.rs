use iced::Font;

// Fonts (might need to move to style.rs if I create that file)
pub const CALIGHRAPHIC_FONT: Font = Font::External {
    name: "Caligraphic",
    bytes: include_bytes!("../assets/beckett/BECKETT.TTF"),
};

// Should probably be used by main pane widgets too
pub const BOOKLETTER_1911: Font = Font::External {
    name: "Goudy_Bookletter_1911",
    bytes: include_bytes!("../assets/Goudy_Bookletter_1911/GoudyBookletter1911-Regular.ttf"),
};

use iced::{button, container, progress_bar, radio, text_input, Background, Color};

const SELECTOR_SURFACE: Color = Color::from_rgb(
    0x25 as f32 / 255.0,
    0x27 as f32 / 255.0,
    0x29 as f32 / 255.0,
);

const SELECTOR_BUTTON_ACTIVE: Color = Color::from_rgb(
    0x2e as f32 / 255.0,
    0x31 as f32 / 255.0,
    0x36 as f32 / 255.0,
);

const SELECTOR_TEXT_MUTED: Color = Color::from_rgb(
    0xdd as f32 / 255.0,
    0xdd as f32 / 255.0,
    0xdd as f32 / 255.0,
);

/// style marker for the widgets composing the left hand side menu structure.
/// This part of the UI is basically a list of buttons which can be in two states:
/// - inactive: a button is inactive when its pane isn't the one displayed or the user
///             doesn't have the focus on it,
/// - selected: the opposite, when the button's pane is displayed or the user has
///             put focus on this button (hover or press).
pub struct PaneSelectorSurface;

impl container::StyleSheet for PaneSelectorSurface {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(SELECTOR_SURFACE)),
            ..container::Style::default()
        }
    }
}

impl progress_bar::StyleSheet for PaneSelectorSurface {
    fn style(&self) -> progress_bar::Style {
        progress_bar::Style {
            background: Background::Color(SELECTOR_SURFACE),
            bar: Background::Color(SELECTOR_TEXT_MUTED),
            border_radius: 0.,
        }
    }
}

pub enum PaneSelectorButton {
    Selected,
    Inactive,
}

impl PaneSelectorButton {
    fn inactive(&self) -> button::Style {
        button::Style {
            border_radius: 0.,
            text_color: SELECTOR_TEXT_MUTED,
            ..button::Style::default()
        }
    }

    fn selected(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(SELECTOR_BUTTON_ACTIVE)),
            text_color: Color::WHITE,
            ..self.inactive()
        }
    }
}

impl button::StyleSheet for PaneSelectorButton {
    // Strangely enough, the active() method return a style used when the button is not active :)
    fn active(&self) -> button::Style {
        match self {
            PaneSelectorButton::Inactive => self.inactive(),
            PaneSelectorButton::Selected => self.selected(),
        }
    }

    fn hovered(&self) -> button::Style {
        self.selected()
    }

    fn pressed(&self) -> button::Style {
        self.selected()
    }
}

pub struct SecondaryMenuSurface;

impl container::StyleSheet for SecondaryMenuSurface {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb8(0x2d, 0x30, 0x34))),
            ..container::Style::default()
        }
    }
}

pub enum SecondaryMenuItem {
    Selected,
    Inactive,
}

impl SecondaryMenuItem {
    fn inactive(&self) -> button::Style {
        button::Style {
            border_radius: 0.,
            text_color: Color::from_rgb8(0x87, 0x90, 0x9c),
            ..button::Style::default()
        }
    }

    fn selected(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(Color::from_rgb8(0x36, 0x39, 0x3F))),
            text_color: Color::WHITE,
            ..self.inactive()
        }
    }
}

impl button::StyleSheet for SecondaryMenuItem {
    // Strangely enough, the active() method return a style used when the button is not active :)
    fn active(&self) -> button::Style {
        match self {
            SecondaryMenuItem::Inactive => self.inactive(),
            SecondaryMenuItem::Selected => self.selected(),
        }
    }

    fn hovered(&self) -> button::Style {
        self.selected()
    }

    fn pressed(&self) -> button::Style {
        self.selected()
    }
}

// Can't use from_rgb8 because the constructor needs to be const fn
const TEXTINPUT_SURFACE: Color = Color::from_rgb(
    0x40 as f32 / 255.0,
    0x44 as f32 / 255.0,
    0x4B as f32 / 255.0,
);

const ACCENT: Color = Color::from_rgb(
    0x6F as f32 / 255.0,
    0xFF as f32 / 255.0,
    0xE9 as f32 / 255.0,
);

const ACTIVE: Color = Color::from_rgb(
    0x72 as f32 / 255.0,
    0x89 as f32 / 255.0,
    0xDA as f32 / 255.0,
);

const HOVERED: Color = Color::from_rgb(
    0x67 as f32 / 255.0,
    0x7B as f32 / 255.0,
    0xC4 as f32 / 255.0,
);

pub struct DebugPane;

impl container::StyleSheet for DebugPane {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(Color::from_rgb8(0xDA, 0x70, 0xD6))),
            ..container::Style::default()
        }
    }
}

const MAIN_SURFACE_BACKGROUND: Color = Color::from_rgb(
    0x36 as f32 / 255.0,
    0x39 as f32 / 255.0,
    0x3F as f32 / 255.0,
);

pub struct MainPane;

impl container::StyleSheet for MainPane {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color(MAIN_SURFACE_BACKGROUND)),
            text_color: Some(Color::WHITE),
            ..container::Style::default()
        }
    }
}

impl radio::StyleSheet for MainPane {
    fn active(&self) -> radio::Style {
        radio::Style {
            background: Background::Color(TEXTINPUT_SURFACE),
            dot_color: ACTIVE,
            border_width: 1.,
            border_color: ACTIVE,
        }
    }

    fn hovered(&self) -> radio::Style {
        radio::Style {
            background: Background::Color(Color {
                a: 0.5,
                ..TEXTINPUT_SURFACE
            }),
            ..self.active()
        }
    }
}

impl text_input::StyleSheet for MainPane {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(TEXTINPUT_SURFACE),
            border_radius: 2.,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
        }
    }

    fn focused(&self) -> text_input::Style {
        text_input::Style {
            border_width: 1.,
            border_color: ACCENT,
            ..self.active()
        }
    }

    fn hovered(&self) -> text_input::Style {
        text_input::Style {
            border_width: 1.,
            border_color: Color { a: 0.3, ..ACCENT },
            ..self.focused()
        }
    }

    fn placeholder_color(&self) -> Color {
        Color::from_rgb(0.4, 0.4, 0.4)
    }

    fn value_color(&self) -> Color {
        Color::WHITE
    }

    fn selection_color(&self) -> Color {
        ACTIVE
    }
}

impl button::StyleSheet for MainPane {
    fn active(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(ACTIVE)),
            border_radius: 3.,
            text_color: Color::WHITE,
            ..button::Style::default()
        }
    }

    fn hovered(&self) -> button::Style {
        button::Style {
            background: Some(Background::Color(HOVERED)),
            text_color: Color::WHITE,
            ..self.active()
        }
    }

    fn pressed(&self) -> button::Style {
        button::Style {
            border_width: 1.,
            border_color: Color::WHITE,
            ..self.hovered()
        }
    }
}

impl progress_bar::StyleSheet for MainPane {
    fn style(&self) -> progress_bar::Style {
        progress_bar::Style {
            background: Background::Color(TEXTINPUT_SURFACE),
            bar: Background::Color(ACTIVE),
            border_radius: 10.,
        }
    }
}

/// A style that decorate a TextInput as a Text.
///
/// This is a workaround until iced support selection of Text directly.
/// See https://github.com/hecrj/iced/issues/36
pub struct InputAsText;

impl text_input::StyleSheet for InputAsText {
    fn active(&self) -> text_input::Style {
        text_input::Style {
            background: Background::Color(MAIN_SURFACE_BACKGROUND),
            border_radius: 0.,
            border_width: 0.,
            border_color: Color::TRANSPARENT,
        }
    }

    fn focused(&self) -> text_input::Style {
        self.active()
    }

    fn placeholder_color(&self) -> Color {
        Color::from_rgb(0.4, 0.4, 0.4)
    }

    fn value_color(&self) -> Color {
        Color::WHITE
    }

    fn selection_color(&self) -> Color {
        self.value_color()
    }
}
