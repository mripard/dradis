use std::{
    os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd},
    path::PathBuf,
    sync::Arc,
};

use v4l2r::{
    device::{
        queue::{
            qbuf::{get_free::GetFreeCaptureBuffer, CaptureQueueable},
            Queue,
        },
        AllocatedQueue, Device, DeviceConfig, Stream, TryDequeue,
    },
    memory::{DmaBufHandle, DmaBufSource, Memory, MemoryType, SelfBacked},
    Format,
};

const NUM_BUFFERS: u32 = 5;

#[derive(Debug, Default)]
pub struct Buffer;

impl AsFd for Buffer {
    fn as_fd(&self) -> BorrowedFd<'_> {
        todo!()
    }
}

impl AsRawFd for Buffer {
    fn as_raw_fd(&self) -> RawFd {
        todo!()
    }
}

impl DmaBufSource for Buffer {
    fn len(&self) -> u64 {
        todo!()
    }
}

fn main() {
    let path = PathBuf::from("/dev/video0");

    let device = Device::open(&path, DeviceConfig::new().non_blocking_dqbuf()).unwrap();

    let mut queue = Queue::get_capture_queue(Arc::new(device)).unwrap();

    // s_edid
    // query_dv_timings

    let format: Format = queue
        .change_format()
        .unwrap()
        .set_pixelformat(b"RGB3")
        .set_size(1280, 720)
        .apply()
        .unwrap();

    let queue = queue
        .request_buffers::<Vec<DmaBufHandle<Buffer>>>(NUM_BUFFERS)
        .unwrap();

    assert_eq!(queue.num_buffers(), NUM_BUFFERS as usize);

    // subscribe_events

    // FIXME: Don't we need to queue the buffers before starting the stream?
    queue.stream_on().unwrap();

    for _idx in 0..NUM_BUFFERS {
        let buffer = queue.try_get_free_buffer().unwrap();

        buffer.queue_with_handles(vec![Buffer.into()]).unwrap();
    }

    loop {
        // Dequeue and Handle Events

        let _frame = queue.try_dequeue().unwrap();

        // Do stuff

        // Requeue the buffer
    }
}
