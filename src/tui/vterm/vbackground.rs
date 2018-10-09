use super::*;

pub struct Background<W: Widget>(pub Color, pub W);
impl<W: Widget> Widget for Background<W> {
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

trait WithBackground<W: Widget> {
    fn background(self, c : Color) -> Background<W>;
}

impl<W : Widget+Sized> WithBackground<W> for W {
    fn background(self, c : Color) -> Background<W> {
        Background(c, self)
    }
}