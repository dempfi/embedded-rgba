use core::convert::Infallible;
use embedded_graphics_core::prelude::*;
use embedded_graphics_core::primitives::*;

pub struct Framebuffer<C: RgbColor, const N: usize> {
    buf: [C; N],
    pub width: u32,
    pub height: u32,
}

impl<C: RgbColor, const N: usize> Framebuffer<C, N> {
    #[inline]
    pub fn new(width: u32, height: u32) -> Self {
        debug_assert_eq!(N as u32, width * height, "N must be width*height");
        Self {
            buf: [C::BLACK; N],
            width,
            height,
        }
    }

    #[inline]
    pub fn iter_colors(&self) -> impl Iterator<Item = C> + '_ {
        self.buf.iter().copied()
    }

    #[inline]
    pub fn buf(&self) -> &[C; N] {
        &self.buf
    }

    #[inline]
    pub(super) fn buf_mut(&mut self) -> &mut [C; N] {
        &mut self.buf
    }

    #[inline]
    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width as usize + x
    }
}

impl<C, const N: usize> OriginDimensions for Framebuffer<C, N>
where
    C: RgbColor,
{
    #[inline(always)]
    fn size(&self) -> Size {
        Size::new(self.width, self.height)
    }
}

impl<C, const N: usize> DrawTarget for Framebuffer<C, N>
where
    C: RgbColor,
{
    type Error = Infallible;
    type Color = C;

    #[inline(always)]
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(p, color) in pixels {
            let x = p.x as u32;
            let y = p.y as u32;
            if x < self.width && y < self.height {
                let idx = self.idx(x as usize, y as usize);
                self.buf[idx] = color;
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        let mut it = colors.into_iter();
        if area.size.width == 0 || area.size.height == 0 {
            return Ok(());
        }

        let clipped = area.intersection(&self.bounding_box());
        if clipped.size.width == 0 || clipped.size.height == 0 {
            // consume to honor e-g expectations
            for _ in 0..area.size.width * area.size.height {
                let _ = it.next();
            }
            return Ok(());
        }

        // Precompute horizontal consumption counts relative to original area.
        let left_out = (clipped.top_left.x - area.top_left.x).max(0) as usize;
        let mid_w = clipped.size.width as usize;
        let right_out =
            (area.bottom_right().unwrap().x - clipped.bottom_right().unwrap().x).max(0) as usize;

        let y0 = area.top_left.y;
        let y1 = y0 + area.size.height as i32;
        let cy0 = clipped.top_left.y;
        let cy1 = cy0 + clipped.size.height as i32;
        let cx0 = clipped.top_left.x as usize;

        for y in y0..y1 {
            // discard left outside segment
            for _ in 0..left_out {
                let _ = it.next();
            }

            if y >= cy0 && y < cy1 {
                let row_start = self.idx(cx0, y as usize);
                for dst in &mut self.buf[row_start..row_start + mid_w] {
                    if let Some(c) = it.next() {
                        *dst = c;
                    } else {
                        break;
                    }
                }
            } else {
                // row fully outside vertically; still consume the inside span
                for _ in 0..mid_w {
                    let _ = it.next();
                }
            }

            // discard right outside segment
            for _ in 0..right_out {
                let _ = it.next();
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        let clipped = area.intersection(&self.bounding_box());
        if clipped.size.width == 0 || clipped.size.height == 0 {
            return Ok(());
        }

        let x0 = clipped.top_left.x as usize;
        let y0 = clipped.top_left.y as usize;
        let span_w = clipped.size.width as usize;
        let y_end = y0 + clipped.size.height as usize;

        // Row-subslice fill = memcpy-class speed
        for y in y0..y_end {
            let start = self.idx(x0, y);
            self.buf[start..start + span_w].fill(color);
        }
        Ok(())
    }

    #[inline(always)]
    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.buf.fill(color);
        Ok(())
    }
}
