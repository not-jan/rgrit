use rgrit::StaticBitmap;

const BACKGROUND: StaticBitmap = rgrit::grit! {
    "assets/test.png",
    transparency = Disabled,
    bit_depth = 8,
    format = Bitmap,
};

fn main() {
    dbg!(&BACKGROUND);
}
