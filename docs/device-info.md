# Device Information

## About

The `device-info` directory contains useful information for debugging devices
that the project developers might not have access to.

## Structure

The information is organized in subdirectories with the following structure:

```
device-info
└── <device-name>+<bridge-vendor>-<bridge-model>
    ├── media-ctl-topology.txt
    └── urls.txt
```

For example, information about a Raspberry Pi 5 featuring a GeekWorm C779 HDIM
to CSI-2 bridge would be stored in `raspberrypi5+geekworm-c779`.

## Files

### `media-ctl-topology.txt`

This file contains information about the media controller topology.
To generate it, run:

```bash
$ media-ctl -p > media-ctl-topology.txt
```

### `urls.txt`

This file contains a list of URLs with relevant information about the bridge.
