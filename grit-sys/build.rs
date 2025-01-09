use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.hpp");
    println!("cargo:rerun-if-changed=build.rs");

    let grit_path = PathBuf::from("grit")
        .canonicalize()
        .expect("cannot canonicalize path");

    let libdir_path = grit_path.join(".libs");

    if !Command::new("git")
        .arg("submodule")
        .arg("init")
        .output()
        .expect("Failed to run git. Is it installed?")
        .status
        .success()
    {
        panic!("Failed to run git. Is it installed?");
    }

    if !Command::new("git")
        .arg("submodule")
        .arg("update")
        .output()
        .expect("Failed to run git. Is it installed?")
        .status
        .success()
    {
        panic!("Failed to run git. Is it installed?");
    }

    let is_patched = Command::new("git")
        .arg("apply")
        .arg("--reverse")
        .arg("--check")
        .arg("../grit.patch")
        .current_dir(&grit_path)
        .output()
        .expect("Failed to run git. Is it installed?");

    if !is_patched.status.success()
        && !Command::new("git")
            .arg("apply")
            .arg("../grit.patch")
            .current_dir(&grit_path)
            .output()
            .expect("Failed to run git. Is it installed?")
            .status
            .success()
    {
        panic!("Failed to apply grit patch");
    }

    if !Command::new("./autogen.sh")
        .current_dir(&grit_path)
        .output()
        .expect("Failed to run autoreconf. Is it installed?")
        .status
        .success()
    {
        panic!("Failed to run autoreconf. Is it installed?");
    }

    if !Command::new("./configure")
        .current_dir(&grit_path)
        .output()
        .expect("Failed to run configure.")
        .status
        .success()
    {
        panic!("Failed to run configure.");
    }

    if !Command::new("make")
        .current_dir(&grit_path)
        .output()
        .expect("Failed to run make. Is it installed?")
        .status
        .success()
    {
        panic!("Failed to build grit");
    }

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", libdir_path.to_str().unwrap());

    if cfg!(target_os = "macos") {
        println!("cargo:rustc-link-search=/opt/homebrew/lib");
        println!("cargo:rustc-link-lib=c++");
    } else {
        println!("cargo:rustc-link-lib=stdc++");
    }

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
