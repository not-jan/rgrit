# rgrit - Rust bindings for grit

[![Crates.io Version](https://img.shields.io/crates/v/rgrit-rs)](https://crates.io/crates/rgrit)
[![docs.rs](https://img.shields.io/docsrs/rgrit)](https://docs.rs/rgrit)
[![Crates.io License](https://img.shields.io/crates/l/rgrit)](https://crates.io/crates/rgrit)



This crate provides a Rust interface to [grit](https://github.com/devkitPro/grit) - the GBA Image Transmogrifier (“grit” for short).
It is a bitmap conversion tool for GBA/NDS development.

The bindings are generated using [bindgen](https://github.com/rust-lang/rust-bindgen).

The manual for grit can be found [here](https://www.coranac.com/man/grit/html/grit.htm).

## Building

```
git clone https://github.com/not-jan/rgrit.git
cd rgrit
git submodule update --init
cargo build
```

## Requirements

### MacOS

```bash
brew install automake libtool freeimage llvm
```

I'm not sure if llvm is required, but it's probably a good idea to install it anyway.

### Linux (Ubuntu / Debian)

```bash
sudo apt-get install autoconf build-essential libtool libfreeimage-dev
```

### Windows

Untested, but might work.

## Usage

```rust
use rgrit::StaticBitmap;

const BACKGROUND: StaticBitmap = rgrit::grit! {
    "assets/test.png",
    transparency = Disabled,
    bit_depth = 16,
    format = Bitmap,
};

fn main() {
    dbg!(&BACKGROUND);
}
```
