use std::env;
use std::path::PathBuf;

// We use the fltk fl files to create this UI, we use this file to load that into in Rust.

fn main() {
    // Tell rust to run this again if we change our UI
    println!("cargo:rerun-if-changed=src/nbt-edit-ui.fl");
    // Use this library to convert the FL into usable Rust code
    let rust_generator = fl2rust::Generator::default();
    // Output to Cargo build directory
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    // Perform conversion
    rust_generator
        .in_out(
            "src/nbt-edit-ui.fl",
            out_path.join("nbt-edit-ui.rs").to_str().unwrap(),
        )
        .expect("Failed to generate rust from fl file!");
}
