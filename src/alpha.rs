use crate::*;
use embedded_graphics_core::pixelcolor::*;
use embedded_graphics_core::prelude::*;
use embedded_graphics_core::primitives::*;

pub struct AlphaCanvas<'a, C: RgbColor, const N: usize> {
    buffer: &'a mut Framebuffer<C, N>,
}

impl<'a, C: RgbColor, const N: usize> AlphaCanvas<'a, C, N>
where
    Rgba<C>: Blend<C>,
{
    #[inline(always)]
    pub fn new(buffer: &'a mut Framebuffer<C, N>) -> Self {
        Self { buffer }
    }
}

impl<'a, C: RgbColor, const N: usize> OriginDimensions for AlphaCanvas<'a, C, N>
where
    Rgba<C>: Blend<C>,
{
    #[inline(always)]
    fn size(&self) -> Size {
        self.buffer.size()
    }
}

impl<'a, C: RgbColor, const N: usize> DrawTarget for AlphaCanvas<'a, C, N>
where
    Rgba<C>: Blend<C>,
{
    type Error = core::convert::Infallible;
    type Color = Rgba<C>;

    #[inline(always)]
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let w_u32 = self.buffer.width as u32;
        let h_u32 = self.buffer.height as u32;
        let w = self.buffer.width;
        let buf = self.buffer.buf_mut();

        for Pixel(p, fg) in pixels {
            let x = p.x as u32;
            let y = p.y as u32;
            if x < w_u32 && y < h_u32 {
                let idx = (y * w + x) as usize;
                let bg = buf[idx];
                buf[idx] = fg.blend(bg);
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

        // Clip once against framebuffer.
        let clipped = area.intersection(&self.buffer.bounding_box());
        if clipped.size.width == 0 || clipped.size.height == 0 {
            for _ in 0..area.size.width * area.size.height {
                let _ = it.next();
            }
            return Ok(());
        }

        let w = self.buffer.width;
        let buf = self.buffer.buf_mut();

        // Horizontal consumption counts relative to original area.
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
            // Discard left part outside framebuffer
            for _ in 0..left_out {
                let _ = it.next();
            }

            if y >= cy0 && y < cy1 {
                let row_start = (y as usize) * w as usize + cx0;
                for dst in &mut buf[row_start..row_start + mid_w] {
                    if let Some(fg) = it.next() {
                        *dst = fg.blend(*dst);
                    } else {
                        break;
                    }
                }
            } else {
                // Row is fully outside vertically: still consume the inside span.
                for _ in 0..mid_w {
                    let _ = it.next();
                }
            }

            // Discard right part outside framebuffer
            for _ in 0..right_out {
                let _ = it.next();
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        if color.a() == 0 {
            return Ok(());
        }

        let clipped = area.intersection(&self.buffer.bounding_box());
        if clipped.size.width == 0 || clipped.size.height == 0 {
            return Ok(());
        }

        let w = self.buffer.width;
        let buf = self.buffer.buf_mut();

        let x0 = clipped.top_left.x as usize;
        let y0 = clipped.top_left.y as usize;
        let w_span = clipped.size.width as usize;
        let y_end = y0 + clipped.size.height as usize;

        for y in y0..y_end {
            let row = y * w as usize;
            for px in &mut buf[row + x0..row + x0 + w_span] {
                *px = color.blend(*px);
            }
        }
        Ok(())
    }

    #[inline(always)]
    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        if color.a() == 0 {
            return Ok(());
        }

        for px in self.buffer.buf_mut().iter_mut() {
            *px = color.blend(*px);
        }
        Ok(())
    }
}
