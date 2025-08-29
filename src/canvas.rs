pub use crate::*;

use core::convert::Infallible;
use embedded_graphics_core::Pixel;
use embedded_graphics_core::{prelude::*, primitives::*};

pub trait BufferStrategy: DrawTarget {
    fn flush<T: DrawTarget<Color = Self::Color>>(&mut self, target: &mut T)
    -> Result<(), T::Error>;
}

pub trait HasFramebuffer<C, const N: usize>
where
    C: RgbColor,
{
    fn current_mut(&mut self) -> &mut Framebuffer<C, N>;
}

/// Double buffering: draw into `current`, compare/prepare against `reference`, then swap on flush.
pub struct DoubleBuffer<C, const N: usize>
where
    C: RgbColor,
{
    current: Framebuffer<C, N>,
    reference: Framebuffer<C, N>,
}

impl<C, const N: usize> DoubleBuffer<C, N>
where
    C: RgbColor,
{
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            current: Framebuffer::new(width, height),
            reference: Framebuffer::new(width, height),
        }
    }
}

impl<C, const N: usize> DrawTarget for DoubleBuffer<C, N>
where
    C: RgbColor,
{
    type Color = C;
    type Error = Infallible;

    #[inline(always)]
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.current.draw_iter(pixels)
    }

    #[inline(always)]
    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.current.fill_contiguous(area, colors)
    }

    #[inline(always)]
    fn fill_solid(&mut self, area: &Rectangle, color: C) -> Result<(), Self::Error> {
        self.current.fill_solid(area, color)
    }

    #[inline(always)]
    fn clear(&mut self, color: C) -> Result<(), Self::Error> {
        self.current.clear(color)
    }
}

impl<C, const N: usize> OriginDimensions for DoubleBuffer<C, N>
where
    C: RgbColor,
{
    fn size(&self) -> Size {
        self.current.size()
    }
}

impl<C, const N: usize> BufferStrategy for DoubleBuffer<C, N>
where
    C: RgbColor,
{
    fn flush<T>(&mut self, target: &mut T) -> Result<(), T::Error>
    where
        T: DrawTarget<Color = Self::Color>,
    {
        target.fill_contiguous(&target.bounding_box(), self.current.iter_colors())?;
        core::mem::swap(&mut self.reference, &mut self.current);
        Ok(())
    }
}

impl<C, const N: usize> HasFramebuffer<C, N> for DoubleBuffer<C, N>
where
    C: RgbColor,
{
    #[inline(always)]
    fn current_mut(&mut self) -> &mut Framebuffer<C, N> {
        &mut self.current
    }
}

/// Single buffering: only one framebuffer; flush pushes it to the target.
pub struct SingleBuffer<C, const N: usize>
where
    C: RgbColor,
{
    current: Framebuffer<C, N>,
}

impl<C, const N: usize> SingleBuffer<C, N>
where
    C: RgbColor,
{
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            current: Framebuffer::new(width, height),
        }
    }
}

impl<C, const N: usize> BufferStrategy for SingleBuffer<C, N>
where
    C: RgbColor,
{
    fn flush<T>(&mut self, target: &mut T) -> Result<(), T::Error>
    where
        T: DrawTarget<Color = Self::Color>,
    {
        target.fill_contiguous(&target.bounding_box(), self.current.iter_colors())
    }
}

impl<C, const N: usize> DrawTarget for SingleBuffer<C, N>
where
    C: RgbColor,
{
    type Color = C;
    type Error = Infallible;

    #[inline(always)]
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.current.draw_iter(pixels)
    }

    #[inline(always)]
    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.current.fill_contiguous(area, colors)
    }

    #[inline(always)]
    fn fill_solid(&mut self, area: &Rectangle, color: C) -> Result<(), Self::Error> {
        self.current.fill_solid(area, color)
    }

    #[inline(always)]
    fn clear(&mut self, color: C) -> Result<(), Self::Error> {
        self.current.clear(color)
    }
}

impl<C, const N: usize> OriginDimensions for SingleBuffer<C, N>
where
    C: RgbColor,
{
    fn size(&self) -> Size {
        self.current.size()
    }
}

impl<C, const N: usize> HasFramebuffer<C, N> for SingleBuffer<C, N>
where
    C: RgbColor,
{
    fn current_mut(&mut self) -> &mut Framebuffer<C, N> {
        &mut self.current
    }
}

pub struct Canvas<'a, T, S>
where
    T: DrawTarget,
    S: BufferStrategy<Color = T::Color>,
{
    strategy: S,
    target: &'a mut T,
}

impl<'a, T, S> Canvas<'a, T, S>
where
    T: DrawTarget,
    S: BufferStrategy<Color = T::Color>,
{
    /// Construct a canvas from an explicit strategy.
    pub fn with_strategy(target: &'a mut T, strategy: S) -> Self {
        Self { strategy, target }
    }

    pub fn flush(&mut self) -> Result<(), T::Error> {
        self.strategy.flush(self.target)
    }
}

impl<'a, T, S> OriginDimensions for Canvas<'a, T, S>
where
    T: DrawTarget + OriginDimensions,
    S: BufferStrategy<Color = T::Color>,
{
    fn size(&self) -> Size {
        self.target.size()
    }
}

impl<'a, T, C, const N: usize> Canvas<'a, T, DoubleBuffer<C, N>>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
{
    pub fn double_buffered(target: &'a mut T) -> Self {
        let size = target.bounding_box().size;
        Self::with_strategy(target, DoubleBuffer::new(size.width, size.height))
    }
}

impl<'a, T, C, const N: usize> Canvas<'a, T, SingleBuffer<C, N>>
where
    C: RgbColor,
    T: DrawTarget<Color = C>,
{
    pub fn single_buffered(target: &'a mut T) -> Self {
        let size = target.bounding_box().size;
        Self::with_strategy(target, SingleBuffer::new(size.width, size.height))
    }
}

impl<'a, T, S> Canvas<'a, T, S>
where
    T: DrawTarget,
    S: BufferStrategy<Color = T::Color>,
    T::Color: RgbColor,
    Rgba<S::Color>: Blend<S::Color>,
{
    pub fn alpha<const N: usize>(&mut self) -> AlphaCanvas<'_, S::Color, N>
    where
        S: HasFramebuffer<S::Color, N>,
    {
        AlphaCanvas::new(self.strategy.current_mut())
    }
}

impl<'a, T, S> DrawTarget for Canvas<'a, T, S>
where
    T: DrawTarget + OriginDimensions,
    S: BufferStrategy<Color = T::Color>,
{
    type Error = S::Error;
    type Color = S::Color;

    #[inline(always)]
    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), S::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        self.strategy.draw_iter(pixels)
    }

    #[inline(always)]
    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), S::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        self.strategy.fill_contiguous(area, colors)
    }

    #[inline(always)]
    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), S::Error> {
        self.strategy.fill_solid(area, color)
    }

    #[inline(always)]
    fn clear(&mut self, color: Self::Color) -> Result<(), S::Error> {
        self.strategy.clear(color)
    }
}
