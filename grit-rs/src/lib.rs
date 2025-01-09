use std::{
    ffi::{CString, NulError},
    fmt::Formatter,
};

use grit_sys::EGritCompression_GRIT_CPRS_HEADER;
use grit_sys::EGritCompression_GRIT_CPRS_RLE;
use grit_sys::{
    cldib_load, grit_alloc, grit_clear, grit_free, grit_init, grit_init_from_dib, grit_run,
    tagRGBQUAD, EGritGraphicsMode_GRIT_GFX_BMP_A, EGritGraphicsTextureFormat_GRIT_TEXFMT_4x4,
    EGritGraphicsTextureFormat_GRIT_TEXFMT_A3I5, EGritGraphicsTextureFormat_GRIT_TEXFMT_A5I3,
    RECORD,
};

use grit_sys::EGritCompression_GRIT_CPRS_HUFF;
use grit_sys::EGritCompression_GRIT_CPRS_LZ77;
use grit_sys::EGritCompression_GRIT_CPRS_OFF;
use proc_macro2::TokenStream;
use quote::TokenStreamExt;
use quote::{quote, ToTokens};

#[derive(Clone, Debug, thiserror::Error)]
pub enum Error {
    #[error("{0} is not a valid value for bit depth")]
    BadBitDepth(u8),
    #[error("Invalid input: {0}")]
    BadInput(#[from] NulError),
    #[error("Unable to find input file: {0}")]
    InputNotFound(String),
    #[error("Unable to convert input file: {0}")]
    ConversionError(String),
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Clone, Copy, Debug, Default)]
pub enum GfxFormat {
    #[default]
    Bitmap,
    Tile,
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

impl ToTokens for BitDepth {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token = match self {
            BitDepth::A3I5 => quote! {
                grit::BitDepth::A3I5
            },
            BitDepth::A5I3 => quote! {
                grit::BitDepth::A5I3
            },
            BitDepth::FourByFour => quote! {
                grit::BitDepth::FourByFour
            },
            BitDepth::Custom(n) => quote! {
                grit::BitDepth::Custom(#n)
            },
        };

        tokens.append_all(token);
    }
}

impl ToTokens for Compression {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token = match self {
            Compression::Off => quote! {
                grit::Compression::Off
            },
            Compression::LZ77 => quote! {
                grit::Compression::LZ77
            },
            Compression::Huffman => quote! {
                grit::Compression::Huffman
            },
            Compression::RLE => quote! {
                grit::Compression::RLE
            },
            Compression::OffHeader => quote! {
                grit::Compression::OffHeader
            },
        };

        tokens.append_all(token);
    }
}

impl ToTokens for GfxFormat {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token = match self {
            GfxFormat::Bitmap => quote! {
                grit::GfxFormat::Bitmap
            },
            GfxFormat::Tile => quote! {
                grit::GfxFormat::Tile
            },
        };

        tokens.append_all(token);
    }
}

impl ToTokens for Transparency {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let token = match self {
            Transparency::Disabled => quote! {
                grit::Transparency::Disabled
            },
            Transparency::Color(Color::RGB { r, g, b }) => {
                quote! {
                    grit::Transparency::Color(grit::Color::RGB { r: #r, g: #g, b: #b })
                }
            }
            Transparency::Color(Color::GBR16(clr)) => {
                quote! {
                    grit::Transparency::Color(grit::Color::GBR16(#clr))
                }
            }
        };

        tokens.append_all(token);
    }
}

#[derive(Clone, Debug, Default)]
pub struct BitmapBuilder {
    input: String,
    format: Option<GfxFormat>,
    tile_width: Option<u8>,
    tile_height: Option<u8>,
    meta_width: Option<u8>,
    meta_height: Option<u8>,
    bit_depth_override: Option<BitDepth>,
    transparency: Option<Transparency>,
    compression: Option<Compression>,

    area_left: Option<i32>,
    area_right: Option<i32>,
    area_width: Option<i32>,
    area_top: Option<i32>,
    area_bottom: Option<i32>,
    area_height: Option<i32>,
}

#[derive(Clone, Copy, Debug)]
pub struct BitmapSpec {
    pub bit_depth: Option<BitDepth>,
    pub format: GfxFormat,
    pub transparency: Transparency,
}

#[derive(Clone, Debug)]
pub struct Bitmap {
    pub gfx: Vec<u8>,
    pub palette: Vec<u8>,
    pub map: Vec<u8>,
    pub meta: Vec<u8>,
    pub spec: BitmapSpec,
}

#[derive(Clone)]
pub struct StaticBitmap {
    pub gfx: &'static [u8],
    pub palette: &'static [u8],
    pub map: &'static [u8],
    pub meta: &'static [u8],
    pub spec: BitmapSpec,
}

impl std::fmt::Debug for StaticBitmap {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StaticBitmap")
            .field("gfx", &format_args!("[u8; {}]", self.gfx.len()))
            .field("palette", &format_args!("[u8; {}]", self.palette.len()))
            .field("map", &format_args!("[u8; {}]", self.map.len()))
            .field("meta", &format_args!("[u8; {}]", self.meta.len()))
            .field("spec", &self.spec)
            .finish()
    }
}

/// # Safety
/// This trait is unsafe because it is not guaranteed that the pointers in [`RECORD`] are valid.
unsafe trait RecordExt {
    fn read(&self) -> Vec<u8>;
}

unsafe impl RecordExt for RECORD {
    fn read(&self) -> Vec<u8> {
        if self.data.is_null() {
            return Vec::new();
        }

        let length = (self.height * self.width) as usize;
        let mut buf = Vec::with_capacity(length);

        (0..length).for_each(|i| {
            buf.push(unsafe { self.data.add(i).read_unaligned() });
        });

        buf
    }
}

impl BitmapBuilder {
    pub fn new(input: impl AsRef<str>) -> BitmapBuilder {
        BitmapBuilder {
            input: input.as_ref().to_string(),
            ..Default::default()
        }
    }

