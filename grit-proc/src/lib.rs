use grit_rs::Bitmap;
use grit_rs::BitmapBuilder;
use grit_rs::GfxFormat;
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
                .with_transparency(grit_rs::Transparency::Disabled)
                .with_bit_depth_override(grit_rs::BitDepth::Custom(16))
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
                                    builder =
                                        builder.with_transparency(grit_rs::Transparency::Disabled)
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
                                        builder.with_bit_depth_override(grit_rs::BitDepth::A3I5)
                                }
                                "A5I3" => {
                                    builder =
                                        builder.with_bit_depth_override(grit_rs::BitDepth::A5I3)
                                }
                                "FourByFour" | "4x4" => {
                                    builder = builder
                                        .with_bit_depth_override(grit_rs::BitDepth::FourByFour)
                                }
                                _ => {
                                    return Err(syn::Error::new(ident.span(), "Unknown bit depth"))
                                }
                            }
                        } else if input.peek(LitInt) {
                            let lit = input.parse::<LitInt>()?;
                            builder = builder.with_bit_depth_override(grit_rs::BitDepth::Custom(
                                lit.base10_parse()?,
                            ));
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
        Some(bit_depth) => quote! { Some(#bit_depth) },
        None => quote! { None },
    };
    let format = input.bitmap.spec.format;
    let transparency = input.bitmap.spec.transparency;

    quote! {
        grit::StaticBitmap {
            gfx: &[#(#gfx),*],
            palette: &[#(#palette),*],
            map: &[#(#map),*],
            meta: &[#(#meta),*],
            spec: grit::BitmapSpec {
                bit_depth: #bit_depth,
                format: #format,
                transparency: #transparency,

            },
        }
    }
    .into()
}
