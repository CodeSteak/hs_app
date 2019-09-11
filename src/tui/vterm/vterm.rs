use super::*;

use unicode_segmentation::UnicodeSegmentation;

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

    fn try_set_size(&mut self, w: isize, _h: isize) {
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
            '\x00'..='\x19' => {
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
