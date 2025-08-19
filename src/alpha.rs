use crate::*;
use embedded_graphics_core::pixelcolor::*;
use embedded_graphics_core::prelude::*;
use embedded_graphics_core::primitives::*;

pub struct AlphaCanvas<'a, C: RgbColor, const N: usize, const W: usize, const H: usize> {
    buffer: &'a mut Framebuffer<C, N, W, H>,
}

impl<'a, C: RgbColor, const N: usize, const W: usize, const H: usize> AlphaCanvas<'a, C, N, W, H>
where
    Rgba<C>: Blend<C>,
{
    pub fn new(buffer: &'a mut Framebuffer<C, N, W, H>) -> Self {
        Self { buffer }
    }
}

impl<'a, C: RgbColor, const N: usize, const W: usize, const H: usize> OriginDimensions
    for AlphaCanvas<'a, C, N, W, H>
where
    Rgba<C>: Blend<C>,
{
    fn size(&self) -> Size {
        self.buffer.size()
    }
}

impl<'a, C: RgbColor, const N: usize, const W: usize, const H: usize> DrawTarget
    for AlphaCanvas<'a, C, N, W, H>
where
    Rgba<C>: Blend<C>,
{
    type Error = core::convert::Infallible;
    type Color = Rgba<C>;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        // Access the underlying buffer once for speed.
        let buf = self.buffer.buf_mut();

        for Pixel(point, fg) in pixels {
            let x = point.x;
            let y = point.y;

            // Fast integer bounds check without converting more than once.
            if x >= 0 && x < W as i32 && y >= 0 && y < H as i32 {
                let xi = x as usize;
                let yi = y as usize;
                let idx = yi * W + xi;

                // Read current background pixel, blend, write back.
                let bg = buf[idx];
                buf[idx] = fg.blend(bg);
            }
        }

        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        use core::cmp::{max, min};

        let mut it = colors.into_iter();

        // Early out if area is empty
        if area.size.width == 0 || area.size.height == 0 {
            return Ok(());
        }

        // Framebuffer bounds
        let fb_left = 0i32;
        let fb_top = 0i32;
        let fb_right = W as i32; // exclusive
        let fb_bottom = H as i32; // exclusive

        let area_left = area.top_left.x;
        let area_top = area.top_left.y;
        let area_right = area_left + area.size.width as i32; // exclusive
        let area_bottom = area_top + area.size.height as i32; // exclusive

        // Clip once
        let clip_left = max(fb_left, area_left);
        let clip_top = max(fb_top, area_top);
        let clip_right = min(fb_right, area_right);
        let clip_bottom = min(fb_bottom, area_bottom);

        // Nothing to draw if fully outside
        if clip_left >= clip_right || clip_top >= clip_bottom {
            // Still must exhaust iterator for API contract
            let _ = it
                .by_ref()
                .take((area.size.width as usize) * (area.size.height as usize))
                .for_each(drop);
            return Ok(());
        }

        let buf = self.buffer.buf_mut();

        // Walk over the original (unclipped) area in row-major order, consuming colors
        // For rows that are outside, we just consume and skip. For the clipped span we do a tight loop.
        for y in area_top..area_bottom {
            let inside_y = y >= clip_top && y < clip_bottom;

            // Left run outside
            let left_run = (clip_left - area_left).max(0) as usize;
            // Middle run inside
            let mid_run = if inside_y {
                (clip_right - clip_left).max(0) as usize
            } else {
                // if the row is outside vertically, mid span is 0
                0
            };
            // Right run outside
            let right_run = (area_right - clip_right).max(0) as usize;

            // Consume left outside span
            for _ in 0..left_run {
                let _ = it.next();
            }

            if inside_y {
                let yi = y as usize;
                let row_start = yi * W;
                let x0 = clip_left as usize;
                let idx0 = row_start + x0;

                // Process contiguous inside span with minimal checks
                for i in 0..mid_run {
                    if let Some(fg) = it.next() {
                        let idx = idx0 + i;
                        let bg = buf[idx];
                        buf[idx] = fg.blend(bg);
                    } else {
                        break;
                    }
                }
            } else {
                // Row is completely outside vertically: consume the mid span too
                for _ in 0..((clip_right - clip_left).max(0) as usize) {
                    let _ = it.next();
                }
            }

            // Consume right outside span
            for _ in 0..right_run {
                let _ = it.next();
            }
        }

        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        use core::cmp::{max, min};

        // Early out
        let a = color.a();
        if a == 0 {
            return Ok(());
        }

        // Compute clipped rectangle
        let fb_left = 0i32;
        let fb_top = 0i32;
        let fb_right = W as i32; // exclusive
        let fb_bottom = H as i32; // exclusive

        let area_left = area.top_left.x;
        let area_top = area.top_left.y;
        let area_right = area_left + area.size.width as i32; // exclusive
        let area_bottom = area_top + area.size.height as i32; // exclusive

        let clip_left = max(fb_left, area_left);
        let clip_top = max(fb_top, area_top);
        let clip_right = min(fb_right, area_right);
        let clip_bottom = min(fb_bottom, area_bottom);

        if clip_left >= clip_right || clip_top >= clip_bottom {
            return Ok(());
        }

        let buf = self.buffer.buf_mut();

        if a == 255 {
            // Opaque: just overwrite the span rows with the opaque color
            let oc: C = color.rgb();
            for y in clip_top as usize..clip_bottom as usize {
                let row_start = y * W;
                let start = row_start + clip_left as usize;
                let end = row_start + clip_right as usize;
                for idx in start..end {
                    buf[idx] = oc;
                }
            }
            return Ok(());
        }

        // Alpha blend per pixel for the clipped span
        for y in clip_top as usize..clip_bottom as usize {
            let row_start = y * W;
            let start = row_start + clip_left as usize;
            let end = row_start + clip_right as usize;
            for idx in start..end {
                let bg = buf[idx];
                buf[idx] = color.blend(bg);
            }
        }

        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        let a = color.a();
        if a == 0 {
            return Ok(());
        }

        let buf = self.buffer.buf_mut();

        if a == 255 {
            let oc: C = color.rgb();
            for px in buf.iter_mut() {
                *px = oc;
            }
            return Ok(());
        }

        for px in buf.iter_mut() {
            let bg = *px;
            *px = color.blend(bg);
        }

        Ok(())
    }
}
