use super::*;


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
            let (cw, _ch) = c.size();

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
            let (_cw, ch) = c.size();

            if y < ch {
                return c.get(x, y);
            } else {
                y -= ch;
            }
        }
        None
    }
}
