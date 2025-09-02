# HDMI Output Test System

## General Architecture

This is an implementation of an HDMI testing tool for Linux.

The test setup relies on having a test runner and a device under test (DUT).

The runner is meant to interact with a CI system and report the status to the test. It does so by running `dradis`.

The DUT runs a specific display application, `boomer` that will display a test pattern, together with a QR-Code that contains frames metadata.

`dradis` will then capture the frames emitted by `boomer`, will retrieve those metadata, and check that the captured frames are indeed what was expected.

## Requirements

The only runner platform we've used and tested so far is a RaspberryPi4. It's been used together with a Toshiba TC358743XBG HDMI to MIPI-CSI bridge, which allows to retrieve the frames sent over an HDMI cable through the RaspberryPi4 MIPI-CSI receiver, `unicam`.

Over the years, we've tested multiple boards featuring the Toshiba TC358743XBG chip:

- [Auvidea B101](https://auvidea.eu/b101-hdmi-to-csi-2-bridge-15-pin-fpc/)
- [Geekworm X630](https://wiki.geekworm.com/X630)

Please note that the [Geekworm X1301](https://wiki.geekworm.com/X1301), while interesting, has a hardware defect that will prevent our test setup to work properly.

## Limitations

The RaspberryPi4 and the Toshiba bridge chip will max out at around 1080p/50fps, so we wouldn't be able to test higher resolutions than that.

Other hardware platforms are more capable, but they haven't been tested yet.

## Setup

The DUT and test runner need to be connected by an HDMI, going from the DUT HDMI output to the runner HDMI bridge input.

Then, `dradis` and `boomer` need to be started on the runner and DUT, respectively. `boomer` needs to be started after `dradis` has started.

The integration into the CI platform is left as an exercise for the reader. An example of such an integration can be found [here](https://github.com/mripard/pegasus-debian), which build a system image based on Debian/Raspberry Pi OS to register and act as a Github runner.
