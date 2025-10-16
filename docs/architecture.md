# Architecture

The goal of the project is to test the HDMI output of a device under test.

To achieve this goal, Dradis and Boomer are run on two different devices:
the device under test and the test runner device.

The device under test is equipped with the GPU to be tested and executes Boomer
to display a test pattern that includes a QR code containing frame metadata.

The test runner device, usually a Raspberry Pi with a HDMI to CSI-2 bridge,
executes Dradis to receive the output of the device under test and, using the
metadata embedded in the QR code, verify the correctness of the received frames.

The project includes the following components:

- [Boomer](boomer/README.md), a Linux KMS application that outputs a test
  pattern and a QR-Code
- [Dradis](dradis/README.md), a Linux Video4Linux2 application that captures the
  frames sent over HDMI, and will make sure they match what `boomer` expected.

And a number of libraries to support `boomer` and `dradis`:

- [dradis-frame-check](dradis-frame-check/README.md), a crate implementing the
  frame decoding, metadata parsing and integrity checks.
- [dradis-threads-pool](dradis-threads-pool/README.md), a crate to spawn new
  threads to execute closures, with a pre-defined maximum limit on the number of
  threads to spawn.
- [facet-enum-repr](facet-enum-repr/README.md), a crate to implement Rust
  `TryFrom`/`Into` traits for an enum discriminant type.
- [facet-enum-repr-derive](facet-enum-repr-derive/README.md), Rust derive macro
  implementation for `facet-enum-repr]`
- [linux-mc](linux-mc/README.md), a crate to support Linux
  [media-controller API](https://docs.kernel.org/userspace-api/media/mediactl/media-controller.html).
- [linux-raw](linux-raw/README.md), a crate to deal with various low-level
  structures and mechanisms.
- [v4l2-raw](v4l2-raw/README.md), a crate supporting the Linux
  [Video4Linux2 API](https://docs.kernel.org/userspace-api/media/v4l/v4l2.html)
- [v4lise](v4lise/README.md), a historical crate to support `v4l2`. Mostly some
  sugar-coating around `v4l2-raw` now, and likely to be removed soon.
