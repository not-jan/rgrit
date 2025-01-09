use rgrit_core::Compression;
use rgrit_core::GfxFormat;
use rgrit_rs::Bitmap;
use rgrit_rs::BitmapBuilder;

use proc_macro::TokenStream;
use quote::quote;
use syn::Ident;
use syn::LitInt;
use syn::{parse::Parse, parse_macro_input, LitStr};

#[derive(Debug, Clone)]
struct Grit {
    bitmap: Bitmap,
}

impl Parse for Grit {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit = input.parse::<LitStr>()?;

        if input.is_empty() {
            let bitmap = BitmapBuilder::new(lit.value())
                .with_transparency(rgrit_core::Transparency::Disabled)
                .with_bit_depth_override(rgrit_core::BitDepth::Custom(16))
                .with_format(GfxFormat::Bitmap)
                .build()
                .map_err(|e| {
                    let msg = format!("Failed to load {}: {}", lit.value(), e);
                    syn::Error::new(lit.span(), msg)
                })?;

            Ok(Grit { bitmap })
        } else {
            if !input.peek(syn::Token![,]) {
                return Err(syn::Error::new(input.span(), "Expected comma after input"));
            }
            input.parse::<syn::Token![,]>()?;

            let mut builder = BitmapBuilder::new(lit.value());

            while !input.is_empty() {
                let ident = input.parse::<Ident>()?;
                input.parse::<syn::Token![=]>()?;

                match ident.to_string().as_str() {
                    "transparency" => {
                        if input.peek(Ident) {
                            let ident = input.parse::<Ident>()?;
                            match ident.to_string().as_str() {
                                "Disabled" => {
                                    builder = builder
                                        .with_transparency(rgrit_core::Transparency::Disabled)
                                }
                                _ => {
                                    return Err(syn::Error::new(
                                        ident.span(),
                                        "Unknown transparency",
                                    ))
                                }
                            }
                        } else if input.peek(LitInt) {
                            todo!()
                        } else {
                            return Err(syn::Error::new(
                                input.span(),
                                "Expected identifier or literal",
                            ));
                        }
                    }
                    "bit_depth" => {
                        if input.peek(Ident) {
                            let ident = input.parse::<Ident>()?;
                            match ident.to_string().as_str() {
                                "A3I5" => {
                                    builder =
                                        builder.with_bit_depth_override(rgrit_core::BitDepth::A3I5)
                                }
                                "A5I3" => {
                                    builder =
                                        builder.with_bit_depth_override(rgrit_core::BitDepth::A5I3)
                                }
                                "FourByFour" | "4x4" => {
                                    builder = builder
                                        .with_bit_depth_override(rgrit_core::BitDepth::FourByFour)
                                }
                                _ => {
                                    return Err(syn::Error::new(ident.span(), "Unknown bit depth"))
                                }
                            }
                        } else if input.peek(LitInt) {
                            let lit = input.parse::<LitInt>()?;
                            builder = builder.with_bit_depth_override(
                                rgrit_core::BitDepth::Custom(lit.base10_parse()?),
                            );
                        } else {
                            return Err(syn::Error::new(
                                input.span(),
                                "Expected identifier or literal",
                            ));
                        }
                    }
                    "format" => {
                        let format_ident = input.parse::<Ident>()?;

                        match format_ident.to_string().as_str() {
                            "Bitmap" => builder = builder.with_format(GfxFormat::Bitmap),
                            "Tile" => builder = builder.with_format(GfxFormat::Tile),
                            _ => {
                                return Err(syn::Error::new(format_ident.span(), "Unknown format"))
                            }
                        }
                    }
                    "tile_width" => {
                        let lit = input.parse::<LitInt>()?;
                        builder = builder.with_tile_width(lit.base10_parse()?);
                    }
                    "tile_height" => {
                        let lit = input.parse::<LitInt>()?;
                        builder = builder.with_tile_height(lit.base10_parse()?);
                    }
                    "meta_width" => {
                        let lit = input.parse::<LitInt>()?;
                        builder = builder.with_meta_width(lit.base10_parse()?);
                    }
                    "meta_height" => {
                        let lit = input.parse::<LitInt>()?;
                        builder = builder.with_meta_height(lit.base10_parse()?);
                    }
                    "area_left" => {
                        let lit = input.parse::<LitInt>()?;
                        builder = builder.with_area_left(lit.base10_parse()?);
                    }
                    "area_right" => {
                        let lit = input.parse::<LitInt>()?;
                        builder = builder.with_area_right(lit.base10_parse()?);
                    }
                    "area_width" => {
                        let lit = input.parse::<LitInt>()?;
                        builder = builder.with_area_width(lit.base10_parse()?);
                    }
                    "area_top" => {
                        let lit = input.parse::<LitInt>()?;
                        builder = builder.with_area_top(lit.base10_parse()?);
                    }
                    "area_bottom" => {
                        let lit = input.parse::<LitInt>()?;
                        builder = builder.with_area_bottom(lit.base10_parse()?);
                    }
                    "area_height" => {
                        let lit = input.parse::<LitInt>()?;
                        builder = builder.with_area_height(lit.base10_parse()?);
                    }
                    "compression" => {
                        let compression_ident = input.parse::<Ident>()?;

                        match compression_ident.to_string().as_str() {
                            "Off" => builder = builder.with_compression(Compression::Off),
                            "LZ77" => builder = builder.with_compression(Compression::LZ77),
                            "Huffman" => builder = builder.with_compression(Compression::Huffman),
                            "RLE" => builder = builder.with_compression(Compression::RLE),
                            "OffHeader" => {
                                builder = builder.with_compression(Compression::OffHeader)
                            }
                            _ => {
                                return Err(syn::Error::new(
                                    compression_ident.span(),
                                    "Unknown compression",
                                ))
                            }
                        }
                    }
                    _ => return Err(syn::Error::new(ident.span(), "Unknown attribute")),
                };

                if input.peek(syn::Token![,]) {
                    input.parse::<syn::Token![,]>()?;
                }
            }

            let bitmap = builder.build().map_err(|e| {
                let msg = format!("Failed to load {}: {}", lit.value(), e);
                syn::Error::new(lit.span(), msg)
            })?;

            Ok(Grit { bitmap })
        }
    }
}

