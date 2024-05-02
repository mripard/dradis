mod library {
    pub struct Device;

    impl Device {
        pub fn new() -> Self {
            // Open /dev/video0
            Self
        }

        pub fn get_queue(&self) -> Queue {
            Queue
        }
    }

    pub struct Queue;

    impl Queue {
        pub fn request_buffers(&mut self) -> Vec<BufferImpl> {
            // Call v4l2 REQBUFS ioctl
            Vec::new()
        }

        pub fn queue_buffer(&mut self, buf: BufferImpl) {
            // Retrieve the buffer index from the app buffer type
            let idx = ...;

            // There's different Buffer Memory Types, with different arguments there
            // * DMA-Buf: Needs a RawFd (allocated by the app)
            // * MMAP: Needs a memory offset (returned by v4l2 QUERYBUF ioctl, and passed by the application) 
            // * UserPTR: Needs a pointer (allocated by the app)
            //
            // A queue will always run with a given memory type. You can't mix them, or change them halfway through the queue use without destroying the queue and creating a new one.

            // Call v4l2 QBUF ioctl
        }

        pub fn dequeue_buffer(&mut self) -> BufferImpl {
            // Call v4l2 DQBUF ioctl
            
            // We get an index from the ioctl, and we need to return the same instance of the app's Buffer type so it can store associated data.

            // Convert into BufferImpl
        }
    }

    pub struct BufferImpl;
}

use library::Device;

struct Buffer {
    inner: (), // Opaque buffer representation from the library
    data: u32,
}

impl Buffer {}

fn main() {
    let device = Device::new();

    let mut queue = device.get_queue();

    let buffers: Vec<Buffer> = queue.request_buffers();

    for buf in buffers {
        queue.queue_buffer(buf);
    }

    loop {
        let buffer = queue.dequeue_buffer();

        queue.queue_buffer(buffer);
    }
}
