use super::*;

use unicode_segmentation::UnicodeSegmentation;

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