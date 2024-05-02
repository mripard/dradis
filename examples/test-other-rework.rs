mod library {
    use std::{
        cell::{Ref, RefCell},
        marker::PhantomData,
        os::fd::AsRawFd,
        rc::Rc,
    };

    pub struct Device;

    impl Device {
        pub fn new() -> Self {
            // Open /dev/video0
            Self
        }
    }

    pub trait GetQueues<M, B>
    where
        B: NewPayload + Sized,
    {
        fn get_queue(&self) -> Queue<M, B>;
    }

    impl<M, B> GetQueues<M, B> for Device
    where
        B: NewPayload + Sized,
    {
        fn get_queue(&self) -> Queue<M, B> {
            Queue {
                buffers: Vec::new(),
                _state: PhantomData,
            }
        }
    }

    pub struct DmaBufMemory;
    pub struct UserPtrMemory;

    pub struct Queue<M, B>
    where
        B: NewPayload + Sized,
    {
        buffers: Vec<Buffer<B>>,
        _state: PhantomData<M>,
    }

    impl<M, B> Queue<M, B>
    where
        B: NewPayload + Sized,
    {
        pub fn request_buffers(&mut self, num: usize) -> Vec<Buffer<B>> {
            // Call v4l2 REQBUFS ioctl

            Vec::new()
        }

        // pub fn queue_buffer(&mut self, buf: &Buffer<B>) {
        //     // Retrieve the buffer index from the app buffer type
        //     let idx = buf.idx;

        //     // There's different Buffer Memory Types, with different arguments there
        //     // * DMA-Buf: Needs a RawFd (allocated by the app)
        //     // * MMAP: Needs a memory offset (returned by v4l2 QUERYBUF ioctl, and passed by the application)
        //     // * UserPTR: Needs a pointer (allocated by the app)
        //     //
        //     // A queue will always run with a given memory type. You can't mix them, or change them halfway through the queue use without destroying the queue and creating a new one.

        //     // Call v4l2 QBUF ioctl
        // }

        // pub fn dequeue_buffer(&mut self) -> &Buffer<B> {
        //     // Call v4l2 DQBUF ioctl

        //     // We get an index from the ioctl, and we need to return the same instance of the app's Buffer type so it can store associated data.

        //     // Convert into Buffer
        //     &self.buffers[0]
        // }
    }

    impl<B> Queue<DmaBufMemory, B>
    where
        B: NewPayload + Sized,
    {
        pub fn queue_buffer(&mut self, buf: Buffer<B>, fd: &impl AsRawFd) {
            let idx = buf.0.borrow().idx;

            unimplemented!();
        }

        pub fn dequeue_buffer(&mut self) -> Buffer<B> {
            unimplemented!()
        }
    }

    impl<B> Queue<UserPtrMemory, B>
    where
        B: NewPayload + Sized,
    {
        pub fn queue_buffer(&mut self, buf: Buffer<B>, ptr: *mut u8) {
            let idx = buf.0.borrow().idx;

            unimplemented!();
        }

        pub fn dequeue_buffer(&mut self) -> Buffer<B> {
            unimplemented!()
        }
    }

    pub trait NewPayload {
        fn new() -> Self;
    }

    impl<T> NewPayload for T
    where
        T: Default,
    {
        fn new() -> T {
            T::default()
        }
    }

    // Buffer struct / Trait
    struct BufferInner<T>
    where
        T: NewPayload + Sized,
    {
        idx: usize,
        pl: T,
    }

    pub struct Buffer<T>(Rc<RefCell<BufferInner<T>>>)
    where
        T: NewPayload + Sized;

    impl<T> Buffer<T>
    where
        T: NewPayload + Sized,
    {
        pub fn payload(&self) -> Ref<'_, T> {
            Ref::map(self.0.borrow(), |inner| &inner.pl)
        }

        pub fn set_payload(self, val: T) -> Self {
            RefCell::borrow_mut(&self.0).pl = val;
            self
        }
    }
}

use std::os::fd::{AsFd, FromRawFd, OwnedFd};

use library::{Device, DmaBufMemory, GetQueues, NewPayload, Queue};

struct BufferMetadata {
    fd: OwnedFd,
}

impl NewPayload for BufferMetadata {
    fn new() -> Self {
        Self {
            fd: unsafe { OwnedFd::from_raw_fd(0) },
        }
    }
}

fn main() {
    let device = Device::new();

    let mut queue: Queue<DmaBufMemory, BufferMetadata> = device.get_queue();

    let buffers = queue.request_buffers(5);

    for buffer in buffers {
        let payload = buffer.payload();
        queue.queue_buffer(buffer, &payload.fd);
    }

    loop {
        let buffer = queue.dequeue_buffer();

        let payload = buffer.payload();
        let arg_fd = payload.fd.as_fd().clone();
        drop(payload);

        queue.queue_buffer(buffer, &arg_fd);
    }
}
