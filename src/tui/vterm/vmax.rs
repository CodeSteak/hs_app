use super::*;


pub struct VMax<W: Widget> {
    pub min_w : isize,
    pub min_h : isize,
    pub inner : W,
}

impl<W : Widget> VMax<W> {
    pub fn new(min_w : isize , min_h : isize ,inner : W ) -> Self{
        VMax { min_w, min_h, inner }
    }
}

impl<W: Widget> Widget for VMax<W> {
    fn size(&mut self) -> (isize, isize) {
        let (cw, ch) = self.inner.size();

        (cw.min(self.min_w), ch.min(self.min_h))
    }

    fn try_set_size(&mut self, w: isize, h: isize) {
        self.inner.try_set_size(
            self.min_w.min(w),
            self.min_h.min(h),
        );
    }

    fn get(&mut self, x: isize, y: isize) -> Option<VChar> {
        self.inner.get(x,y)
    }
}

pub trait WithMax<W: Widget> {
    fn max_size(self, w : isize, h : isize) -> VMax<W>;
}

impl<W : Widget+Sized> WithMax<W> for W {
    fn max_size(self, w : isize, h : isize) -> VMax<W> {
        VMax::new(w,h, self)
    }
}