#![allow(unsafe_code)]

#[cfg(target_arch = "aarch64")]
mod aarch64 {
    use core::ffi::c_void;
    use std::arch::global_asm;

    global_asm!(include_str!("./aarch64/memcpy.S"));
    global_asm!(include_str!("./aarch64/memcpy-advsimd.S"));

    extern "C" {
        pub(crate) fn __memcpy_aarch64(dst: *mut c_void, src: *const c_void, count: usize);
        pub(crate) fn __memcpy_aarch64_simd(dst: *mut c_void, src: *const c_void, count: usize);
    }

    pub(crate) fn optimized_memcpy<T>(dst: *mut T, src: *const T, count: usize) {
        if cfg!(target_feature = "neon") {
            unsafe {
                __memcpy_aarch64_simd(
                    dst.cast::<c_void>(),
                    src.cast::<c_void>(),
                    count * size_of::<T>(),
                )
            };
        } else {
            unreachable!();
            unsafe {
                __memcpy_aarch64(
                    dst.cast::<c_void>(),
                    src.cast::<c_void>(),
                    count * size_of::<T>(),
                )
            };
        }
    }
}

#[cfg(target_arch = "aarch64")]
pub(crate) use aarch64::optimized_memcpy;

#[cfg(not(target_arch = "aarch64"))]
pub(crate) fn optimized_memcpy<T>(dst: *mut T, src: *const T, count: usize) {
    unsafe {
        core::ptr::copy(src, dst, count);
    }
}
