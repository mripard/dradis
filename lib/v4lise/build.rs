extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/wrapper.h");

    let bindings = bindgen::Builder::default()
	.header("src/wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
	.parse_callbacks(Box::new(bindgen::CargoCallbacks))
	.derive_debug(true)
	.derive_default(true)
	.rustified_enum(".*")
	.whitelist_var("V4L2_CAP_.*")
	// This is not a v4l2_capability, but a v4l2_captureparm one
	.blacklist_item("V4L2_CAP_TIMEPERFRAME")
	.whitelist_type("v4l2_buf_type")
	.whitelist_type("v4l2_buffer")
	.whitelist_type("v4l2_capability")
	.whitelist_type("v4l2_fmtdesc")
	.whitelist_type("v4l2_frmsizeenum")
	.whitelist_type("v4l2_format")
	.whitelist_type("v4l2_memory")
	.whitelist_type("v4l2_requestbuffers")
	.rustfmt_bindings(true)
	.generate()
	.expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
	.write_to_file(out_path.join("bindings.rs"))
	.expect("Couldn't write bindings!");
}
