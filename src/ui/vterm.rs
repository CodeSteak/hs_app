use std::io;
use std::io::Write;

use unicode_segmentation::UnicodeSegmentation;

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

pub const SIMPLE_BOX: [char; 9] = ['*', '-', '*', '|', ' ', '|', '*', '-', '*'];

pub const BORDER_BOX: [char; 9] = ['┌', '─', '┐', '│', ' ', '│', '└', '─', '┘'];

pub const DOUBLE_BORDER_BOX: [char; 9] =
    ['╔', '═', '╗', '║', ' ', '║', '╚', '═', '╝'];

pub const NONE_BOX: [char; 9] = [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '];

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
        stdout.flush();
    }
}

enum Pos {
    Begin,
    Middle,
    End,
}

pub struct VBox<W: Widget>(pub [char; 9], pub Color, pub W);
impl<W: Widget> Widget for VBox<W> {
    fn size(&mut self) -> (isize, isize) {
        let (w, h) = self.2.size();

        (w + 2, h + 2)
    }

    fn try_set_size(&mut self, w: isize, h: isize) {
        self.2.try_set_size(w - 2, h - 2);
    }

    fn get(&mut self, x: isize, y: isize) -> Option<VChar> {
        let (w, h) = self.size();

        if w <= x || h <= y {
            return None;
        }

        if x < 0 || y < 0 {
            return None;
        }

        let xpos = if x == 0 {
            Pos::Begin
        } else if x == w - 1 {
            Pos::End
        } else {
            Pos::Middle
        };

        let ypos = if y == 0 {
            Pos::Begin
        } else if y == h - 1 {
            Pos::End
        } else {
            Pos::Middle
        };

        let bch = match (xpos, ypos) {
            (Pos::Begin, Pos::Begin) => self.0[0],
            (Pos::Middle, Pos::Begin) => self.0[1],
            (Pos::End, Pos::Begin) => self.0[2],

            (Pos::Begin, Pos::Middle) => self.0[3],
            (Pos::Middle, Pos::Middle) => {
                return self.2.get(x - 1, y - 1).unwrap_or(VChar::SPACE).into();
            }
            (Pos::End, Pos::Middle) => self.0[5],

            (Pos::Begin, Pos::End) => self.0[6],
            (Pos::Middle, Pos::End) => self.0[7],
            (Pos::End, Pos::End) => self.0[8],
        };

        Some(VChar::new(bch, self.1))
    }
}

pub struct Margin<W: Widget>(pub (isize, isize), pub W);
impl<W: Widget> Widget for Margin<W> {
    fn size(&mut self) -> (isize, isize) {
        let (w, h) = self.1.size();

        (w + (self.0).0 * 2, h + (self.0).1 * 2)
    }

    fn try_set_size(&mut self, w: isize, h: isize) {
        self.1.try_set_size(w - (self.0).0 * 2, h - (self.0).1 * 2);
    }

    fn get(&mut self, x: isize, y: isize) -> Option<VChar> {
        let (w, h) = self.size();

        if x < 0 || y < 0 || x >= w || y >= h {
            return None;
        }

        let xpos = if x < (self.0).0 {
            Pos::Begin
        } else if x >= w - (self.0).0 {
            Pos::End
        } else {
            Pos::Middle
        };

        let ypos = if y < (self.0).1 {
            Pos::Begin
        } else if y >= h - (self.0).1 {
            Pos::End
        } else {
            Pos::Middle
        };

        match (xpos, ypos) {
            (Pos::Middle, Pos::Middle) => self
                .1
                .get(x - (self.0).0, y - (self.0).1)
                .unwrap_or(VChar::SPACE)
                .into(),
            (_, _) => Some(VChar::SPACE),
        }
    }
}

pub struct Backgound<W: Widget>(pub Color, pub W);
impl<W: Widget> Widget for Backgound<W> {
    fn size(&mut self) -> (isize, isize) {
        self.1.size()
    }

    fn try_set_size(&mut self, w: isize, h: isize) {
        self.1.try_set_size(w, h);
    }

    fn get(&mut self, x: isize, y: isize) -> Option<VChar> {
        match self.1.get(x, y) {
            Some(mut c) => {
                if c.background == Color::None {
                    c.background = self.0;
                }
                Some(c)
            }
            None => None,
        }
    }
}

pub struct VText {
    width: isize,
    content: Vec<(Color, String)>,

    term: Option<VTerm>,
}

const DEFAULT_WIDTH: isize = 80;

