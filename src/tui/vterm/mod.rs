use std::io;
use std::io::Write;

// Components :

mod vbox;
pub use self::vbox::*;

mod vmax;
pub use self::vmax::*;

mod vmargin;
pub use self::vmargin::*;

mod vbackground;
pub use self::vbackground::*;

mod vtext;
pub use self::vtext::*;

mod vspacer;
pub use self::vspacer::*;

mod vcenter;
pub use self::vcenter::*;

mod vgrid;
pub use self::vgrid::*;

mod vterm;
pub use self::vterm::*;

// TOOD: remove pos from here
enum Pos {
    Begin,
    Middle,
    End,
}


#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Color {
    None,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Bits8(u8),
    Custom(u8, u8, u8),
}

impl Default for Color {
    fn default() -> Self {
        Color::None
    }
}

enum StringLike {
    Static(&'static str),
    Dyn(String),
}

use std::fmt::{self, Display};
impl Display for StringLike {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            StringLike::Static(s) => s.fmt(f),
            StringLike::Dyn(s) => s.fmt(f),
        }
    }
}

impl Color {
    pub fn rgb(rgb: u32) -> Self {
        Color::Custom(
            ((rgb >> 16) & 0xFF) as u8,
            ((rgb >> 8) & 0xFF) as u8,
            ((rgb) & 0xFF) as u8,
        )
    }

    fn to_ansi_foreground(&self) -> impl Display {
        let stat = match self {
            Color::None => "\x1B[0m",

            Color::Black => "\x1B[30m",
            Color::Red => "\x1B[31m",
            Color::Green => "\x1B[32m",
            Color::Yellow => "\x1B[33m",
            Color::Blue => "\x1B[34m",
            Color::Magenta => "\x1B[35m",
            Color::Cyan => "\x1B[36m",
            Color::White => "\x1B[37m",
            Color::BrightBlack => "\x1B[90m",
            Color::BrightRed => "\x1B[91m",
            Color::BrightGreen => "\x1B[92m",
            Color::BrightYellow => "\x1B[93m",
            Color::BrightBlue => "\x1B[94m",
            Color::BrightMagenta => "\x1B[95m",
            Color::BrightCyan => "\x1B[96m",
            Color::BrightWhite => "\x1B[97m",
            Color::Bits8(c) => return StringLike::Dyn(format!("\x1B[38;5;{}m", c)),
            Color::Custom(r, g, b) => {
                return StringLike::Dyn(format!("\x1B[38;2;{};{};{}m", r, g, b))
            }
        };

        StringLike::Static(stat)
    }

    fn to_ansi_background(&self) -> impl Display {
        let stat = match self {
            Color::None => "\x1B[0m",

            Color::Black => "\x1B[40m",
            Color::Red => "\x1B[41m",
            Color::Green => "\x1B[42m",
            Color::Yellow => "\x1B[43m",
            Color::Blue => "\x1B[44m",
            Color::Magenta => "\x1B[45m",
            Color::Cyan => "\x1B[46m",
            Color::White => "\x1B[47m",
            Color::BrightBlack => "\x1B[100m",
            Color::BrightRed => "\x1B[101m",
            Color::BrightGreen => "\x1B[102m",
            Color::BrightYellow => "\x1B[103m",
            Color::BrightBlue => "\x1B[104m",
            Color::BrightMagenta => "\x1B[105m",
            Color::BrightCyan => "\x1B[106m",
            Color::BrightWhite => "\x1B[107m",
            Color::Bits8(c) => return StringLike::Dyn(format!("\x1B[48;5;{}m", c)),
            Color::Custom(r, g, b) => {
                return StringLike::Dyn(format!("\x1B[48;2;{};{};{}m", r, g, b))
            }
        };

        StringLike::Static(stat)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct VChar {
    char: char,
    pub foreground: Color,
    pub background: Color,
}

impl VChar {
    pub fn new(ch: char, foreground: Color) -> Self {
        VChar {
            char: ch,
            foreground,
            background: Color::None,
        }
    }

    pub fn full(ch: char, foreground: Color, background: Color) -> Self {
        VChar {
            char: ch,
            foreground,
            background,
        }
    }

    pub const SPACE: VChar = VChar {
        char: ' ',
        foreground: Color::None,
        background: Color::None,
    };
}

pub trait Widget {
    fn size(&mut self) -> (isize, isize);
    fn try_set_size(&mut self, w: isize, h: isize);
    fn get(&mut self, x: isize, y: isize) -> Option<VChar>;

    fn render_to_stdout(&mut self) {
        use std::fmt::Write;
        use std::string::String;

        let mut out = String::new();
        let mut last_foreground = Color::None;
        let mut last_background = Color::None;

        write!(out, "\x1B[0;0H"); // Goto Home
        write!(out, "{}", last_foreground.to_ansi_foreground());
        write!(out, "{}", last_background.to_ansi_foreground());

        let (w, h) = self.size();

        if w <= 0 || h <= 0 {
            return;
        }

        for y in 0..h {
            for x in 0..w {
                let vch = self.get(x, y).unwrap_or(VChar::SPACE);

                if last_foreground != vch.foreground {
                    last_foreground = vch.foreground;

                    if last_foreground == Color::None {
                        last_background = Color::None;
                    }
                    write!(out, "{}", last_foreground.to_ansi_foreground());
                }

                if last_background != vch.background {
                    last_background = vch.background;

                    write!(out, "{}", last_background.to_ansi_background());

                    if last_background == Color::None {
                        write!(out, "{}", last_foreground.to_ansi_foreground());
                    }
                }
                write!(out, "{}", vch.char);
            }

            if y != h - 1 {
                write!(out, "\n");
            }
        }

        let mut stdout = io::stdout();
        write!(stdout, "{}", out);
        let _ = stdout.flush();
    }
}
