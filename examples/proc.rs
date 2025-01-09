use grit::StaticBitmap;
use grit_proc::grit;

const BACKGROUND: StaticBitmap = grit! {
    "assets/test.png",
    transparency = Disabled,
    bit_depth = 16,
    format = Bitmap,
};

fn main() {
    dbg!(&BACKGROUND);
}
