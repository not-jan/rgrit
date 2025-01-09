use grit_rs::GfxFormat;

fn main() {
    let bitmap = grit_rs::BitmapBuilder::new("assets/test.png")
        .with_transparency(grit_rs::Transparency::Disabled)
        .with_bit_depth_override(grit_rs::BitDepth::Custom(16))
        .with_format(GfxFormat::Bitmap)
        .build()
        .unwrap();

    std::fs::write("test.bin", bitmap.gfx).unwrap();
}
