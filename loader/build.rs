use std::env;

fn main() {
    // because this script is called from the Makefile, the
    // LINKER_FILE variables, which is exported on line 43, 
    // is available as an environment variable
    let linker_file = env::var("LINKER_FILE").unwrap_or_default();

    // Recompile if linker script or build script has changed
    println!("cargo:rerun-if-changed={}", linker_file);
    println!("cargo:rerun-if-changed=build.rs");
}
