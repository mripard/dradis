extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=src/wrapper.h");

    let bindings = bindgen::Builder::default()
        .clang_arg("-Isrc/headers/arm/include/")
        .header("src/wrapper.h")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .derive_debug(true)
        .derive_default(true)
        .rustified_enum(".*")
        .allowlist_var("V4L2_CAP_.*")
        .allowlist_var("V4L2_EVENT_.*")
        // This is not a v4l2_capability, but a v4l2_captureparm one
        .blocklist_item("V4L2_CAP_TIMEPERFRAME")
        .allowlist_var("V4L2_DV_BT_656_1120")
        .allowlist_type("v4l2_buf_type")
        .allowlist_type("v4l2_buffer")
        .allowlist_type("v4l2_capability")
        .allowlist_type("v4l2_edid")
        .allowlist_type("v4l2_event")
        .allowlist_type("v4l2_event_subscription")
        .allowlist_type("v4l2_dv_timings")
        .allowlist_type("v4l2_fmtdesc")
        .allowlist_type("v4l2_frmsizeenum")
        .allowlist_type("v4l2_format")
        .allowlist_type("v4l2_memory")
        .allowlist_type("v4l2_requestbuffers")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