impl VText {
    fn term(&mut self) -> &mut VTerm {
        if self.term.is_none() {
            let mut term = VTerm::new(self.width);
            for (color, word) in self.content.iter() {
                //TODO: Refactor
                for word in word.split_word_bounds() {
                    term.write_single_word_color(word, color.clone());
                }
            }

            self.term = Some(term);
        }

        if let Some(ref mut t) = self.term {
            return &mut *t;
        }

        unreachable!()
    }
    fn dirty(&mut self) {
        self.term = None;
    }

    pub fn colored(color: Color, s: &str) -> VText {
        VText {
            width: DEFAULT_WIDTH,
            content: vec![(color, s.to_string())],

            term: None,
        }
    }

    pub fn simple(f: &str) -> VText {
        VText {
            width: DEFAULT_WIDTH,
            content: vec![(Color::None, f.to_string())],

            term: None,
        }
    }

    pub fn write(&mut self, f: String) {
        self.dirty();

        self.content.push((Color::None, f));
    }

    pub fn write_color(&mut self, f: String, col: Color) {
        self.dirty();

        self.content.push((col, f));
    }

    pub fn clear(&mut self) {
        self.dirty();

        self.content.clear();
    }
}

impl Widget for VText {
    fn size(&mut self) -> (isize, isize) {
        self.term().size()
    }

    fn try_set_size(&mut self, w: isize, _h: isize) {
        self.dirty();

        self.width = w;
    }

    fn get(&mut self, x: isize, y: isize) -> Option<VChar> {
        self.term().get(x, y)
    }
}

pub struct Spacer<W: Widget> {
    pub w: isize,
    pub h: isize,
    pub inner: W,
}

impl<W: Widget> Spacer<W> {
    pub fn new(c: W) -> Self {
        Spacer {
            w: 0,
            h: 0,
            inner: c,
        }
    }
}

impl<W: Widget> Widget for Spacer<W> {
    fn size(&mut self) -> (isize, isize) {
        let (cw, ch) = self.inner.size();

        (cw.max(self.w), ch.max(self.h))
    }

    fn try_set_size(&mut self, w: isize, h: isize) {
        self.w = w;
        self.h = h;

        self.inner.try_set_size(w, h);
    }

    fn get(&mut self, mut x: isize, y: isize) -> Option<VChar> {
        self.inner.get(x, y).unwrap_or(VChar::SPACE).into()
    }
}

pub struct Center<W: Widget> {
    w: isize,
    h: isize,
    cw: isize,
    ch: isize,
    inner: W,
}

impl<W: Widget> Center<W> {
    pub fn new(mut c: W) -> Self {
        let (cw, ch) = c.size();

        Center {
            w: 0,
            h: 0,
            inner: c,
            cw,
            ch,
        }
    }

    fn into_inner(self) -> W {
        self.inner
    }
}

impl<W: Widget> Widget for Center<W> {
    fn size(&mut self) -> (isize, isize) {
        let (cw, ch) = self.inner.size();

        let (cw, ch) = self.inner.size();
        self.cw = cw;
        self.ch = ch;

        (cw.max(self.w), ch.max(self.h))
    }

    fn try_set_size(&mut self, w: isize, h: isize) {
        self.w = w;
        self.h = h;

        self.inner.try_set_size(w, h);

        let (cw, ch) = self.inner.size();
        self.cw = cw;
        self.ch = ch;
    }

    fn get(&mut self, mut x: isize, y: isize) -> Option<VChar> {
        let offsetx = self.w - self.cw;
        let offsety = self.h - self.ch;

        self.inner
            .get(x - offsetx / 2, y - offsety / 2)
            .unwrap_or(VChar::SPACE)
            .into()
    }
}

#[derive(Default)]
pub struct GridH {
    pub content: Vec<Box<dyn Widget>>,
}

impl GridH {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add<W: 'static + Widget>(mut self, c: W) -> Self {
        self.content.push(Box::new(c));
        self
    }

    pub fn push<W: 'static + Widget>(&mut self, c: W) {
        self.content.push(Box::new(c));
    }
}

impl Widget for GridH {
    fn size(&mut self) -> (isize, isize) {
        let mut w = 0;
        let mut h = 0;

        for c in self.content.iter_mut() {
            let (cw, ch) = c.size();

            w += cw;
            h = h.max(ch);
        }

        (w, h)
    }

    fn try_set_size(&mut self, w: isize, h: isize) {
        let len = self.content.len() as isize;
        if len == 0 {
            return;
        }

        let avg_w = w / len;

        for c in self.content.iter_mut() {
            c.try_set_size(avg_w, h)
        }
    }

    fn get(&mut self, mut x: isize, y: isize) -> Option<VChar> {
        for c in self.content.iter_mut() {
            let (cw, ch) = c.size();

            if x < cw {
                return c.get(x, y);
            } else {
                x -= cw;
            }
        }
        None
    }
}

#[derive(Default)]
pub struct GridV {
    pub content: Vec<Box<dyn Widget>>,
}

