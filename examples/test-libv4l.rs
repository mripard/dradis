use std::path::PathBuf;

use v4l::Device;

fn main() {
    let path = PathBuf::from("/dev/video0");

    let dev = Device::with_path(&path).unwrap();
}