    pub fn with_format(mut self, format: GfxFormat) -> Self {
        self.format = Some(format);
        self
    }

    pub fn with_tile_width(mut self, tile_width: u8) -> Self {
        self.tile_width = Some(tile_width);
        self
    }

    pub fn with_tile_height(mut self, tile_height: u8) -> Self {
        self.tile_height = Some(tile_height);
        self
    }

    pub fn with_meta_width(mut self, meta_width: u8) -> Self {
        self.meta_width = Some(meta_width);
        self
    }

    pub fn with_meta_height(mut self, meta_height: u8) -> Self {
        self.meta_height = Some(meta_height);
        self
    }

    pub fn with_bit_depth_override(mut self, bit_depth: BitDepth) -> Self {
        self.bit_depth_override = Some(bit_depth);
        self
    }

    pub fn with_transparency(mut self, transparency: Transparency) -> Self {
        self.transparency = Some(transparency);
        self
    }

    pub fn with_compression(mut self, compression: Compression) -> Self {
        self.compression = Some(compression);
        self
    }

    pub fn with_area_left(mut self, area_left: i32) -> Self {
        self.area_left = Some(area_left);
        self
    }

    pub fn with_area_right(mut self, area_right: i32) -> Self {
        self.area_right = Some(area_right);
        self
    }

    pub fn with_area_width(mut self, area_width: i32) -> Self {
        self.area_width = Some(area_width);
        self
    }

    pub fn with_area_top(mut self, area_top: i32) -> Self {
        self.area_top = Some(area_top);
        self
    }

    pub fn with_area_bottom(mut self, area_bottom: i32) -> Self {
        self.area_bottom = Some(area_bottom);
        self
    }

    pub fn with_area_height(mut self, area_height: i32) -> Self {
        self.area_height = Some(area_height);
        self
    }