impl GridV {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add<W: 'static + Widget>(mut self, c: W) -> Self {
        self.content.push(Box::new(c));
        self
    }

    pub fn push<W: 'static + Widget>(&mut self, c: W) {
        self.content.push(Box::new(c));
    }
}

impl Widget for GridV {
    fn size(&mut self) -> (isize, isize) {
        let mut w = 0;
        let mut h = 0;

        for c in self.content.iter_mut() {
            let (cw, ch) = c.size();

            h += ch;
            w = w.max(cw);
        }

        (w, h)
    }

    fn try_set_size(&mut self, w: isize, h: isize) {
        let len = self.content.len() as isize;
        if len == 0 {
            return;
        }

        let avg_h = h / len;

        for c in self.content.iter_mut() {
            c.try_set_size(w, avg_h)
        }
    }

    fn get(&mut self, x: isize, mut y: isize) -> Option<VChar> {
        for c in self.content.iter_mut() {
            let (cw, ch) = c.size();

            if y < ch {
                return c.get(x, y);
            } else {
                y -= ch;
            }
        }
        None
    }
}

pub struct VTerm {
    width: isize,
    lines: Vec<Vec<VChar>>,

    pub tab_size: isize,
    pub tab_char: VChar,
}

impl Widget for VTerm {
    fn size(&mut self) -> (isize, isize) {
        let w = self.width;
        let h = self.lines.len();

        (w as isize, h as isize)
    }

    fn try_set_size(&mut self, w: isize, h: isize) {
        for line in self.lines.iter() {
            if line.len() > w as usize {
                return;
            }
        }

        self.width = w;
    }

    fn get(&mut self, x: isize, y: isize) -> Option<VChar> {
        let line = self.lines.get(y as usize)?;

        line.get(x as usize)?.clone().into()
    }
}

impl VTerm {
    pub fn new(width: isize) -> Self {
        VTerm {
            width,
            lines: Default::default(),

            tab_size: 4,
            tab_char: VChar {
                char: ' ',
                foreground: Color::None,
                background: Color::None,
            },
        }
    }

    pub fn write_words(&mut self, content: &str) {
        self.write_words_color(content, Color::None);
    }

    pub fn write_words_color(&mut self, content: &str, color: Color) {
        for word in content.split_word_bounds() {
            self.write_single_word_color(word, color);
        }
    }

    pub fn write(&mut self, content: &str) {
        self.write_color(content, Color::None);
    }

    pub fn write_color(&mut self, content: &str, color: Color) {
        for c in content.chars() {
            self.write_char_color(c, color);
        }
    }

    pub fn write_single_word(&mut self, content: &str) {
        self.write_single_word_color(content, Color::None);
    }

    pub fn write_single_word_color(&mut self, content: &str, color: Color) {
        if self.width == 0 {
            return; // Cannot mod by zero;
        }
        let word_len = content.len();
        let space_left = (self.width
            - (self
                .lines
                .last()
                .map(|vec| (vec.len() as isize) % (self.width))
                .unwrap_or(0))) as usize;

        if space_left < word_len && word_len < self.width as usize {
            self.write_char_color('\n', color);
        }

        for c in content.chars() {
            self.write_char_color(c, color);
        }
    }

    pub fn write_vchar(&mut self, ch: VChar) {
        self.write_char_color(ch.char, ch.foreground);
    }

    pub fn write_char_color(&mut self, ch: char, color: Color) {
        match ch {
            '\n' => {
                self.lines
                    .push(Vec::with_capacity((self.width as usize).min(1024)));
                return;
            }
            '\t' => {
                let cursor_pos = self
                    .lines
                    .last()
                    .map(|vec| (vec.len() as isize))
                    .unwrap_or(0);

                if self.tab_size == 0 || self.width == 0 {
                    return; // can't mod 0, so ignore.
                }

                let mut indent = self.tab_size - (cursor_pos % (self.tab_size));

                if cursor_pos + indent >= self.width {
                    self.write_char_color('\n', color);
                    indent = self.tab_size;
                }

                for _ in 0..indent {
                    let tc = self.tab_char;
                    self.write_char_color(tc.char, tc.foreground);
                }

                return;
            }
            '\x00'...'\x19' => {
                return;
            }
            _ => (),
        };

        if self
            .lines
            .last()
            .map(|vec| vec.len() >= self.width as usize)
            .unwrap_or(true)
        {
            self.lines
                .push(Vec::with_capacity((self.width as usize).min(1024)));
        }

        let current_line: &mut Vec<VChar> = self.lines.last_mut().unwrap();
        current_line.push(VChar {
            char: ch,
            foreground: color,
            background: Color::None,
        });
    }
}
