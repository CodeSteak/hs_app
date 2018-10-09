use super::*;

pub mod boxtype {
    pub const SIMPLE_BOX: [char; 9] = ['*', '-', '*', '|', ' ', '|', '*', '-', '*'];

    pub const BORDER_BOX: [char; 9] = ['┌', '─', '┐', '│', ' ', '│', '└', '─', '┘'];

    pub const DOUBLE_BORDER_BOX: [char; 9] =
        ['╔', '═', '╗', '║', ' ', '║', '╚', '═', '╝'];

    pub const NONE_BOX: [char; 9] = [' ', ' ', ' ', ' ', ' ', ' ', ' ', ' ', ' '];
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

pub trait WithBox<W: Widget> {
    fn boxed(self, box_type : [char; 9], c : Color) -> VBox<W>;
}

impl<W : Widget+Sized> WithBox<W> for W {
    fn boxed(self, box_type : [char; 9], c : Color) -> VBox<W> {
        VBox(box_type, c, self)
    }
}