#[proc_macro]
pub fn grit(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Grit);

    // Put all the fields into a struct as `&'static [u8]`.
    let gfx = input.bitmap.gfx;
    let palette = input.bitmap.palette;
    let map = input.bitmap.map;
    let meta = input.bitmap.meta;

    // Also put some metadata so we can automatically display it.
    let bit_depth = match input.bitmap.spec.bit_depth {
        Some(rgrit_core::BitDepth::A3I5) => quote! { Some(rgrit_core::BitDepth::A3I5) },
        Some(rgrit_core::BitDepth::A5I3) => quote! { Some(rgrit_core::BitDepth::A5I3) },
        Some(rgrit_core::BitDepth::FourByFour) => quote! { Some(rgrit_core::BitDepth::FourByFour) },
        Some(rgrit_core::BitDepth::Custom(n)) => quote! { Some(rgrit_core::BitDepth::Custom(#n)) },

        None => quote! { None },
    };
    let format = match input.bitmap.spec.format {
        rgrit_core::GfxFormat::Bitmap => quote! { rgrit_core::GfxFormat::Bitmap },
        rgrit_core::GfxFormat::Tile => quote! { rgrit_core::GfxFormat::Tile },
    };
    let transparency = match input.bitmap.spec.transparency {
        rgrit_core::Transparency::Disabled => quote! { rgrit_core::Transparency::Disabled },
        rgrit_core::Transparency::Color(rgrit_core::Color::RGB { r, g, b }) => {
            quote! { grit_core::Transparency::Color(rgrit_core::Color::RGB { r: #r, g: #g, b: #b }) }
        }
        rgrit_core::Transparency::Color(rgrit_core::Color::GBR16(clr)) => {
            quote! { grit_core::Transparency::Color(rgrit_core::Color::GBR16(#clr)) }
        }
    };

    quote! {
        rgrit::StaticBitmap {
            gfx: &[#(#gfx),*],
            palette: &[#(#palette),*],
            map: &[#(#map),*],
            meta: &[#(#meta),*],
            spec: rgrit_core::BitmapSpec {
                bit_depth: #bit_depth,
                format: #format,
                transparency: #transparency,
            },
        }
    }
    .into()
}
