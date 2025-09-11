# Contributing

## Compile and run the code

Dradis, Boomer and the other crates can be compiled and run using cargo:

```bash
$ cargo build
```

Once compiled, 2 applications can be run. Check the help for the available
options:

```bash
$ cargo run --bin dradis -- --help
$ cargo run --bin boomer -- --help
```

### Running Boomer

Boomer needs to select a GPU device to use. This is done by passing the
`--device` option. By default, it will use `/dev/dri/card0`:

```bash
$ cargo run --bin boomer -- --device /dev/dri/card1
```

### Running Dradis

Dradis needs to select a media controller device to use. To find the correct
device, run the following command and look for the device bridge:

```bash
$ v4l2-ctl --list-devices
[...]
rp1-cfe (platform:1f00128000.csi):
        /dev/video0
        /dev/video1
        /dev/video2
        /dev/media0
```

Then, pass this device to Dradis and select a test configuration file:

```bash
$ cargo run --bin dradis -- --device /dev/media0 ./dradis/samples/test-single-mode-720p.yaml
```

## Future Plans

- [ ] We want to evaluate the Rockchip RK3588 System-on-Chip that features an HDMI receiver directly into the SoC. There's a driver for it in Linux since 6.15, and it's said to be capable of handling 2160p/60fps.

- [ ] Implement tests for hotplugging. This includes various scenarios, like:
  - [ ] Testing that if the same display is disconnected and reconnected, the signal will be emitted again with the same timings.
  - [ ] Testing that, if a display is disconnected and another one is reconnected:
	- [ ] If the KMS application handles hotplug signals, the timings emitted should match the new one.
	- [ ] If the KMS application doesn't handle hotplug signals, the timings emitted should match the old one.
  - [ ] This means that we also need to implement a system to pass data from `dradis` to `boomer` to tell it if it should ignore hotplugging or not. Putting some metadata in the vendor-specific parts of the EDIDs sounds like the most plausible candidate.

- [ ] Test infoframes
- [ ] Test Audio output
- [ ] Test CEC
- [ ] Expand the tests to something other than HDMI. DisplayPort, and MIPI-DSI seem like obvious candidates.

