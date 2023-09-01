extern crate cbindgen;

use std::env;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    cbindgen::Builder::new()
      .with_crate(crate_dir)
      .rename_item("Runtime", "void")
      .with_language(cbindgen::Language::C)
      .with_cpp_compat(false)
      .with_no_includes()
      .generate()
      .expect("Unable to generate bindings")
      .write_to_file("bindings.h");
}