    pub fn build(&self) -> Result<Bitmap> {
        let gr = unsafe { grit_alloc() };
        unsafe {
            grit_clear(gr);
            grit_init(gr);
        }

        let src = CString::new(self.input.as_bytes())?;

        let gr = unsafe { &mut (*gr) };

        gr.srcPath = src.as_ptr() as *mut i8;
        let dib = unsafe { cldib_load(gr.srcPath, core::ptr::null_mut()) };

        if dib.is_null() {
            return Err(Error::InputNotFound(self.input.clone()));
        }

        gr.srcDib = dib;

        unsafe { grit_init_from_dib(gr) };

        match self.format.unwrap_or_default() {
            GfxFormat::Bitmap => {
                gr.tileWidth = self.tile_width.unwrap_or(1);
                gr.tileHeight = self.tile_height.unwrap_or(1);
            }
            GfxFormat::Tile => {
                gr.tileWidth = self.tile_width.unwrap_or(8);
                gr.tileHeight = self.tile_height.unwrap_or(8);
            }
        }

        if let Some(bit_depth) = &self.bit_depth_override {
            match bit_depth {
                BitDepth::A3I5 => {
                    gr.gfxTexMode = EGritGraphicsTextureFormat_GRIT_TEXFMT_A3I5 as u8;
                }
                BitDepth::A5I3 => {
                    gr.gfxTexMode = EGritGraphicsTextureFormat_GRIT_TEXFMT_A5I3 as u8;
                }
                BitDepth::FourByFour => {
                    gr.gfxTexMode = EGritGraphicsTextureFormat_GRIT_TEXFMT_4x4 as u8;
                }
                // Check if the bit depth is a power of two
                BitDepth::Custom(n) if (*n & (*n - 1)) == 0 && *n > 0 && *n < 32 => {
                    gr.gfxBpp = *n;
                }
                BitDepth::Custom(n) => {
                    return Err(Error::BadBitDepth(*n));
                }
            }
        }

        let area_left = self.area_left.unwrap_or(0);
        let area_top = self.area_top.unwrap_or(0);
        gr.areaLeft = area_left;
        gr.areaTop = area_top;

        match (self.area_width, self.area_right) {
            (Some(-1), Some(area_right)) if area_right != -1 => {
                gr.areaRight = area_right;
            }
            (Some(area_width), _) if area_width != -1 => {
                gr.areaRight = gr.areaLeft + area_width;
            }
            _ => {}
        };

        match (self.area_height, self.area_bottom) {
            (Some(-1), Some(area_bottom)) if area_bottom != -1 => {
                gr.areaBottom = area_bottom;
            }
            (Some(area_width), _) if area_width != -1 => {
                gr.areaBottom = gr.areaTop + area_width;
            }
            _ => {}
        };

        gr.metaWidth = self.meta_width.unwrap_or(1);
        gr.metaHeight = self.meta_height.unwrap_or(1);

        match self.transparency.unwrap_or_default() {
            // NDS only
            Transparency::Disabled => {
                gr.gfxMode = EGritGraphicsMode_GRIT_GFX_BMP_A as u8;
            }
            Transparency::Color(Color::RGB { r, g, b }) => {
                gr.gfxHasAlpha = true;
                gr.gfxAlphaColor = tagRGBQUAD {
                    rgbBlue: b,
                    rgbGreen: g,
                    rgbRed: r,
                    rgbReserved: 0,
                };
            }
            Transparency::Color(Color::GBR16(clr)) => {
                gr.gfxHasAlpha = true;

                // 5 bit per color with one bit left to spare
                // Each channel goes from 0 to 31 (0b11111) and we want to map that to 0 to 255
                // We can do this by multiplying by 255 and dividing by 31
                let r = (clr & 0b11111) * 0b1111_1111 / 0b11111;
                let g = ((clr >> 5) & 0b11111) * 0b1111_1111 / 0b11111;
                let b = ((clr >> 10) & 0b11111) * 0b1111_1111 / 0b11111;

                gr.gfxAlphaColor = tagRGBQUAD {
                    rgbBlue: b as u8,
                    rgbGreen: g as u8,
                    rgbRed: r as u8,
                    rgbReserved: 0,
                };
            }
        }

        if let Some(compression) = &self.compression {
            let value = match compression {
                Compression::Off => EGritCompression_GRIT_CPRS_OFF as u8,
                Compression::LZ77 => EGritCompression_GRIT_CPRS_LZ77 as u8,
                Compression::Huffman => EGritCompression_GRIT_CPRS_HUFF as u8,
                Compression::RLE => EGritCompression_GRIT_CPRS_RLE as u8,
                Compression::OffHeader => EGritCompression_GRIT_CPRS_HEADER as u8,
            };

            gr.gfxCompression = value;
            gr.palCompression = value;
            gr.mapCompression = value;
        }

        let mut symbol_name = unsafe { core::mem::zeroed::<[i8; 256]>() };
        gr.bExport = false;
        gr.symName = symbol_name.as_mut_ptr();

        // This actually runs the conversion
        let result = unsafe { grit_run(gr) };

        // Read all the records
        let gfx = gr._gfxRec.read();
        let palette = gr._palRec.read();
        let map = gr._mapRec.read();
        let meta = gr._metaRec.read();

        // [`grit_free`] frees the memory allocated by [`grit_alloc`] and a bunch of nested pointers
        // If we set those pointers to null, free() will not do anything
        gr.srcDib = core::ptr::null_mut();
        gr.srcPath = core::ptr::null_mut();
        gr.symName = core::ptr::null_mut();

        unsafe { grit_free(gr as *mut _) };

        if result {
            Ok(Bitmap {
                gfx,
                palette,
                map,
                meta,
                spec: BitmapSpec {
                    bit_depth: self.bit_depth_override,
                    format: self.format.unwrap_or_default(),
                    transparency: self.transparency.unwrap_or_default(),
                },
            })
        } else {
            Err(Error::ConversionError(self.input.clone()))
        }
    }
}
