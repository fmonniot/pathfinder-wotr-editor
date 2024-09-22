use iced::widget::{button, container};
use iced::{border, color, theme::Palette, Background, Border, Color, Font, Theme};

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
const SELECTOR_SURFACE: Color = color!(0x252729);
const SECONDARY_MENU_SURFACE: Color = color!(0x2d3034);
const SELECTOR_BUTTON_ACTIVE: Color = color!(0x2e3136);
const SELECTOR_TEXT_MUTED: Color = color!(0xdddddd);

const MAIN_SURFACE_BACKGROUND: Color = color!(0x36393F);

pub fn app_theme() -> Theme {
    // The color palette is derived from the NORD theme. We changed the
    // background color to better match the gray declination we have
    // in the lateral menu.
    let p = Palette {
        background: MAIN_SURFACE_BACKGROUND, // color!(0x282828),
        text: color!(0xeceff4),              // nord6
        primary: color!(0x8fbcbb),           // nord7
        success: color!(0xa3be8c),           // nord14
        danger: color!(0xbf616a),            // nord11
    };
    Theme::custom("Wotr Editor".to_string(), p)
}

pub fn pane_selector(_theme: &Theme) -> container::Style {
    container::Style {
        border: border::color(SELECTOR_SURFACE),
        background: Some(Background::Color(SELECTOR_SURFACE)),
        ..Default::default()
    }
}

pub fn pane_selector_button(_theme: &Theme, status: button::Status) -> button::Style {
    fn pane_selector_inactive_button() -> button::Style {
        button::Style {
            border: Border {
                radius: (0.).into(),
                ..Default::default()
            },
            text_color: SELECTOR_TEXT_MUTED,
            ..Default::default()
        }
    }

    fn pane_selecter_active_button() -> button::Style {
        button::Style {
            background: Some(Background::Color(SELECTOR_BUTTON_ACTIVE)),
            text_color: Color::WHITE,
            ..pane_selector_inactive_button()
        }
    }

    // The button is in the disabled state when it represent the already active
    // menu. As such we highlight it in the same way we would an hover or press
    // action.
    match status {
        button::Status::Active => pane_selector_inactive_button(),
        button::Status::Hovered => pane_selecter_active_button(),
        button::Status::Pressed => pane_selecter_active_button(),
        button::Status::Disabled => pane_selecter_active_button(),
    }
}

pub fn secondary_menu(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(SECONDARY_MENU_SURFACE)),
        ..Default::default()
    }
}

pub fn secondary_menu_button(_theme: &Theme, status: button::Status) -> button::Style {
    fn secondary_menu_inactive_button() -> button::Style {
        button::Style {
            border: Border {
                radius: (0.).into(),
                ..Default::default()
            },
            text_color: color!(0x87, 0x90, 0x9c),
            ..Default::default()
        }
    }

    fn secondary_menu_active_button() -> button::Style {
        button::Style {
            background: Some(Background::Color(color!(0x36, 0x39, 0x3F))),
            text_color: Color::WHITE,
            ..secondary_menu_inactive_button()
        }
    }

    // The button is in the disabled state when it represent the already active
    // menu. As such we highlight it in the same way we would an hover or press
    // action.
    match status {
        button::Status::Active => secondary_menu_inactive_button(),
        button::Status::Hovered => secondary_menu_active_button(),
        button::Status::Pressed => secondary_menu_active_button(),
        button::Status::Disabled => secondary_menu_active_button(),
    }
}

pub fn main_pane(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(MAIN_SURFACE_BACKGROUND)),
        text_color: Some(Color::WHITE),
        ..Default::default()
    }
}

pub fn army_widget(_theme: &Theme) -> container::Style {
    container::Style {
        border: Border {
            color: Color::WHITE,
            width: 1.,
            ..Default::default()
        },
        ..Default::default()
    }
}
