use super::*;


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

    fn get(&mut self, x: isize, y: isize) -> Option<VChar> {
        let offsetx = self.w - self.cw;
        let offsety = self.h - self.ch;

        self.inner
            .get(x - offsetx / 2, y - offsety / 2)
            .unwrap_or(VChar::SPACE)
            .into()
    }
}

pub trait WithCenter<W: Widget> {
    fn centered(self) -> Center<W>;
}

impl<W : Widget+Sized> WithCenter<W> for W {
    fn centered(self) -> Center<W> {
        Center::new(self)
    }
}