use ::tui::Color;

use std::env;

pub struct Theme {
    pub background: Color,

    pub textback1: Color,
    pub textback2: Color,

    pub text: Color,
    pub heading: Color,

    pub error : Color,
}


pub fn select_colorscheme() -> Theme {
    let truecolor = env::var("COLORTERM")
        .map(|s| s.to_lowercase().contains("truecolor"))
        .unwrap_or(false);

    if truecolor {
        Theme {
            background: solarized::CYAN,

            textback1: solarized::BASE3,
            textback2: solarized::BASE2,

            text: solarized::BASE00,
            heading: solarized::BASE01,

            error: solarized::RED,
        }
    } else {
        Theme {
            background: Color::Cyan,

            textback1: Color::Blue,
            textback2: Color::Magenta,

            text: Color::White,
            heading: Color::White,

            error: Color::Red,
        }
    }
}


mod solarized {
    use ::tui::Color;

    pub const BASE3: Color = Color::Custom(0xfd, 0xf6, 0xe3);
    pub const BASE2: Color = Color::Custom(0xee, 0xe8, 0xd5);

    pub const BASE00: Color = Color::Custom(0x65, 0x7b, 0x83);
    pub const BASE01: Color = Color::Custom(0x58, 0x6e, 0x75);

    pub const YELLOW: Color = Color::Custom(0xb5, 0x89, 0x00);
    pub const ORANGE: Color = Color::Custom(0xcb, 0x4b, 0x16);
    pub const RED: Color = Color::Custom(0xdc, 0x32, 0x2f);
    pub const MAGENTA: Color = Color::Custom(0xd3, 0x36, 0x82);
    pub const VIOLET: Color = Color::Custom(0x6c, 0x71, 0xc4);
    pub const BLUE: Color = Color::Custom(0x26, 0x8b, 0xd2);
    pub const CYAN: Color = Color::Custom(0x2a, 0xa1, 0x98);
    pub const GREEN: Color = Color::Custom(0x85, 0x99, 0x00);

}
