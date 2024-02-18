use iced::widget::{button, container, progress_bar, text, text_input};
use iced::{application, color, Background, Border, Color, Font};

// Do not forget to load custom fonts in the main function

// I had to regenerate the file cause the original had a weight of 5, which doesn't correspond to any of the
// standard weight and thus couldn't be loaded by iced. It now has a weight of 500 which correspond to the
// medium weight. That does mean I cannot use the simple `with_name` construct as this one assume a weight of 400.
pub const BECKETT_FONT: Font = Font {
    family: iced::font::Family::Name("Beckett"),
    weight: iced::font::Weight::Medium,
    stretch: iced::font::Stretch::Normal,
    style: iced::font::Style::Normal,
};

// Should probably be used by main pane widgets too
pub const BOOKLETTER_1911: Font = Font::with_name("Goudy Bookletter 1911"); // working

// TODO Inline those three using the color! macro
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

const MAIN_SURFACE_BACKGROUND: Color = Color::from_rgb(
    0x36 as f32 / 255.0,
    0x39 as f32 / 255.0,
    0x3F as f32 / 255.0,
);

#[derive(Debug, Clone, Copy, Default)]
pub struct Theme;

impl application::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: &Self::Style) -> application::Appearance {
        application::Appearance {
            background_color: color!(0x28, 0x28, 0x28),
            text_color: color!(0xeb, 0xdb, 0xb2),
        }
    }
}

impl text::StyleSheet for Theme {
    type Style = ();

