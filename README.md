# HDMI Output Test System

## General Architecture

Dradis is the main component of a system designed to test the HDMI output of a
Device-Under-Test (DUT), from another dedicated board.

It's been prototyped with a RaspberryPi4 using an
[Auvidea B101 HDMI to MIPI-CSI Bridge](https://auvidea.eu/b101-hdmi-to-csi-2-bridge-15-pin-fpc/).

It relies on the DUT sending frames with a QR-Code containing some metadata that
Dradis will decode and check. This will allow it to make sure the frame has been
properly sent and thus report whether or not the HDMI output is functional.

A system image for a RaspberryPi4 can be found
[here](https://github.com/mripard/pegasus-debian). In addition to dradis, it
also has a dnsmasq instance already set up to work as a DHCP proxy server that
will allow the DUT to network boot the Linux Kernel image we want to test.
It also embeds the latest version of the
[GitHub Actions Runner](https://github.com/actions/runner) in order to register
as a self-hosted runner that will be able to perform the HDMI output test and be
part of the usual Github Actions CI system.

On the DUT side, Boomer, a reference implementation of a KMS application setting
up a test pattern and a QR-Code at boot can be found
[here](https://github.com/mripard/dradis/tree/main/boomer), and a system image
embedding Boomer can be found [here]().
