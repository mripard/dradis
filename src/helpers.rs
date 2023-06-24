use std::{
    os::unix::io::{AsRawFd, RawFd},
    thread::sleep,
    time::{Duration, Instant},
};

use bitflags::bitflags;
use chrono::{Local, DateTime, TimeZone, LocalResult};
use edid::{
    EDIDDescriptor, EDIDDetailedTiming, EDIDDetailedTimingDigitalSync, EDIDDetailedTimingSync,
    EDIDDisplayColorEncoding, EDIDDisplayColorTypeEncoding, EDIDVersion,
    EDIDVideoDigitalColorDepth, EDIDVideoDigitalInterface, EDIDVideoDigitalInterfaceStandard,
    EDIDVideoInput, EDIDWeekYear, EDID,
};
use log::{debug, info, warn};
use v4lise::{
    v4l2_buffer, v4l2_dequeue_buffer, v4l2_event_subscription, v4l2_query_dv_timings, v4l2_queue_buffer,
    v4l2_set_dv_timings, v4l2_set_edid, Device, Result, v4l2_dequeue_event, v4l2_subscribe_event, v4l2_event, v4l2_event_frame_sync, v4l2_event_src_change,
    V4L2_EVENT_SOURCE_CHANGE, v4l2_start_streaming, v4l2_buf_type, v4l2_stop_streaming, v4l2_requestbuffers, MemoryType, v4l2_memory, v4l2_request_buffers, QueueType,
};

use crate::{Dradis, TestEdid, BUFFER_TYPE, MEMORY_TYPE};

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub(crate) struct EventSourceChanges: u32 {
        const RESOLUTION = 0b00000001;
    }
}

impl From<v4l2_event_src_change> for EventSourceChanges {
    fn from(value: v4l2_event_src_change) -> Self {
        Self::from_bits(value.changes)
            .unwrap()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum EventKind {
    SourceChange(EventSourceChanges),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Event {
    pub(crate) kind: EventKind,
    pub(crate) pending: usize,
    pub(crate) sequence: usize,
    pub(crate) timestamp: DateTime<Local>,
}

impl From<v4l2_event> for Event {
    fn from(value: v4l2_event) -> Self {
        Self {
            kind: match value.type_ {
                V4L2_EVENT_SOURCE_CHANGE => {
                    let event = unsafe { value.u.src_change };

                    EventKind::SourceChange(event.into())
                },
                _ => todo!(),
            },
            pending: value.pending as usize,
            sequence: value.sequence as usize,
            timestamp: {
                let secs = value.timestamp.tv_sec;
                let nsecs = value.timestamp.tv_nsec.try_into().unwrap();

                match Local.timestamp_opt(secs, nsecs) {
                    LocalResult::Single(t) => t,
                    _ => todo!(),
                }
            },
        }
    }
}

pub(crate) fn subscribe_event(dev: &Device, event_type: u32) -> Result<()> {
    let raw_struct = v4l2_event_subscription {
        type_: event_type,
        .. Default::default()
    };

    v4l2_subscribe_event(dev, raw_struct)?;

    Ok(())
}

pub(crate) fn dequeue_event(dev: &Device) -> Result<Event> {
    let raw_struct = v4l2_dequeue_event(dev)?;

    println!("{:#?}", raw_struct.type_);

    Ok(raw_struct.into())
}

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

pub(crate) fn clear_buffers(device: &Device, buf_type: v4l2_buf_type, mem_type: v4l2_memory) -> Result<()> {
    let rbuf = v4l2_requestbuffers {
        count: 0,
        type_: buf_type as u32,
        memory: mem_type as u32,
        .. Default::default()
    };

    v4l2_request_buffers(device, rbuf)?;

    Ok(())

}

pub(crate) struct StreamingDevice<'a> {
    device: &'a Device,
    buf_type: v4l2_buf_type,
}

impl Drop for StreamingDevice<'_> {
    fn drop(&mut self) {
        info!("Stopping Streaming");

        v4l2_stop_streaming(self.device, self.buf_type)
            .unwrap();

        clear_buffers(self.device, self.buf_type, v4l2_memory::V4L2_MEMORY_DMABUF)
            .unwrap();
    }
}

pub(crate) fn start_streaming<'a>(device: &'a Device, buf_type: v4l2_buf_type) -> Result<StreamingDevice<'a>> {
    info!("Starting Streaming");

    v4l2_start_streaming(device, buf_type)?;

    Ok(StreamingDevice { device, buf_type })
}