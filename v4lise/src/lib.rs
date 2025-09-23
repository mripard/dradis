#![warn(rust_2018_idioms)]
#![warn(missing_debug_implementations)]

mod capabilities;
mod device;
mod queue;

pub use crate::{device::Device, queue::Queue};
