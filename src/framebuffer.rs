use core::convert::Infallible;
use embedded_graphics_core::Pixel;
use embedded_graphics_core::prelude::*;

pub struct Framebuffer<C: RgbColor, const N: usize, const W: usize, const H: usize> {
    buf: [C; N],
}

impl<C: RgbColor, const N: usize, const W: usize, const H: usize> Framebuffer<C, N, W, H> {
    pub fn new() -> Self {
        Self { buf: [C::BLACK; N] }
    }

    #[inline]
    pub fn iter_colors(&self) -> impl Iterator<Item = C> + '_ {
        self.buf.iter().copied()
    }

    #[inline]
    pub(super) fn buf_mut(&mut self) -> &mut [C; N] {
        &mut self.buf
    }
}

impl<C, const N: usize, const W: usize, const H: usize> OriginDimensions for Framebuffer<C, N, W, H>
where
    C: RgbColor,
{
    fn size(&self) -> Size {
        Size::new(W as u32, H as u32)
    }
}

impl<C, const N: usize, const W: usize, const H: usize> DrawTarget for Framebuffer<C, N, W, H>
where
    C: RgbColor,
{
    type Error = Infallible;
    type Color = C;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(point, color) in pixels {
            if point.x >= 0 && point.x < W as i32 && point.y >= 0 && point.y < H as i32 {
                let x = point.x as usize;
                let y = point.y as usize;
                self.buf[y * W + x] = color;
            }
        }
        Ok(())
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        self.buf.fill(color);
        Ok(())
    }
}
