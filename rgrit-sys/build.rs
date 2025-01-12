use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.hpp");
    println!("cargo:rerun-if-changed=build.rs");

    let grit_path = PathBuf::from("grit")
        .canonicalize()
        .expect("cannot canonicalize path");

    // Build grit in the path `grit` and install it in `$OUT_DIR`
    let dst = if cfg!(target_os = "macos") {
        // Autotools doesn't pick up installed libraries on macOS automatically so we need to
        // manually add the include and library paths for brew
        // TODO: Figure out
        //   a) if there's a better way to do this and
        //   b) how to make this work with e.g. macports
        autotools::Config::new(&grit_path)
            .ldflag("-L/opt/homebrew/lib")
            .cxxflag("-std=c++14")
            .cxxflag("-I/opt/homebrew/include")
            .enable_static()
            .build()
    } else {
        autotools::Config::new(&grit_path)
            .cxxflag("-std=c++14")
            .enable_static()
            .build()
    };

    println!(
        "cargo:rustc-link-search=native={}",
        dst.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=grit");

    println!("cargo:rustc-link-lib=grit");
    println!("cargo:rustc-link-lib=cldib");

    println!("cargo:rustc-link-lib=freeimage");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let mut builder = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.hpp")
        .clang_arg("-I./grit/libgrit/")
        .clang_arg("-I./grit/cldib/")
        .clang_arg("-I./grit/extlib/")
        .clang_arg("-I./grit/srcgrit/");

    if cfg!(target_os = "macos") {
        builder = builder.clang_arg("-I/opt/homebrew/include");
    }

    let bindings = builder
        .allowlist_file("./grit/libgrit/grit_core.h")
        .allowlist_file("./grit/libcldib/cldib_core.h")
        .allowlist_file("./grit/extlib/fi.h")
        .use_core()
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
