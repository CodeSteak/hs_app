use super::*;


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

pub trait WithMargin<W: Widget> {
    fn margin(self, x : isize, y : isize) -> Margin<W>;
}

impl<W : Widget+Sized> WithMargin<W> for W {
    fn margin(self, x : isize, y : isize) -> Margin<W> {
        Margin((x,y), self)
    }
}