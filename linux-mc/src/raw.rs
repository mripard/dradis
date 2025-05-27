use std::{io, os::fd::BorrowedFd};

use rustix::{
    io::Errno,
    ioctl::{Updater, ioctl, opcode},
};

pub(crate) mod bindgen {
    #![allow(clippy::decimal_literal_representation)]
    #![allow(clippy::multiple_inherent_impl)]
    #![allow(clippy::multiple_unsafe_ops_per_block)]
    #![allow(clippy::pub_underscore_fields)]
    #![allow(clippy::std_instead_of_alloc)]
    #![allow(clippy::std_instead_of_core)]
    #![allow(clippy::type_complexity)]
    #![allow(clippy::undocumented_unsafe_blocks)]
    #![allow(clippy::unreadable_literal)]
    #![allow(dead_code)]
    #![allow(missing_debug_implementations)]
    #![allow(missing_docs)]
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unreachable_pub)]
    #![allow(unsafe_code)]

    use core::fmt;

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    impl fmt::Debug for media_v2_interface {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let id = self.id;
            let intf_type = self.intf_type;
            let flags = self.flags;
            f.debug_struct("media_v2_interface")
                .field("id", &id)
                .field("intf_type", &intf_type)
                .field("flags", &flags)
                .field("devnode", unsafe { &self.__bindgen_anon_1.devnode })
                .finish_non_exhaustive()
        }
    }
}

const MEDIA_IOC_MAGIC: u8 = b'|';
const MEDIA_IOC_DEVICE_INFO: u8 = 0x00;
const MEDIA_IOC_G_TOPOLOGY: u8 = 0x04;

pub use bindgen::media_device_info;
const MEDIA_IOC_DEVICE_INFO_OPCODE: u32 =
    opcode::read_write::<media_device_info>(MEDIA_IOC_MAGIC, MEDIA_IOC_DEVICE_INFO);

/// Queries Device Information
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
pub fn media_ioctl_device_info(fd: BorrowedFd<'_>) -> io::Result<media_device_info> {
    let mut info = media_device_info::default();

    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj =
        unsafe { Updater::<MEDIA_IOC_DEVICE_INFO_OPCODE, media_device_info>::new(&mut info) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| info)
        .map_err(<Errno as Into<io::Error>>::into)
}

pub use bindgen::{
    media_v2_entity, media_v2_interface, media_v2_link, media_v2_pad, media_v2_topology,
};
const MEDIA_IOC_G_TOPOLOGY_OPCODE: u32 =
    opcode::read_write::<media_v2_topology>(MEDIA_IOC_MAGIC, MEDIA_IOC_G_TOPOLOGY);

/// Enumerates the graph topology and graph element properties
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
pub fn media_ioctl_g_topology(
    fd: BorrowedFd<'_>,
    mut topo: media_v2_topology,
) -> io::Result<media_v2_topology> {
    // SAFETY: We checked both the opcode and the type.
    let ioctl_obj =
        unsafe { Updater::<MEDIA_IOC_G_TOPOLOGY_OPCODE, media_v2_topology>::new(&mut topo) };

    // SAFETY: This function is unsafe because the driver isn't guaranteed to implement the ioctl
    // properly. We don't have much of a choice and still have to trust the
    // kernel there.
    unsafe { ioctl(fd, ioctl_obj) }
        .map(|()| topo)
        .map_err(<Errno as Into<io::Error>>::into)
}
