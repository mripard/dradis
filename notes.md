# HDMI Test Device Notes

## Header Format

| 0     | 1     | 2-3      | 4-7   | 8-11        | 12-15 | Test     |
| ----- | ----- | -------- | ------| ----------- | ----- | ---------|
| Major | Minor | Reserved | Magic | Frame Index | Hash  |  Payload |

All multi-bytes data are stored in little endian.

The Major and Minor supported so far are 1 and 0, respectively.

The magic is be "CRNO" encoded in ascii, C being at offset 4 and O at offset 7.

The hash is computed using the xxhash32 algorithm.

## Test Capture Component TODO List

### Detect that we have a compatible client

We want to be able to detect that a compatible client has been connected and is
properly setting up the display chain.

There's several thing that we need to pay attention to:

* We want to wait for the attached device to send data. This is partly
  implemented in `wait_and_set_dv_timings`, but it doesn't differentiate between
  the client not sending anything for now (`ENOLINK`) and a legit error we have
  to report.

* Once we receive some data, we want to detect that we have a compatible client.
  This should be done by checking the header magic. We then have two outcome:
  
  * The header is valid. Then:

    * There's a good chance that our client has started streaming

      * We might have a false positive though. Is it something we need to worry
        about?

    * We can assume that the next frames are all going to be ok, or it should be
      an error with a threshold after which we declare the test as failed (with
      a default to 1). We should also dump all the bad frames for further
      inspection.

  * The header isn't valid. We wait for a valid header until a timeout is
    reached.

### Test multiple resolutions

We should allow the test capture application to test multiple resolutions
according to a defined test. This would require a few things to get going:

* We should go over all the resolutions we want to test and:

  * Create an EDID for that resolution
  * Set it up in the bridge
  * Start capturing the data and detect the new resolution
  * Stop after a while and switch to the other one

There's a few parameters there: the resolutions, the number of frames we want
to wait for the client, the number of consecutive frames. I guess we could use a
configuration file to create the test pattern and set the parameters

### Easy addition of new tests

We should allow to test multiple scenarios, including with something like
enabling / disabling cycles, async commits, etc.

### Test configuration

There's a number of tests we want to achieve:

* [ ] Test a single mode
* [ ] Test that we can switch between modes by relying on the hotplug signal

Interesting, needs to check

* [ ] Test a single mode with clock doubling
* [ ] Test that we can have odd timings on vc5 if DBLCLK is there
 
In addition, there's a couple we should have but can't with the current setup

* Test that we can detect whether a connector is there while the controller is off
* Test that we can't have odd timings on vc5
* Check for timeouts with 4k60 (core clock properly raised)
* Check that the scrambler is still on after a HPD pulse
* Check that after a modeset the audio still works
* Check that after an HPD pulse the CEC still has a physical address
* Check that after a modeset the CEC still works
* Check that the display with a BPC > 8 works
* Check that the display with HDR works

### Sample Configuration File

```yaml
test:
  - duration: 10
    edid:
      detailed-timing:
        clock: 74250

        hfp: 220
        hdisplay: 1280
        hbp: 330
        hsync: 40

        vfp: 20
        vdisplay: 720
        vbp: 25
        vsync: 5

  - enable: true
    edid:
      detailed-timing:
        ...
```

### How to actually deploy a test

There's a couple of different type of tests that we want to do:

* DUT-only tests:

  * We want to have a few tests that will run on the DUT, and do not involve any
    other device, such as:

    * Making sure we don't have any regressions with things like CEC access
      while the device is off, audio playback, screen detection, etc.

    * Making sure we can do an enable / disable / enable cycle without any
      timeout

  * For those tests, we'll want to have the tests local on the DUT, and control
    them through a controller board, possibly the same one that will hold the
    CSI bridge.

* DUT+Bridge tests:

  * Those tests involve a coordination between a board receiving frames and the
    DUT emitting them. In this case, both the DUT client and the controller
    "server" will need to be started around the same time, and the controller
    will eventually report if the test is successful.

In order to do that properly, a simple setup that could work would be a custom
HAT for the controller board, with:

* A slot to let the ribbon for the bridge go through

* Holes to mount the bridge and the controller

* Some way (a transistor?) to power the DUT using the 5v input

* A UART to connect to the DUT

### Summary

* [x] Convert to nix

* [x] Convert to memmap2

  * [x] Discuss how to support FDs instead of `std::fs::File`
  * [x] Do the proper code
  * [x] Send the PR

* [x] Handle the errors properly

* [x] Fix Clippy errors

* [x] Bindgen warnings

* [x] rust-fmt

* [x] Test for the return code in test-capture

* [x] Implement logging

* [x] Align the headers on test-capture and boomer

* [x] Make the header parsing code more robust

  * [x] Check Magic
  * [x] Check Version
  * [x] Check Frame index
  * [x] Check Frame

* [x] Implement the timeout if no frame is dequeued

  * [x] Make the dqueue call non-blocking
  * [x] Wait for the actual timeout

* [x] Implement the timeout if no link is detected

* [x] Implement the timeout if no header is detected

* [x] Implement the test configuration file format

* [x] Support multiple resolutions

* [x] Find a name for test-capture

* [ ] The CRC and header in general will be an issue with properties that will
      modify the output, such as the bpc, scaling, CSC, etc. How should we test
      that?

### Less important

* [ ] Make the lowlevel v4l2 functions consistent, some times they take the
      structure as an argument, sometimes they just return it

## Test Display Component TODO List

### React to hotplug

* Multiple threads: one to display the frames, one to poll for hotplug signals,
  kill the thread and respawn it.

  * Polling for hotplug?

    * poll/epoll/select on the status sysfs file doesn't work.

  * udev script?

### Misc

* [ ] Use resvg instead of image
* [ ] Embed the test image in the binary
* [ ] Check the binary size
* [x] Proper command line arguments to set the debug level
