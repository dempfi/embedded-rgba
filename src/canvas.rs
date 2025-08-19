pub use crate::*;

use embedded_graphics_core::Pixel;
use embedded_graphics_core::{prelude::*, primitives::*};

/// A strategy that backs a Canvas with different buffering policies.
pub trait BufferStrategy<T, C, const N: usize, const W: usize, const H: usize>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
{
    fn draw_iter<I>(&mut self, pixels: I)
    where
        I: IntoIterator<Item = Pixel<C>>;

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I)
    where
        I: IntoIterator<Item = C>;

    fn fill_solid(&mut self, area: &Rectangle, color: C);
    fn clear(&mut self, color: C);

    /// Push the backing buffer(s) to the provided target.
    fn flush(&mut self, target: &mut T) -> Result<(), T::Error>;
}

/// Optional access to the current full-frame buffer (for alpha blending, etc.).
pub trait HasFramebuffer<C, const N: usize, const W: usize, const H: usize>
where
    C: RgbColor,
{
    fn current_mut(&mut self) -> &mut Framebuffer<C, N, W, H>;
}

/// Double buffering: draw into `current`, compare/prepare against `reference`, then swap on flush.
pub struct DoubleBuffer<C, const N: usize, const W: usize, const H: usize>
where
    C: RgbColor,
{
    current: Framebuffer<C, N, W, H>,
    reference: Framebuffer<C, N, W, H>,
}

impl<C, const N: usize, const W: usize, const H: usize> DoubleBuffer<C, N, W, H>
where
    C: RgbColor,
{
    pub fn new() -> Self {
        Self {
            current: Framebuffer::new(),
            reference: Framebuffer::new(),
        }
    }
}

impl<T, C, const N: usize, const W: usize, const H: usize> BufferStrategy<T, C, N, W, H>
    for DoubleBuffer<C, N, W, H>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
{
    fn draw_iter<I>(&mut self, pixels: I)
    where
        I: IntoIterator<Item = Pixel<C>>,
    {
        // Framebuffer ops are infallible
        self.current.draw_iter(pixels).unwrap();
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I)
    where
        I: IntoIterator<Item = C>,
    {
        self.current.fill_contiguous(area, colors).unwrap();
    }

    fn fill_solid(&mut self, area: &Rectangle, color: C) {
        self.current.fill_solid(area, color).unwrap();
    }

    fn clear(&mut self, color: C) {
        self.current.clear(color).unwrap();
    }

    fn flush(&mut self, target: &mut T) -> Result<(), T::Error> {
        let rect = Rectangle::new(Point::zero(), Size::new(W as u32, H as u32));
        target.fill_contiguous(&rect, self.current.iter_colors())?;
        core::mem::swap(&mut self.reference, &mut self.current);
        Ok(())
    }
}

impl<C, const N: usize, const W: usize, const H: usize> HasFramebuffer<C, N, W, H>
    for DoubleBuffer<C, N, W, H>
where
    C: RgbColor,
{
    fn current_mut(&mut self) -> &mut Framebuffer<C, N, W, H> {
        &mut self.current
    }
}

/// Single buffering: only one framebuffer; flush pushes it to the target.
pub struct SingleBuffer<C, const N: usize, const W: usize, const H: usize>
where
    C: RgbColor,
{
    current: Framebuffer<C, N, W, H>,
}

impl<C, const N: usize, const W: usize, const H: usize> SingleBuffer<C, N, W, H>
where
    C: RgbColor,
{
    pub fn new() -> Self {
        Self {
            current: Framebuffer::new(),
        }
    }
}

impl<T, C, const N: usize, const W: usize, const H: usize> BufferStrategy<T, C, N, W, H>
    for SingleBuffer<C, N, W, H>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
{
    fn draw_iter<I>(&mut self, pixels: I)
    where
        I: IntoIterator<Item = Pixel<C>>,
    {
        self.current.draw_iter(pixels).unwrap();
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I)
    where
        I: IntoIterator<Item = C>,
    {
        self.current.fill_contiguous(area, colors).unwrap();
    }

    fn fill_solid(&mut self, area: &Rectangle, color: C) {
        self.current.fill_solid(area, color).unwrap();
    }

    fn clear(&mut self, color: C) {
        self.current.clear(color).unwrap();
    }

    fn flush(&mut self, target: &mut T) -> Result<(), T::Error> {
        let rect = Rectangle::new(Point::zero(), Size::new(W as u32, H as u32));
        target.fill_contiguous(&rect, self.current.iter_colors())
    }
}

impl<C, const N: usize, const W: usize, const H: usize> HasFramebuffer<C, N, W, H>
    for SingleBuffer<C, N, W, H>
where
    C: RgbColor,
{
    fn current_mut(&mut self) -> &mut Framebuffer<C, N, W, H> {
        &mut self.current
    }
}

pub struct Canvas<T, C, S, const N: usize, const W: usize, const H: usize>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
    S: BufferStrategy<T, C, N, W, H>,
{
    strategy: S,
    target: T,
}

impl<T, C, S, const N: usize, const W: usize, const H: usize> Canvas<T, C, S, N, W, H>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
    S: BufferStrategy<T, C, N, W, H>,
{
    /// Construct a canvas from an explicit strategy.
    pub fn with_strategy(target: T, strategy: S) -> Self {
        Self { strategy, target }
    }

    /// Flush via the strategy.
    pub fn flush(&mut self) -> Result<(), T::Error> {
        self.strategy.flush(&mut self.target)
    }
}

impl<T, C, S, const N: usize, const W: usize, const H: usize> OriginDimensions
    for Canvas<T, C, S, N, W, H>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
    S: BufferStrategy<T, C, N, W, H>,
{
    fn size(&self) -> Size {
        Size::new(W as u32, H as u32)
    }
}

impl<T, C, const N: usize, const W: usize, const H: usize>
    Canvas<T, C, DoubleBuffer<C, N, W, H>, N, W, H>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
{
    pub fn double_buffered(target: T) -> Self {
        Self::with_strategy(target, DoubleBuffer::new())
    }
}

impl<T, C, const N: usize, const W: usize, const H: usize>
    Canvas<T, C, SingleBuffer<C, N, W, H>, N, W, H>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
{
    pub fn single_buffered(target: T) -> Self {
        Self::with_strategy(target, SingleBuffer::new())
    }
}

impl<T, C, S, const N: usize, const W: usize, const H: usize> Canvas<T, C, S, N, W, H>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
    S: BufferStrategy<T, C, N, W, H> + HasFramebuffer<C, N, W, H>,
    Rgba<C>: Blend<C>,
{
    pub fn alpha(&mut self) -> AlphaCanvas<'_, C, N, W, H> {
        AlphaCanvas::new(self.strategy.current_mut())
    }
}

impl<T, C, S, const N: usize, const W: usize, const H: usize> DrawTarget
    for Canvas<T, C, S, N, W, H>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
    S: BufferStrategy<T, C, N, W, H>,
{
    type Error = T::Error;
    type Color = C;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), T::Error>
    where
        I: IntoIterator<Item = Pixel<T::Color>>,
    {
        self.strategy.draw_iter(pixels);
        Ok(())
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), T::Error>
    where
        I: IntoIterator<Item = T::Color>,
    {
        self.strategy.fill_contiguous(area, colors);
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: T::Color) -> Result<(), T::Error> {
        self.strategy.fill_solid(area, color);
        Ok(())
    }

    fn clear(&mut self, color: T::Color) -> Result<(), T::Error> {
        self.strategy.clear(color);
        Ok(())
    }
}
