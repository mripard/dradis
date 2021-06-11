use std::{
    hash::Hasher,
    os::unix::io::{AsRawFd, RawFd},
    thread::sleep,
    time::Duration,
};

use byteorder::{ByteOrder, LittleEndian};
use dma_buf::MappedDmaBuf;
use dma_heap::{DmaBufHeap, DmaBufHeapType};
use edid::{
    EDIDDescriptor, EDIDDetailedTiming, EDIDDetailedTimingDigitalSync, EDIDDetailedTimingSync,
    EDIDDisplayColorEncoding, EDIDDisplayColorTypeEncoding, EDIDVersion,
    EDIDVideoDigitalColorDepth, EDIDVideoDigitalInterface, EDIDVideoDigitalInterfaceStandard,
    EDIDVideoInput, EDIDWeekYear, EDID,
};
use log::{debug, info, warn};
use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};
use twox_hash::XxHash32;
use v4lise::{
    v4l2_buf_type, v4l2_buffer, v4l2_dequeue_buffer, v4l2_memory, v4l2_query_buffer,
    v4l2_query_dv_timings, v4l2_queue_buffer, v4l2_set_dv_timings, v4l2_set_edid,
    v4l2_start_streaming, Device, FrameFormat, MemoryType, PixelFormat, QueueType, Result,
};

struct V4L2Buffer {
    dmabuf: MappedDmaBuf,
}

const BUFFER_TYPE: v4l2_buf_type = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE;
const MEMORY_TYPE: v4l2_memory = v4l2_memory::V4L2_MEMORY_DMABUF;
const NUM_BUFFERS: usize = 5;

fn dequeue_buffer(dev: &Device) -> Result<u32> {
    let mut raw_struct: v4l2_buffer = Default::default();
    raw_struct.type_ = BUFFER_TYPE as u32;
    raw_struct.memory = MEMORY_TYPE as u32;

    raw_struct = v4l2_dequeue_buffer(dev, raw_struct)?;

    Ok(raw_struct.index)
}

fn queue_buffer(dev: &Device, idx: usize, fd: RawFd) -> Result<()> {
    let mut raw_struct: v4l2_buffer = Default::default();
    raw_struct.index = idx as u32;
    raw_struct.type_ = BUFFER_TYPE as u32;
    raw_struct.memory = MEMORY_TYPE as u32;
    raw_struct.m.fd = fd;

    v4l2_queue_buffer(dev, raw_struct)?;

    Ok(())
}

fn set_edid(dev: &impl AsRawFd) {
    let mut edid = EDID::new(EDIDVersion::V1R4)
        .set_manufacturer_id("CRN")
        .set_week_year(EDIDWeekYear::YearOfManufacture(2021))
        .set_input(EDIDVideoInput::Digital(EDIDVideoDigitalInterface::new(
            EDIDVideoDigitalInterfaceStandard::HDMIa,
            EDIDVideoDigitalColorDepth::Depth8bpc,
        )))
        .set_display_color_type_encoding(EDIDDisplayColorTypeEncoding::ColorEncoding(
            EDIDDisplayColorEncoding::RGB444,
        ))
        .set_preferred_timings_native(true)
        .add_descriptor(EDIDDescriptor::DetailedTiming(
            EDIDDetailedTiming::new()
                .set_front_porch(220, 20)
                .set_display(1280, 720)
                .set_sync_pulse(40, 5)
                .set_blanking(370, 30)
                .set_pixel_clock(74250)
                .set_sync_type(EDIDDetailedTimingSync::Digital(
                    EDIDDetailedTimingDigitalSync::Separate(true, true),
                )),
        ))
        .serialize();

    v4l2_set_edid(dev, &mut edid);
}

