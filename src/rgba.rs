use embedded_graphics_core::pixelcolor::raw::*;
use embedded_graphics_core::pixelcolor::*;

/// Simple RGBA color wrapper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba<C: RgbColor>(C, u8);

#[inline(always)]
fn mul_blend_u8(delta: u32, a: u32) -> u32 {
    // Exact (delta * a) / 255 using the div255 trick (no slow integer division).
    // Valid for 0..=65535 inputs; see Hacker's Delight 10-16.
    let t = delta * a + 128;
    (t + (t >> 8)) >> 8
}

impl<C: RgbColor> Rgba<C> {
    /// Create a new RGBA color.
    pub const fn new(color: C, alpha: u8) -> Self {
        Self(color, alpha)
    }

    /// Get the color component.
    pub const fn rgb(&self) -> C {
        self.0
    }

    pub fn r(&self) -> u8 {
        self.0.r()
    }

    pub fn g(&self) -> u8 {
        self.0.g()
    }

    pub fn b(&self) -> u8 {
        self.0.b()
    }

    /// Get the alpha component (0..=255).
    pub const fn a(&self) -> u8 {
        self.1
    }
}

impl<C: RgbColor> PixelColor for Rgba<C> {
    type Raw = C::Raw;
}

pub trait Blend<T> {
    fn blend(&self, bg: T) -> T;
}

impl Blend<Rgb565> for Rgba<Rgb565> {
    #[inline(always)]
    fn blend(&self, bg: Rgb565) -> Rgb565 {
        let a = self.a() as u32;
        if a == 0 {
            return bg;
        }
        if a == 255 {
            return self.rgb();
        }

        let f = self.rgb().into_storage() as u32; // 0RRRRR GGGGGG BBBBB
        let b = bg.into_storage() as u32;

        let fr = (f >> 11) & 0x1F;
        let fg = (f >> 5) & 0x3F;
        let fb = f & 0x1F;

        let br = (b >> 11) & 0x1F;
        let bgc = (b >> 5) & 0x3F;
        let bb = b & 0x1F;

        // Blend in native bit depth (5/6/5) using exact div-by-255 trick.
        let r = (br + mul_blend_u8(fr.wrapping_sub(br), a)) & 0x1F;
        let g = (bgc + mul_blend_u8(fg.wrapping_sub(bgc), a)) & 0x3F;
        let bl = (bb + mul_blend_u8(fb.wrapping_sub(bb), a)) & 0x1F;

        let out = ((r << 11) | (g << 5) | bl) as u16;
        Rgb565::from(RawU16::new(out))
    }
}

impl Blend<Rgb888> for Rgba<Rgb888> {
    #[inline(always)]
    fn blend(&self, bg: Rgb888) -> Rgb888 {
        let a = self.a() as u32;
        if a == 0 {
            return bg;
        }
        if a == 255 {
            return self.rgb();
        }

        let fr = self.rgb().r() as u32;
        let fg = self.rgb().g() as u32;
        let fb = self.rgb().b() as u32;

        let br = bg.r() as u32;
        let bgc = bg.g() as u32;
        let bb = bg.b() as u32;

        let r = (br + mul_blend_u8(fr.wrapping_sub(br), a)) as u8;
        let g = (bgc + mul_blend_u8(fg.wrapping_sub(bgc), a)) as u8;
        let b = (bb + mul_blend_u8(fb.wrapping_sub(bb), a)) as u8;

        Rgb888::new(r, g, b)
    }
}

impl Blend<Rgb666> for Rgba<Rgb666> {
    #[inline(always)]
    fn blend(&self, bg: Rgb666) -> Rgb666 {
        let a = self.a() as u32;
        if a == 0 {
            return bg;
        }
        if a == 255 {
            return self.rgb();
        }

        let fr = self.rgb().r() as u32; // 0..63
        let fg = self.rgb().g() as u32; // 0..63
        let fb = self.rgb().b() as u32; // 0..63

        let br = bg.r() as u32;
        let bgc = bg.g() as u32;
        let bb = bg.b() as u32;

        let r = (br + mul_blend_u8(fr.wrapping_sub(br), a)) as u8; // 0..63
        let g = (bgc + mul_blend_u8(fg.wrapping_sub(bgc), a)) as u8;
        let b = (bb + mul_blend_u8(fb.wrapping_sub(bb), a)) as u8;

        Rgb666::new(r, g, b)
    }
}
