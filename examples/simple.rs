use rgrit::GfxFormat;

fn main() {
    let bitmap = rgrit::BitmapBuilder::new("assets/test.png")
        .with_transparency(rgrit::Transparency::Disabled)
        .with_bit_depth_override(rgrit::BitDepth::Custom(16))
        .with_format(GfxFormat::Bitmap)
        .build()
        .unwrap();

    std::fs::write("test.bin", bitmap.gfx).unwrap();
}