    fn appearance(&self, _style: Self::Style) -> text::Appearance {
        text::Appearance {
            color: color!(0xeb, 0xdb, 0xb2).into(),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Container {
    #[default]
    Default,

    /// style marker for the widgets composing the left hand side menu structure.
    /// This part of the UI is basically a list of buttons which can be in two states:
    /// - inactive: a button is inactive when its pane isn't the one displayed or the user
    ///             doesn't have the focus on it,
    /// - selected: the opposite, when the button's pane is displayed or the user has
    ///             put focus on this button (hover or press).
    PaneSelectorSurface,
    SecondaryMenuSurface,

    MainPane,

    ArmyWidget,

    /// A style used when debugging the application during development. Ideally
    /// it should not be seen by end users :)
    #[allow(unused)]
    DebugPane,
}

impl container::StyleSheet for Theme {
    type Style = Container;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        match style {
            Container::Default => container::Appearance::default(),
            Container::PaneSelectorSurface => container::Appearance {
                border: Border {
                    color: SELECTOR_SURFACE,
                    ..Default::default()
                },
                ..Default::default()
            },
            Container::SecondaryMenuSurface => container::Appearance {
                background: Some(Background::Color(color!(0x2d, 0x30, 0x34))),
                ..Default::default()
            },
            Container::DebugPane => container::Appearance {
                background: Some(Background::Color(color!(0xDA, 0x70, 0xD6))),
                ..Default::default()
            },
            Container::MainPane => container::Appearance {
                background: Some(Background::Color(MAIN_SURFACE_BACKGROUND)),
                text_color: Some(Color::WHITE),
                ..Default::default()
            },
            Container::ArmyWidget => container::Appearance {
                border: Border {
                    color: Color::WHITE,
                    width: 1.,
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum ProgressBar {
    #[default]
    Default,
}

impl progress_bar::StyleSheet for Theme {
    type Style = ProgressBar;

    fn appearance(&self, style: &Self::Style) -> progress_bar::Appearance {
        match style {
            ProgressBar::Default => progress_bar::Appearance {
                background: Background::Color(SELECTOR_SURFACE),
                bar: Background::Color(SELECTOR_TEXT_MUTED),
                border_radius: (0.).into(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Button {
    #[default]
    Default,

    PaneSelectorActive,
    PaneSelectorInactive,

    SecondaryMenuItemActive,
    SecondaryMenuItemInactive,
}

impl button::StyleSheet for Theme {
    type Style = Button;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Default => Default::default(),
            Button::PaneSelectorActive => pane_selecter_active_button(),
            Button::PaneSelectorInactive => pane_selector_inactive_button(),
            Button::SecondaryMenuItemActive => secondary_menu_active_button(),
            Button::SecondaryMenuItemInactive => secondary_menu_inactive_button(),
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Default => Default::default(),
            Button::PaneSelectorActive | Button::PaneSelectorInactive => {
                pane_selecter_active_button()
            }
            Button::SecondaryMenuItemActive | Button::SecondaryMenuItemInactive => {
                secondary_menu_active_button()
            }
        }
    }

    fn pressed(&self, style: &Self::Style) -> button::Appearance {
        match style {
            Button::Default => Default::default(),
            Button::PaneSelectorActive | Button::PaneSelectorInactive => {
                pane_selecter_active_button()
            }
            Button::SecondaryMenuItemActive | Button::SecondaryMenuItemInactive => {
                secondary_menu_active_button()
            }
        }
    }
}

fn pane_selecter_active_button() -> button::Appearance {
    button::Appearance {
        background: Some(Background::Color(SELECTOR_BUTTON_ACTIVE)),
        text_color: Color::WHITE,
        ..pane_selector_inactive_button()
    }
}

fn pane_selector_inactive_button() -> button::Appearance {
    button::Appearance {
        border: Border {
            radius: (0.).into(),
            ..Default::default()
        },
        text_color: SELECTOR_TEXT_MUTED,
        ..Default::default()
    }
}

fn secondary_menu_active_button() -> button::Appearance {
    button::Appearance {
        background: Some(Background::Color(color!(0x36, 0x39, 0x3F))),
        text_color: Color::WHITE,
        ..secondary_menu_inactive_button()
    }
}

fn secondary_menu_inactive_button() -> button::Appearance {
    button::Appearance {
        border: Border {
            radius: (0.).into(),
            ..Default::default()
        },
        text_color: color!(0x87, 0x90, 0x9c),
        ..Default::default()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum TextInput {
    #[default]
    Default,

    /// A style that decorate a TextInput as a Text.
    ///
    /// This is a workaround until iced support selection of Text directly.
    /// See https://github.com/hecrj/iced/issues/36
    InputAsText,
}

impl text_input::StyleSheet for Theme {
    type Style = TextInput;

    fn active(&self, style: &Self::Style) -> text_input::Appearance {
        match style {
            TextInput::Default => text_input::Appearance {
                background: Background::Color(TEXTINPUT_SURFACE),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.,
                    radius: (2.).into(),
                },
                icon_color: Color::TRANSPARENT,
            },
            TextInput::InputAsText => text_input::Appearance {
                background: Background::Color(MAIN_SURFACE_BACKGROUND),
                border: Border {
                    color: Color::TRANSPARENT,
                    width: 0.,
                    radius: (0.).into(),
                },
                icon_color: Color::TRANSPARENT,
            },
        }
    }

    fn disabled(&self, _style: &Self::Style) -> text_input::Appearance {
        text_input::Appearance {
            background: Background::Color(MAIN_SURFACE_BACKGROUND),
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.,
                radius: (0.).into(),
            },
            icon_color: Color::TRANSPARENT,
        }
    }

    fn focused(&self, style: &Self::Style) -> text_input::Appearance {
        match style {
            TextInput::Default => text_input::Appearance {
                border: Border {
                    color: ACCENT,
                    width: 1.,
                    ..Default::default()
                },
                ..self.active(style)
            },
            _ => self.active(style),
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgb(0.4, 0.4, 0.4)
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::WHITE
    }

    fn selection_color(&self, style: &Self::Style) -> Color {
        match style {
            TextInput::Default => ACTIVE,
            _ => self.value_color(style),
        }
    }

    fn hovered(&self, style: &Self::Style) -> text_input::Appearance {
        match style {
            TextInput::Default => text_input::Appearance {
                border: Border {
                    color: Color { a: 0.3, ..ACCENT },
                    width: 1.,
                    ..Default::default()
                },
                ..self.focused(style)
            },
            _ => self.focused(style),
        }
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::WHITE
    }
}