fn wait_and_set_dv_timings(dev: &impl AsRawFd, width: usize, height: usize) -> Result<()> {
    loop {
        let timings = v4l2_query_dv_timings(dev);

        match timings {
            Ok(timings) => {
                let bt = unsafe { timings.__bindgen_anon_1.bt };

                if bt.width as usize == width && bt.height as usize == height {
                    info!("Source started to transmit the proper resolution.");
                    let _ = v4l2_set_dv_timings(dev, timings)?;
                    return Ok(());
                }
            }

            Err(e) => match e {
                v4lise::Error::Io(ref io) => match io.raw_os_error() {
                    Some(libc::ENOLCK) => {
                        debug!("Link detected but unstable.");
                    }
                    Some(libc::ENOLINK) => {
                        debug!("No link detected.")
                    }
                    _ => return Err(e),
                },
                _ => return Err(e),
            },
        }

        sleep(Duration::from_millis(100));
    }
}

struct CapturedFrame {
    index: usize,
    frame_hash: u32,
    computed_hash: u32,
}

fn decode_captured_frame(data: &[u8]) -> std::result::Result<CapturedFrame, dma_buf::Error> {
    let index = LittleEndian::read_u16(&data[3..5]) as usize;

    let mut frame_hash: u32 = LittleEndian::read_u16(&data[6..8]) as u32;
    frame_hash = frame_hash | ((LittleEndian::read_u16(&data[9..11]) as u32) << 16);

    let mut hasher = XxHash32::with_seed(0);
    hasher.write(&data[15..]);
    let computed_hash = hasher.finish() as u32;

    Ok(CapturedFrame {
        index,
        frame_hash,
        computed_hash,
    })
}

fn main() {
    TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .unwrap();

    let heap = DmaBufHeap::new(DmaBufHeapType::Cma).unwrap();

    let mut buffers: Vec<V4L2Buffer> = Vec::with_capacity(NUM_BUFFERS);
    let dev = Device::new("/dev/video0").expect("Couldn't open the v4l2 device");
    let queue = dev
        .get_queue(QueueType::Capture)
        .expect("Couldn't get our queue");

    set_edid(&dev);

    wait_and_set_dv_timings(&dev, 1280, 720);

    let fmt = queue
        .get_pixel_formats()
        .filter(|fmt| *fmt == PixelFormat::RGB24)
        .next()
        .expect("Couldn't find our format");

    let (width, height) = queue
        .get_sizes(fmt)
        .filter(|(width, height)| *width == 1280 && *height == 720)
        .next()
        .expect("Size not supported");

    queue
        .set_format(
            queue
                .get_current_format()
                .expect("Couldn't get our queue format")
                .set_pixel_format(fmt)
                .set_frame_size(width, height),
        )
        .expect("Couldn't change our queue format");

    queue
        .request_buffers(MemoryType::DMABUF, NUM_BUFFERS)
        .expect("Couldn't request our buffers");

    for idx in 0..NUM_BUFFERS {
        let mut rbuf: v4l2_buffer = Default::default();
        rbuf.index = idx as u32;
        rbuf.type_ = BUFFER_TYPE as u32;
        rbuf.memory = MEMORY_TYPE as u32;

        rbuf = v4l2_query_buffer(&dev, rbuf).expect("Couldn't query our buffer");

        let len = rbuf.length as usize;
        let buffer: MappedDmaBuf = heap
            .allocate::<dma_buf::DmaBuf>(len)
            .unwrap()
            .memory_map()
            .unwrap();

        queue_buffer(&dev, idx, buffer.as_raw_fd()).expect("Couldn't queue our buffer");
        buffers.push(V4L2Buffer { dmabuf: buffer });
    }

    v4l2_start_streaming(&dev, BUFFER_TYPE).expect("Couldn't start streaming");

    loop {
        let idx = dequeue_buffer(&dev).expect("Couldn't dequeue our buffer");

        let buf = &buffers[idx as usize];

        let frame = buf.dmabuf.read(decode_captured_frame).unwrap();

        if frame.frame_hash == frame.computed_hash {
            info!("Frame valid");
        } else {
            warn!(
                "Frame {} corrupted, hash mismatch: retrieved {:#x} vs computed {:#x}",
                frame.index, frame.frame_hash, frame.computed_hash
            );
        }

        queue_buffer(&dev, idx as usize, buf.dmabuf.as_raw_fd())
            .expect("Couldn't queue our buffer");
    }
}
