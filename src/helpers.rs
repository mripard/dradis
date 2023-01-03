use std::{
    os::unix::io::{AsRawFd, RawFd},
    thread::sleep,
    time::{Duration, Instant},
};

use edid::{
    EDIDDescriptor, EDIDDetailedTiming, EDIDDetailedTimingDigitalSync, EDIDDetailedTimingSync,
    EDIDDisplayColorEncoding, EDIDDisplayColorTypeEncoding, EDIDVersion,
    EDIDVideoDigitalColorDepth, EDIDVideoDigitalInterface, EDIDVideoDigitalInterfaceStandard,
    EDIDVideoInput, EDIDWeekYear, EDID,
};
use log::{debug, info};
use v4lise::{
    v4l2_buffer, v4l2_dequeue_buffer, v4l2_query_dv_timings, v4l2_queue_buffer,
    v4l2_set_dv_timings, v4l2_set_edid, Device, Result,
};

use crate::{Dradis, TestEdid, BUFFER_TYPE, MEMORY_TYPE};

pub(crate) fn dequeue_buffer(dev: &Device) -> Result<u32> {
    let mut raw_struct = v4l2_buffer {
        type_: BUFFER_TYPE as u32,
        memory: MEMORY_TYPE as u32,
        ..v4l2_buffer::default()
    };

    raw_struct = v4l2_dequeue_buffer(dev, raw_struct)?;

    Ok(raw_struct.index)
}

pub(crate) fn queue_buffer(dev: &Device, idx: u32, fd: RawFd) -> Result<()> {
    let mut raw_struct = v4l2_buffer {
        index: idx,
        type_: BUFFER_TYPE as u32,
        memory: MEMORY_TYPE as u32,
        ..v4l2_buffer::default()
    };
    raw_struct.m.fd = fd;

    v4l2_queue_buffer(dev, raw_struct)?;

    Ok(())
}

macro_rules! cast_panic {
    ($x:expr) => {
        $x.try_into().unwrap()
    };
}

// Yes, VBLANK is similar to HBLANK
#[allow(clippy::similar_names)]
pub(crate) fn set_edid(dev: &impl AsRawFd, edid: &TestEdid) -> Result<()> {
    let mut test_edid = EDID::new(EDIDVersion::V1R4)
        .set_manufacturer_id("CRN")
        .set_week_year(EDIDWeekYear::YearOfManufacture(2021))
        .set_input(EDIDVideoInput::Digital(EDIDVideoDigitalInterface::new(
            EDIDVideoDigitalInterfaceStandard::HDMIa,
            EDIDVideoDigitalColorDepth::Depth8bpc,
        )))
        .set_display_color_type_encoding(EDIDDisplayColorTypeEncoding::ColorEncoding(
            EDIDDisplayColorEncoding::RGB444,
        ))
        .set_preferred_timings_native(true);

    test_edid = match edid {
        TestEdid::DetailedTiming(dtd) => {
            let hblanking = dtd.hfp + dtd.hsync + dtd.hbp;
            let vblanking = dtd.vfp + dtd.vsync + dtd.vbp;

            test_edid.add_descriptor(EDIDDescriptor::DetailedTiming(
                EDIDDetailedTiming::new()
                    .set_front_porch(cast_panic!(dtd.hfp), cast_panic!(dtd.vfp))
                    .set_display(cast_panic!(dtd.hdisplay), cast_panic!(dtd.vdisplay))
                    .set_sync_pulse(cast_panic!(dtd.hsync), cast_panic!(dtd.vsync))
                    .set_blanking(cast_panic!(hblanking), cast_panic!(vblanking))
                    .set_pixel_clock(cast_panic!(dtd.clock))
                    .set_sync_type(EDIDDetailedTimingSync::Digital(
                        EDIDDetailedTimingDigitalSync::Separate(true, true),
                    )),
            ))
        }
    };

    v4l2_set_edid(dev, &mut test_edid.serialize())?;

    Ok(())
}

pub(crate) fn wait_and_set_dv_timings(
    suite: &Dradis<'_>,
    width: usize,
    height: usize,
) -> Result<()> {
    let start = Instant::now();

    loop {
        if start.elapsed() > suite.cfg.link_timeout {
            return Err(v4lise::Error::Empty);
        }

        let timings = v4l2_query_dv_timings(suite.dev);

        match timings {
            Ok(timings) => {
                let bt = unsafe { timings.__bindgen_anon_1.bt };

                if bt.width as usize == width && bt.height as usize == height {
                    info!("Source started to transmit the proper resolution.");
                    let _ = v4l2_set_dv_timings(suite.dev, timings)?;
                    return Ok(());
                }
            }

            Err(e) => match e {
                v4lise::Error::Io(ref io) => match io.raw_os_error() {
                    Some(libc::ENOLCK) => {
                        debug!("Link detected but unstable.");
                    }
                    Some(libc::ENOLINK) => {
                        debug!("No link detected.");
                    }
                    _ => return Err(e),
                },
                _ => return Err(e),
            },
        }

        sleep(Duration::from_millis(100));
    }
}
