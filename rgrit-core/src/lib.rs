#![no_std]

use core::fmt::Formatter;

#[derive(Clone)]
pub struct StaticBitmap {
    pub gfx: &'static [u8],
    pub palette: &'static [u8],
    pub map: &'static [u8],
    pub meta: &'static [u8],
    pub spec: BitmapSpec,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Compression {
    #[default]
    Off,
    LZ77,
    Huffman,
    RLE,
    OffHeader,
}

#[derive(Clone, Copy, Debug)]
pub enum Color {
    RGB { r: u8, g: u8, b: u8 },
    GBR16(u16),
}

#[derive(Clone, Copy, Debug)]
pub enum Transparency {
    Disabled,
    Color(Color),
}

impl Default for Transparency {
    fn default() -> Self {
        Transparency::Color(Color::RGB {
            r: 0xFF,
            g: 0,
            b: 0xFF,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub enum BitDepth {
    A3I5,
    A5I3,
    FourByFour,
    Custom(u8),
}

#[derive(Clone, Copy, Debug)]
pub struct BitmapSpec {
    pub bit_depth: Option<BitDepth>,
    pub format: GfxFormat,
    pub transparency: Transparency,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum GfxFormat {
    #[default]
    Bitmap,
    Tile,
}

impl core::fmt::Debug for StaticBitmap {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("StaticBitmap")
            .field("gfx", &format_args!("[u8; {}]", self.gfx.len()))
            .field("palette", &format_args!("[u8; {}]", self.palette.len()))
            .field("map", &format_args!("[u8; {}]", self.map.len()))
            .field("meta", &format_args!("[u8; {}]", self.meta.len()))
            .field("spec", &self.spec)
            .finish()
    }
}
