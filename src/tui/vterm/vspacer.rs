use super::*;

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

    fn get(&mut self, x: isize, y: isize) -> Option<VChar> {
        self.inner.get(x, y).unwrap_or(VChar::SPACE).into()
    }
}

pub trait WithSpacer<W: Widget> {
    fn maximized(self) -> Spacer<W>;
}

impl<W : Widget+Sized> WithSpacer<W> for W {
    fn maximized(self) -> Spacer<W> {
        Spacer::new(self)
    }
}