use core::{fmt, str::FromStr as _};
use std::{io, os::fd::BorrowedFd};

use linux_raw::KernelVersion;
use rustix::time::Timespec;
use tracing::instrument;

use crate::{
    ConversionError,
    format::{v4l2_mbus_pixelcode, v4l2_pix_fmt},
    raw::{
        self, V4L2_EVENT_ALL, V4L2_EVENT_CTRL, V4L2_EVENT_CTRL_CH_DIMENSIONS,
        V4L2_EVENT_CTRL_CH_FLAGS, V4L2_EVENT_CTRL_CH_RANGE, V4L2_EVENT_CTRL_CH_VALUE,
        V4L2_EVENT_EOS, V4L2_EVENT_FRAME_SYNC, V4L2_EVENT_MD_FL_HAVE_FRAME_SEQ,
        V4L2_EVENT_MOTION_DET, V4L2_EVENT_SOURCE_CHANGE, V4L2_EVENT_SRC_CH_RESOLUTION,
        V4L2_EVENT_VSYNC,
    },
    v4l2_buf_type, v4l2_colorspace, v4l2_field, v4l2_hsv_encoding, v4l2_memory, v4l2_quantization,
    v4l2_xfer_func, v4l2_ycbcr_encoding,
};

/// V4L2 Colorspace Encoding
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum v4l2_encoding {
    /// YCbCr Colorspace Encoding
    YCbCr(v4l2_ycbcr_encoding),

    /// Hue/Saturation/Value Encoding
    Hsv(v4l2_hsv_encoding),
}

impl fmt::Display for v4l2_encoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            match self {
                v4l2_encoding::YCbCr(e) => format!("YCbCr/{e}"),
                v4l2_encoding::Hsv(e) => format!("Hue/{e}"),
            }
            .as_str(),
        )
    }
}

impl TryFrom<u32> for v4l2_encoding {
    type Error = ConversionError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > 128 {
            let max_discriminant_value = v4l2_hsv_encoding::V4L2_HSV_ENC_256 as u32;

            if value <= max_discriminant_value {
                Ok(Self::Hsv(
                    // SAFETY: We know that all values between 128 and V4L2_HSV_ENC_256 are valid
                    // enum values, so we can just transmute from u32 into the enum.
                    unsafe { core::mem::transmute::<u32, v4l2_hsv_encoding>(value) },
                ))
            } else {
                Err(Self::Error::InvalidValue(format!("{value:#?}")))
            }
        } else {
            let max_discriminant_value = v4l2_ycbcr_encoding::V4L2_YCBCR_ENC_SMPTE240M as u32;

            if value <= max_discriminant_value {
                Ok(Self::YCbCr(
                    // SAFETY: We know that all values between 0 and V4L2_YCBCR_ENC_SMPTE240M are
                    // valid enum values, so we can just transmute from u32
                    // into the enum.
                    unsafe { core::mem::transmute::<u32, v4l2_ycbcr_encoding>(value) },
                ))
            } else {
                Err(Self::Error::InvalidValue(format!("{value:#?}")))
            }
        }
    }
}

impl From<v4l2_encoding> for u32 {
    fn from(value: v4l2_encoding) -> Self {
        match value {
            v4l2_encoding::YCbCr(e) => e as Self,
            v4l2_encoding::Hsv(e) => e as Self,
        }
    }
}

impl From<v4l2_encoding> for u16 {
    fn from(value: v4l2_encoding) -> Self {
        let value = u32::from(value);

        value
            .try_into()
            .expect("v4l2_encoding values always fit into a u16")
    }
}

impl From<v4l2_encoding> for u8 {
    fn from(value: v4l2_encoding) -> Self {
        let value = u32::from(value);

        value
            .try_into()
            .expect("v4l2_encoding values always fit into a u8")
    }
}

/// Single-Planar Data Format
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct v4l2_pix_format {
    width: u32,
    height: u32,
    pixelformat: v4l2_pix_fmt,
    field: v4l2_field,
    bytesperline: u32,
    sizeimage: u32,
    colorspace: v4l2_colorspace,
    private: u32,
    flags: u32,
    encoding: u32,
    quantization: v4l2_quantization,
    xfer_func: v4l2_xfer_func,
}

impl v4l2_pix_format {
    /// Sets the image colorspace.
    ///
    /// This function is only effective for output streams. The colorspace will be ignored and set
    /// by the driver for capture streams.
    #[must_use]
    pub fn set_colorspace(mut self, colorspace: v4l2_colorspace) -> Self {
        self.colorspace = colorspace;
        self
    }

    /// Sets the image color encoding.
    ///
    /// This function is only effective for output streams. The colorspace will be ignored and set
    /// by the driver for capture streams.
    #[must_use]
    pub fn set_encoding(mut self, enc: v4l2_encoding) -> Self {
        self.encoding = enc.into();
        self
    }

    /// Sets the image field order.
    #[must_use]
    pub fn set_field(mut self, field: v4l2_field) -> Self {
        self.field = field;
        self
    }

    /// Sets the image height, in pixels.
    #[must_use]
    pub fn set_height(mut self, height: u32) -> Self {
        self.height = height;
        self
    }

    /// Sets the image pixel format.
    #[must_use]
    pub fn set_pixel_format(mut self, fmt: v4l2_pix_fmt) -> Self {
        self.pixelformat = fmt;
        self
    }

    /// Sets the image quantization.
    ///
    /// This function is only effective for output streams. The colorspace will be ignored and set
    /// by the driver for capture streams.
    #[must_use]
    pub fn set_quantization(mut self, quant: v4l2_quantization) -> Self {
        self.quantization = quant;
        self
    }

    /// Sets the image width, in pixels.
    #[must_use]
    pub fn set_width(mut self, width: u32) -> Self {
        self.width = width;
        self
    }

    /// Sets the image colorspace transfer function.
    ///
    /// This function is only effective for output streams. The colorspace will be ignored and set
    /// by the driver for capture streams.
    #[must_use]
    pub fn set_xfer_func(mut self, func: v4l2_xfer_func) -> Self {
        self.xfer_func = func;
        self
    }
}

impl TryFrom<raw::v4l2_pix_format> for v4l2_pix_format {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_pix_format) -> Result<Self, Self::Error> {
        Ok(Self {
            width: value.width,
            height: value.height,
            pixelformat: value.pixelformat.try_into()?,
            field: value.field.try_into()?,
            bytesperline: value.bytesperline,
            sizeimage: value.sizeimage,
            colorspace: value.colorspace.try_into()?,
            private: value.priv_,
            flags: value.flags,
            encoding: {
                // Because the encoding changes representation between the various structures, we
                // can't really have one that would have the same layout than all the users. Let's
                // try to convert it into a valid enum first, and once we know it's valid, we can
                // store it as the raw underlying representation.
                //
                // SAFETY: Both sides of the union have values that don't overlap, and we are
                // able to deal with all of these values, so we can just use any of the union
                // fields as a u32.
                let enc: v4l2_encoding = unsafe { value.__bindgen_anon_1.ycbcr_enc }.try_into()?;

                enc.into()
            },
            quantization: value.quantization.try_into()?,
            xfer_func: value.xfer_func.try_into()?,
        })
    }
}

impl From<v4l2_pix_format> for raw::v4l2_pix_format {
    fn from(value: v4l2_pix_format) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_pix_format, Self>(value) }
    }
}

#[cfg(test)]
mod tests_v4l2_pix_format {
    use crate::{
        format::v4l2_pix_fmt,
        raw::{
            self, v4l2_colorspace, v4l2_field, v4l2_pix_format_encoding, v4l2_quantization,
            v4l2_xfer_func,
        },
        v4l2_fourcc, wrapper,
    };

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_pix_format>(),
            size_of::<raw::v4l2_pix_format>()
        );

        assert_eq!(
            align_of::<wrapper::v4l2_pix_format>(),
            align_of::<raw::v4l2_pix_format>()
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, width),
            std::mem::offset_of!(raw::v4l2_pix_format, width)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, height),
            std::mem::offset_of!(raw::v4l2_pix_format, height)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, pixelformat),
            std::mem::offset_of!(raw::v4l2_pix_format, pixelformat)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, field),
            std::mem::offset_of!(raw::v4l2_pix_format, field)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, bytesperline),
            std::mem::offset_of!(raw::v4l2_pix_format, bytesperline)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, sizeimage),
            std::mem::offset_of!(raw::v4l2_pix_format, sizeimage)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, colorspace),
            std::mem::offset_of!(raw::v4l2_pix_format, colorspace)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, private),
            std::mem::offset_of!(raw::v4l2_pix_format, priv_)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, flags),
            std::mem::offset_of!(raw::v4l2_pix_format, flags)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, encoding),
            std::mem::offset_of!(raw::v4l2_pix_format, __bindgen_anon_1)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, quantization),
            std::mem::offset_of!(raw::v4l2_pix_format, quantization)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_pix_format, xfer_func),
            std::mem::offset_of!(raw::v4l2_pix_format, xfer_func)
        );
    }

    #[test]
    fn convert() {
        assert_eq!(
            wrapper::v4l2_pix_format {
                width: 1280,
                height: 720,
                pixelformat: v4l2_pix_fmt::V4L2_PIX_FMT_RGB24,
                field: v4l2_field::V4L2_FIELD_NONE,
                bytesperline: 3840,
                sizeimage: 2764800,
                colorspace: v4l2_colorspace::V4L2_COLORSPACE_SRGB,
                private: 0,
                flags: 0,
                encoding: 1,
                quantization: v4l2_quantization::V4L2_QUANTIZATION_LIM_RANGE,
                xfer_func: v4l2_xfer_func::V4L2_XFER_FUNC_SRGB,
            },
            raw::v4l2_pix_format {
                width: 1280,
                height: 720,
                pixelformat: v4l2_fourcc!('R', 'G', 'B', '3'),
                field: 1,
                bytesperline: 3840,
                sizeimage: 2764800,
                colorspace: 8,
                priv_: 0,
                flags: 0,
                __bindgen_anon_1: v4l2_pix_format_encoding { ycbcr_enc: 1 },
                quantization: 2,
                xfer_func: 2
            }
            .try_into()
            .unwrap()
        );
    }
}

/// V4L2 Stream Data Format
///
/// All directions are from the system point of view, ie. capture means the system receives frames,
/// output that it emits them.
#[repr(C, u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
#[expect(
    variant_size_differences,
    reason = "We don't choose the size of that enum and its variants."
)]
pub enum v4l2_format {
    /// Single-Planar Video Capture
    VideoCapture(v4l2_pix_format) = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE as u32,

    /// Single-Planar Video Output
    VideoOutput(v4l2_pix_format) = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT as u32,

    #[doc(hidden)]
    VideoOverlay(raw::v4l2_window) = v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OVERLAY as u32,

    #[doc(hidden)]
    _Raw([u8; 200]),
}

impl TryFrom<raw::v4l2_format> for v4l2_format {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_format) -> Result<Self, Self::Error> {
        let kind =
            v4l2_buf_type::try_from(value.type_).map_err(|_e| Self::Error::InvalidStructField {
                name: String::from("type"),
                value: format!("{}", value.type_),
            })?;
        match kind {
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE => Ok(Self::VideoCapture({
                // SAFETY: We just checked the union tag, we know we access the right part of it.
                let fmt = unsafe { value.fmt.pix };

                fmt.try_into()
                    .map_err(|_e| Self::Error::InvalidStructField {
                        name: String::from("pix"),
                        value: format!("{fmt:#?}"),
                    })?
            })),
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT => Ok(Self::VideoOutput({
                // SAFETY: We just checked the union tag, we know we access the right part of it.
                let fmt = unsafe { value.fmt.pix };

                fmt.try_into()
                    .map_err(|_e| Self::Error::InvalidStructField {
                        name: String::from("pix"),
                        value: format!("{fmt:#?}"),
                    })?
            })),
            v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OVERLAY
            | v4l2_buf_type::V4L2_BUF_TYPE_VBI_CAPTURE
            | v4l2_buf_type::V4L2_BUF_TYPE_VBI_OUTPUT
            | v4l2_buf_type::V4L2_BUF_TYPE_SLICED_VBI_CAPTURE
            | v4l2_buf_type::V4L2_BUF_TYPE_SLICED_VBI_OUTPUT
            | v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT_OVERLAY
            | v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE_MPLANE
            | v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_OUTPUT_MPLANE
            | v4l2_buf_type::V4L2_BUF_TYPE_SDR_CAPTURE
            | v4l2_buf_type::V4L2_BUF_TYPE_SDR_OUTPUT
            | v4l2_buf_type::V4L2_BUF_TYPE_META_CAPTURE
            | v4l2_buf_type::V4L2_BUF_TYPE_META_OUTPUT
            | v4l2_buf_type::V4L2_BUF_TYPE_PRIVATE => unimplemented!(),
        }
    }
}

impl From<v4l2_format> for raw::v4l2_format {
    fn from(value: v4l2_format) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_format, Self>(value) }
    }
}

#[cfg(test)]
mod tests_v4l2_format {
    use crate::{
        format::v4l2_pix_fmt,
        raw::{
            self, v4l2_buf_type, v4l2_colorspace, v4l2_field, v4l2_pix_format_encoding,
            v4l2_quantization, v4l2_xfer_func,
        },
        v4l2_fourcc, wrapper,
    };

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_format>(),
            size_of::<raw::v4l2_format>(),
            concat!("Size of: ", stringify!(v4l2_format))
        );

        assert_eq!(
            align_of::<wrapper::v4l2_format>(),
            align_of::<raw::v4l2_format>(),
            concat!("Alignment of ", stringify!(v4l2_format))
        );
    }

    #[test]
    fn convert() {
        assert_eq!(
            wrapper::v4l2_format::VideoCapture(wrapper::v4l2_pix_format {
                width: 1280,
                height: 720,
                pixelformat: v4l2_pix_fmt::V4L2_PIX_FMT_RGB24,
                field: v4l2_field::V4L2_FIELD_NONE,
                bytesperline: 3840,
                sizeimage: 2764800,
                colorspace: v4l2_colorspace::V4L2_COLORSPACE_SRGB,
                private: 0,
                flags: 0,
                encoding: 1,
                quantization: v4l2_quantization::V4L2_QUANTIZATION_LIM_RANGE,
                xfer_func: v4l2_xfer_func::V4L2_XFER_FUNC_SRGB,
            }),
            raw::v4l2_format {
                type_: v4l2_buf_type::V4L2_BUF_TYPE_VIDEO_CAPTURE as u32,
                fmt: raw::v4l2_format_content {
                    pix: raw::v4l2_pix_format {
                        width: 1280,
                        height: 720,
                        pixelformat: v4l2_fourcc!('R', 'G', 'B', '3'),
                        field: 1,
                        bytesperline: 3840,
                        sizeimage: 2764800,
                        colorspace: 8,
                        priv_: 0,
                        flags: 0,
                        __bindgen_anon_1: v4l2_pix_format_encoding { ycbcr_enc: 1 },
                        quantization: 2,
                        xfer_func: 2
                    }
                }
            }
            .try_into()
            .unwrap()
        );
    }
}

/// Gets the current data format.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
#[instrument(level = "trace")]
pub fn v4l2_ioctl_g_fmt(fd: BorrowedFd<'_>, kind: v4l2_buf_type) -> io::Result<v4l2_format> {
    let fmt = raw::v4l2_format {
        type_: kind.into(),
        ..Default::default()
    };

    raw::v4l2_ioctl_g_fmt(fd, fmt).map(|f| {
        f.try_into()
            .expect("The kernel returned an unexpected type")
    })
}

/// Sets the data format.
///
/// The driver can adjust the format to accomodate hardware limitations. The actual format used by
/// the driver will be returned.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
#[instrument(level = "trace")]
pub fn v4l2_ioctl_s_fmt(fd: BorrowedFd<'_>, fmt: v4l2_format) -> io::Result<v4l2_format> {
    raw::v4l2_ioctl_s_fmt(fd, fmt.into()).map(|f| {
        f.try_into()
            .expect("The kernel returned an unexpected type")
    })
}

/// Tries to the data format.
///
/// The driver can adjust the format to accomodate hardware limitations. The format the driver would
/// use will be returned.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
#[instrument(level = "trace")]
pub fn v4l2_ioctl_try_fmt(fd: BorrowedFd<'_>, fmt: v4l2_format) -> io::Result<v4l2_format> {
    raw::v4l2_ioctl_try_fmt(fd, fmt.into()).map(|f| {
        f.try_into()
            .expect("The kernel returned an unexpected type")
    })
}

/// Frame format on the media bus
#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub struct v4l2_mbus_framefmt {
    width: u32,
    height: u32,
    code: v4l2_mbus_pixelcode,
    field: v4l2_field,
    colorspace: v4l2_colorspace,
    encoding: u16,
    quantization: u16,
    xfer_func: u16,
    flags: u16,
    _reserved: [u16; 10],
}

impl TryFrom<raw::v4l2_mbus_framefmt> for v4l2_mbus_framefmt {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_mbus_framefmt) -> Result<Self, Self::Error> {
        Ok(Self {
            width: value.width,
            height: value.height,
            code: value.code.try_into()?,
            field: value.field.try_into()?,
            colorspace: value.colorspace.try_into()?,
            encoding: {
                // Because the encoding changes representation between the various structures, we
                // can't really have one that would have the same layout than all the users. Let's
                // try to convert it into a valid enum first, and once we know it's valid, we can
                // store it as the raw underlying representation.
                //
                // SAFETY: Both sides of the union have values that don't overlap, and we are
                // able to deal with all of these values, so we can just use any of the union
                // fields as a u32.
                let enc_raw: u32 = unsafe { value.__bindgen_anon_1.ycbcr_enc }.into();

                let enc: v4l2_encoding = enc_raw.try_into()?;

                enc.into()
            },
            quantization: {
                // Because the quantization has a smaller representation than the generated enum, we
                // can't use it directly. Let's
                // try to convert it into a valid enum first, and once we know it's valid, we can
                // store it as the raw underlying representation.
                let quant_raw: u32 = value.quantization.into();
                let quant: v4l2_quantization = quant_raw.try_into()?;

                quant.into()
            },
            xfer_func: {
                // Because the xfer_func has a smaller representation than the generated enum, we
                // can't use it directly. Let's
                // try to convert it into a valid enum first, and once we know it's valid, we can
                // store it as the raw underlying representation.
                let xfer_raw: u32 = value.xfer_func.into();
                let xfer: v4l2_xfer_func = xfer_raw.try_into()?;

                xfer.into()
            },
            flags: value.flags,
            _reserved: [0; 10],
        })
    }
}

impl TryFrom<v4l2_pix_format> for v4l2_mbus_framefmt {
    type Error = ConversionError;

    fn try_from(value: v4l2_pix_format) -> Result<Self, Self::Error> {
        Ok(Self {
            width: value.width,
            height: value.height,
            code: value.pixelformat.try_into()?,
            field: value.field,
            colorspace: value.colorspace,
            encoding: {
                // Because the encoding changes representation between the various structures, we
                // can't really have one that would have the same layout than all the users. Let's
                // try to convert it into a valid enum first, and once we know it's valid, we can
                // store it as the raw underlying representation.
                let enc_raw: u32 = value.encoding;
                let enc: v4l2_encoding = enc_raw.try_into()?;

                enc.into()
            },
            quantization: value.quantization.into(),
            xfer_func: value.xfer_func.into(),
            flags: 0, // FIXME: Carry over the flags
            _reserved: [0; 10],
        })
    }
}

impl fmt::Debug for v4l2_mbus_framefmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("v4l2_mbus_framefmt")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("code", &self.code)
            .field("field", &self.field)
            .field("colorspace", &self.colorspace)
            .field("encoding", &self.encoding)
            .field("quantization", &self.quantization)
            .field("xfer_func", &self.xfer_func)
            .field("flags", &self.flags)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for v4l2_mbus_framefmt {
    #[expect(
        clippy::unwrap_in_result,
        reason = "We know that the conversions won't fail."
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}/{}x{}, field: {}, colorspace: {}, xfer: {}, quantization: {}, enc: {}, flags: {:x}",
            self.code,
            self.width,
            self.height,
            self.field,
            self.colorspace,
            {
                v4l2_xfer_func::try_from(u32::from(self.xfer_func))
                    .expect("We know the content of the variable is a valid variant by now.")
            },
            {
                v4l2_quantization::try_from(u32::from(self.quantization))
                    .expect("We know the content of the variable is a valid variant by now.")
            },
            {
                v4l2_encoding::try_from(u32::from(self.encoding))
                    .expect("We know the content of the variable is a valid variant by now.")
            },
            self.flags,
        ))
    }
}

impl Default for v4l2_mbus_framefmt {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            code: v4l2_mbus_pixelcode::V4L2_MBUS_FMT_FIXED,
            field: v4l2_field::V4L2_FIELD_ANY,
            colorspace: v4l2_colorspace::V4L2_COLORSPACE_DEFAULT,
            encoding: v4l2_encoding::YCbCr(v4l2_ycbcr_encoding::V4L2_YCBCR_ENC_DEFAULT).into(),
            quantization: v4l2_quantization::V4L2_QUANTIZATION_DEFAULT.into(),
            xfer_func: v4l2_xfer_func::V4L2_XFER_FUNC_DEFAULT.into(),
            flags: 0,
            _reserved: [0; 10],
        }
    }
}

#[cfg(test)]
mod tests_v4l2_mbus_framefmt {
    use crate::{raw, wrapper};

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_mbus_framefmt>(),
            size_of::<raw::v4l2_mbus_framefmt>()
        );

        assert_eq!(
            align_of::<wrapper::v4l2_mbus_framefmt>(),
            align_of::<raw::v4l2_mbus_framefmt>()
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_mbus_framefmt, width),
            std::mem::offset_of!(raw::v4l2_mbus_framefmt, width)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_mbus_framefmt, height),
            std::mem::offset_of!(raw::v4l2_mbus_framefmt, height)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_mbus_framefmt, code),
            std::mem::offset_of!(raw::v4l2_mbus_framefmt, code)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_mbus_framefmt, field),
            std::mem::offset_of!(raw::v4l2_mbus_framefmt, field)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_mbus_framefmt, colorspace),
            std::mem::offset_of!(raw::v4l2_mbus_framefmt, colorspace)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_mbus_framefmt, encoding),
            std::mem::offset_of!(raw::v4l2_mbus_framefmt, __bindgen_anon_1)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_mbus_framefmt, quantization),
            std::mem::offset_of!(raw::v4l2_mbus_framefmt, quantization)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_mbus_framefmt, xfer_func),
            std::mem::offset_of!(raw::v4l2_mbus_framefmt, xfer_func)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_mbus_framefmt, flags),
            std::mem::offset_of!(raw::v4l2_mbus_framefmt, flags)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_mbus_framefmt, _reserved),
            std::mem::offset_of!(raw::v4l2_mbus_framefmt, reserved)
        );
    }
}

/// Sub-device Frame Format
#[repr(C)]
#[derive(Copy, Clone, PartialEq)]
pub struct v4l2_subdev_format {
    which: raw::v4l2_subdev_format_whence,
    pad: u32,
    format: v4l2_mbus_framefmt,
    stream: u32,
    _reserved: [u32; 7],
}

impl v4l2_subdev_format {
    /// Creates a new [`v4l2_subdev_format`] to apply to the hardware
    #[must_use]
    pub fn new_active() -> Self {
        Self {
            which: raw::v4l2_subdev_format_whence::V4L2_SUBDEV_FORMAT_ACTIVE,
            pad: 0,
            format: v4l2_mbus_framefmt::default(),
            stream: 0,
            _reserved: [0; 7],
        }
    }

    /// Creates a new [`v4l2_subdev_format`] structure to try on the sub-device
    #[must_use]
    pub fn new_try() -> Self {
        Self {
            which: raw::v4l2_subdev_format_whence::V4L2_SUBDEV_FORMAT_TRY,
            pad: 0,
            format: v4l2_mbus_framefmt::default(),
            stream: 0,
            _reserved: [0; 7],
        }
    }

    /// Sets the mediabus frame format
    #[must_use]
    pub fn set_format(mut self, fmt: v4l2_mbus_framefmt) -> Self {
        self.format = fmt;
        self
    }

    /// Sets the sub-device pad this format applies to.
    #[must_use]
    pub fn set_pad(mut self, pad: u32) -> Self {
        self.pad = pad;
        self
    }

    /// Sets the sub-device stream this format applies to.
    ///
    /// # Panics
    ///
    /// If streams aren't supported by the kernel we're running on.
    #[must_use]
    pub fn set_stream(mut self, stream: u32) -> Self {
        let uname = rustix::system::uname();
        let version_str = uname
            .release()
            .to_str()
            .expect("The kernel release name is always in ASCII.");

        let version = KernelVersion::from_str(version_str)
            .expect("The version comes straight from uname. It's valid.");

        assert!(
            version >= KernelVersion::new(6, 3, 0),
            "Streams are not supported on this platform"
        );

        // FIXME: Streams got introduced with Linux 6.3
        self.stream = stream;
        self
    }
}

impl TryFrom<raw::v4l2_subdev_format> for v4l2_subdev_format {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_subdev_format) -> Result<Self, Self::Error> {
        Ok(Self {
            which: raw::v4l2_subdev_format_whence::try_from(value.which)?,
            pad: value.pad,
            format: v4l2_mbus_framefmt::try_from(value.format)?,
            stream: value.stream,
            _reserved: [0; 7],
        })
    }
}

impl From<v4l2_subdev_format> for raw::v4l2_subdev_format {
    fn from(value: v4l2_subdev_format) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_subdev_format, Self>(value) }
    }
}

impl fmt::Debug for v4l2_subdev_format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("v4l2_subdev_format")
            .field("which", &self.which)
            .field("pad", &self.pad)
            .field("format", &self.format)
            .field("stream", &self.stream)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for v4l2_subdev_format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "pad: {}, stream: {}, which: {}, fmt: {}",
            self.pad, self.stream, self.which, self.format
        ))
    }
}

#[cfg(test)]
mod tests_v4l2_subdev_format {
    use crate::{raw, wrapper};

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_subdev_format>(),
            size_of::<raw::v4l2_subdev_format>()
        );

        assert_eq!(
            align_of::<wrapper::v4l2_subdev_format>(),
            align_of::<raw::v4l2_subdev_format>()
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_subdev_format, which),
            std::mem::offset_of!(raw::v4l2_subdev_format, which)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_subdev_format, pad),
            std::mem::offset_of!(raw::v4l2_subdev_format, pad)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_subdev_format, format),
            std::mem::offset_of!(raw::v4l2_subdev_format, format)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_subdev_format, stream),
            std::mem::offset_of!(raw::v4l2_subdev_format, stream)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_subdev_format, _reserved),
            std::mem::offset_of!(raw::v4l2_subdev_format, reserved)
        );
    }
}

/// Sets the given format on the given sub-device file descriptor.
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
#[instrument(level = "trace")]
pub fn v4l2_ioctl_subdev_s_fmt(
    fd: BorrowedFd<'_>,
    fmt: v4l2_subdev_format,
) -> io::Result<v4l2_subdev_format> {
    raw::v4l2_ioctl_subdev_s_fmt(fd, fmt.into()).map(|f| {
        f.try_into()
            .expect("The kernel returned an unexpected type")
    })
}

/// Reqbufs main structure
#[repr(C)]
#[derive(Clone, Copy)]
pub struct v4l2_requestbuffers {
    count: u32,
    kind: v4l2_buf_type,
    memory: v4l2_memory,
    caps: u32,
    flags: u8,
    _reserved: [u8; 3],
}

impl v4l2_requestbuffers {
    /// Creates a new structure
    #[must_use]
    pub fn new(kind: v4l2_buf_type, memory: v4l2_memory) -> Self {
        Self {
            count: 0,
            kind,
            memory,
            caps: 0,
            flags: 0,
            _reserved: [0; 3],
        }
    }

    /// Sets the buffer count
    #[must_use]
    pub fn set_count(mut self, count: u32) -> Self {
        self.count = count;
        self
    }
}

impl fmt::Debug for v4l2_requestbuffers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("v4l2_requestbuffers")
            .field("count", &self.count)
            .field("type", &self.kind)
            .field("memory", &self.memory)
            .field("capabilities", &self.caps)
            .field("flags", &self.flags)
            .finish_non_exhaustive()
    }
}

impl TryFrom<raw::v4l2_requestbuffers> for v4l2_requestbuffers {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_requestbuffers) -> Result<Self, Self::Error> {
        Ok(Self {
            count: value.count,
            kind: v4l2_buf_type::try_from(value.type_)?,
            memory: v4l2_memory::try_from(value.memory)?,
            caps: value.capabilities,
            flags: value.flags,
            _reserved: [0; 3],
        })
    }
}

impl From<v4l2_requestbuffers> for raw::v4l2_requestbuffers {
    fn from(value: v4l2_requestbuffers) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_requestbuffers, Self>(value) }
    }
}

#[cfg(test)]
mod tests_v4l2_requestbuffers {
    use crate::{raw, wrapper};

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_requestbuffers>(),
            size_of::<raw::v4l2_requestbuffers>()
        );

        assert_eq!(
            align_of::<wrapper::v4l2_requestbuffers>(),
            align_of::<raw::v4l2_requestbuffers>()
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_requestbuffers, count),
            std::mem::offset_of!(raw::v4l2_requestbuffers, count)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_requestbuffers, kind),
            std::mem::offset_of!(raw::v4l2_requestbuffers, type_)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_requestbuffers, memory),
            std::mem::offset_of!(raw::v4l2_requestbuffers, memory)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_requestbuffers, caps),
            std::mem::offset_of!(raw::v4l2_requestbuffers, capabilities)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_requestbuffers, flags),
            std::mem::offset_of!(raw::v4l2_requestbuffers, flags)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_requestbuffers, _reserved),
            std::mem::offset_of!(raw::v4l2_requestbuffers, reserved)
        );
    }
}

/// Allocates Device Buffers.
///
/// # Errors
///
/// If there's an I/O Error while accessing the given file descriptor
#[instrument(level = "trace")]
pub fn v4l2_ioctl_reqbufs(
    fd: BorrowedFd<'_>,
    reqbufs: v4l2_requestbuffers,
) -> io::Result<v4l2_requestbuffers> {
    raw::v4l2_ioctl_reqbufs(fd, reqbufs.into()).map(|f| {
        f.try_into()
            .expect("The kernel returned an unexpected type")
    })
}

/// Starts Streaming I/O
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
#[instrument(level = "trace")]
pub fn v4l2_ioctl_streamon(fd: BorrowedFd<'_>, buf_kind: v4l2_buf_type) -> io::Result<()> {
    raw::v4l2_ioctl_streamon(fd, buf_kind.into())
}

/// Stops Streaming I/O
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
#[instrument(level = "trace")]
pub fn v4l2_ioctl_streamoff(fd: BorrowedFd<'_>, buf_kind: v4l2_buf_type) -> io::Result<()> {
    raw::v4l2_ioctl_streamoff(fd, buf_kind.into())
}

/// Sets the EDID of a v4l2 device
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor, or if the EDIDs length isn't
/// aligned to a block size (128 bytes)
#[instrument(level = "trace")]
pub fn v4l2_ioctl_s_edid(fd: BorrowedFd<'_>, edid: &mut [u8]) -> io::Result<()> {
    if (edid.len() % 128) != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "EDIDs size must be aligned to 128 bytes",
        ));
    }

    let num_blocks = edid.len() / 128;
    if num_blocks > 255 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "EDIDs must not have more than 255 blocks",
        ));
    }

    let num_blocks =
        u32::try_from(num_blocks).expect("We just checked that num_blocks fit into a u32");

    let arg = raw::v4l2_edid {
        blocks: num_blocks,
        edid: edid.as_mut_ptr(),
        ..Default::default()
    };

    raw::v4l2_ioctl_s_edid(fd, arg).map(|_| ())
}

/// Sets the EDID of a v4l2 sub-device
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
#[instrument(level = "trace")]
pub fn v4l2_ioctl_subdev_s_edid(fd: BorrowedFd<'_>, edid: &mut [u8]) -> io::Result<()> {
    if (edid.len() % 128) != 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "EDIDs size must be aligned to 128 bytes",
        ));
    }

    let num_blocks = edid.len() / 128;
    if num_blocks > 255 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "EDIDs must not have more than 255 blocks",
        ));
    }

    let num_blocks =
        u32::try_from(num_blocks).expect("We just checked that num_blocks fit into a u32");

    let arg = raw::v4l2_edid {
        blocks: num_blocks,
        edid: edid.as_mut_ptr(),
        ..Default::default()
    };

    raw::v4l2_ioctl_subdev_s_edid(fd, arg).map(|_| ())
}

/// Digital Video Timings
#[repr(C, u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum v4l2_dv_timings {
    /// BT656 / BT1120 Timings Type
    Bt_656_1120(raw::v4l2_bt_timings) = raw::V4L2_DV_BT_656_1120,

    #[doc(hidden)]
    _Reserved([u32; 32]),
}

#[cfg(test)]
mod tests_v4l2_dv_timings {
    use crate::{raw, wrapper};

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_dv_timings>(),
            size_of::<raw::v4l2_dv_timings>(),
            concat!("Size of: ", stringify!(v4l2_dv_timings))
        );

        // We can't check for alignment equality because Rust doesn't allow for packed attributes to
        // be set on union. Since we convert from the lowest alignment one to the highest alignment
        // one, but transmute from the highest to lowest, it should be just fine.
        assert!(
            align_of::<wrapper::v4l2_dv_timings>() >= align_of::<raw::v4l2_dv_timings>(),
            concat!("Alignment of ", stringify!(v4l2_dv_timings))
        );
    }
}

impl TryFrom<raw::v4l2_dv_timings> for v4l2_dv_timings {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_dv_timings) -> Result<Self, Self::Error> {
        let kind = value.type_;
        match kind {
            raw::V4L2_DV_BT_656_1120 => Ok(Self::Bt_656_1120(
                // SAFETY: We just checked the union tag, we know we access the right part of it.
                unsafe { value.__bindgen_anon_1.bt },
            )),
            _ => Err(Self::Error::InvalidStructField {
                name: String::from("type"),
                value: format!("{kind}"),
            }),
        }
    }
}

impl From<v4l2_dv_timings> for raw::v4l2_dv_timings {
    fn from(value: v4l2_dv_timings) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_dv_timings, Self>(value) }
    }
}

/// Sets the DV Timings on a v4l2 device.
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
#[instrument(level = "trace")]
pub fn v4l2_ioctl_s_dv_timings(fd: BorrowedFd<'_>, timings: v4l2_dv_timings) -> io::Result<()> {
    raw::v4l2_ioctl_s_dv_timings(fd, timings.into()).map(|_| ())
}

/// Sets the DV Timings on a v4l2 sub-device.
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
#[instrument(level = "trace")]
pub fn v4l2_ioctl_subdev_s_dv_timings(
    fd: BorrowedFd<'_>,
    timings: v4l2_dv_timings,
) -> io::Result<()> {
    raw::v4l2_ioctl_s_dv_timings(fd, timings.into()).map(|_| ())
}

/// Senses the DV Timings on a v4l2 device
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
#[instrument(level = "trace")]
pub fn v4l2_ioctl_query_dv_timings(fd: BorrowedFd<'_>) -> io::Result<v4l2_dv_timings> {
    let arg = raw::v4l2_dv_timings::default();

    raw::v4l2_ioctl_query_dv_timings(fd, arg).map(|f| {
        f.try_into()
            .expect("The kernel returned an unexpected type")
    })
}

/// Senses the DV Timings on a v4l2 sub-device
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
#[instrument(level = "trace")]
pub fn v4l2_ioctl_subdev_query_dv_timings(fd: BorrowedFd<'_>) -> io::Result<v4l2_dv_timings> {
    let arg = raw::v4l2_dv_timings::default();

    raw::v4l2_ioctl_subdev_query_dv_timings(fd, arg).map(|f| {
        f.try_into()
            .expect("The kernel returned an unexpected type")
    })
}

/// Vertical Sync Event Data
#[repr(C, packed)]
#[derive(Clone, Copy, PartialEq)]
pub struct v4l2_event_vsync {
    field: u8,
}

impl TryFrom<raw::v4l2_event_vsync> for v4l2_event_vsync {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_event_vsync) -> Result<Self, Self::Error> {
        let field: v4l2_field = <u8 as Into<u32>>::into(value.field)
            .try_into()
            .map_err(|_field| Self::Error::InvalidValue(format!("{}", value.field)))?;

        if field > v4l2_field::V4L2_FIELD_BOTTOM {
            return Err(Self::Error::InvalidValue(format!("{}", value.field)));
        }

        Ok(Self {
            field: field.into(),
        })
    }
}

impl From<v4l2_event_vsync> for raw::v4l2_event_vsync {
    fn from(value: v4l2_event_vsync) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_event_vsync, Self>(value) }
    }
}

impl fmt::Debug for v4l2_event_vsync {
    #[expect(
        clippy::unwrap_in_result,
        reason = "We know we will never hit that condition"
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let field: v4l2_field = <u8 as Into<u32>>::into(self.field)
            .try_into()
            .expect("We already checked it was a valid value in TryFrom.");

        f.debug_struct("v4l2_event_vsync")
            .field("field", &field)
            .finish()
    }
}

#[cfg(test)]
mod tests_v4l2_event_vsync {
    use crate::{raw, wrapper};

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_event_vsync>(),
            size_of::<raw::v4l2_event_vsync>()
        );

        assert_eq!(
            align_of::<wrapper::v4l2_event_vsync>(),
            align_of::<raw::v4l2_event_vsync>()
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event_vsync, field),
            std::mem::offset_of!(raw::v4l2_event_vsync, field)
        );
    }
}

/// Control Event Data
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct v4l2_event_ctrl {
    changes: u32,
    kind: u32,
    value: i64,
    flags: u32,
    mininum: i32,
    maximum: i32,
    step: i32,
    default_value: i32,
}

impl TryFrom<raw::v4l2_event_ctrl> for v4l2_event_ctrl {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_event_ctrl) -> Result<Self, Self::Error> {
        if (value.changes
            & !(V4L2_EVENT_CTRL_CH_VALUE
                | V4L2_EVENT_CTRL_CH_FLAGS
                | V4L2_EVENT_CTRL_CH_RANGE
                | V4L2_EVENT_CTRL_CH_DIMENSIONS))
            != 0
        {
            return Err(Self::Error::InvalidValue(format!("{}", value.changes)));
        }

        unimplemented!();
    }
}

impl From<v4l2_event_ctrl> for raw::v4l2_event_ctrl {
    fn from(value: v4l2_event_ctrl) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_event_ctrl, Self>(value) }
    }
}

#[cfg(test)]
mod tests_v4l2_event_ctrl {
    use crate::{raw, wrapper};

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_event_ctrl>(),
            size_of::<raw::v4l2_event_ctrl>()
        );

        assert_eq!(
            align_of::<wrapper::v4l2_event_ctrl>(),
            align_of::<raw::v4l2_event_ctrl>()
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event_ctrl, changes),
            std::mem::offset_of!(raw::v4l2_event_ctrl, changes)
        );
    }
}

/// Frame Reception Event Data
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct v4l2_event_frame_sync {
    frame_sequence: u32,
}

impl TryFrom<raw::v4l2_event_frame_sync> for v4l2_event_frame_sync {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_event_frame_sync) -> Result<Self, Self::Error> {
        Ok(Self {
            frame_sequence: value.frame_sequence,
        })
    }
}

impl From<v4l2_event_frame_sync> for raw::v4l2_event_frame_sync {
    fn from(value: v4l2_event_frame_sync) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_event_frame_sync, Self>(value) }
    }
}

#[cfg(test)]
mod tests_v4l2_event_frame_sync {
    use crate::{raw, wrapper};

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_event_frame_sync>(),
            size_of::<raw::v4l2_event_frame_sync>()
        );

        assert_eq!(
            align_of::<wrapper::v4l2_event_frame_sync>(),
            align_of::<raw::v4l2_event_frame_sync>()
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event_frame_sync, frame_sequence),
            std::mem::offset_of!(raw::v4l2_event_frame_sync, frame_sequence)
        );
    }
}

/// Source Parameters Change Event Data
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct v4l2_event_src_change {
    changes: u32,
}

impl TryFrom<raw::v4l2_event_src_change> for v4l2_event_src_change {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_event_src_change) -> Result<Self, Self::Error> {
        if (value.changes & !V4L2_EVENT_SRC_CH_RESOLUTION) != 0 {
            return Err(Self::Error::InvalidValue(format!("{}", value.changes)));
        }

        Ok(Self {
            changes: value.changes,
        })
    }
}

impl From<v4l2_event_src_change> for raw::v4l2_event_src_change {
    fn from(value: v4l2_event_src_change) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_event_src_change, Self>(value) }
    }
}

#[cfg(test)]
mod tests_v4l2_event_src_change {
    use crate::{raw, wrapper};

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_event_src_change>(),
            size_of::<raw::v4l2_event_src_change>()
        );

        assert_eq!(
            align_of::<wrapper::v4l2_event_src_change>(),
            align_of::<raw::v4l2_event_src_change>()
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event_src_change, changes),
            std::mem::offset_of!(raw::v4l2_event_src_change, changes)
        );
    }
}

/// Motion Detection Event Data
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct v4l2_event_motion_det {
    flags: u32,
    frame_sequence: u32,
    region_mask: u32,
}

impl TryFrom<raw::v4l2_event_motion_det> for v4l2_event_motion_det {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_event_motion_det) -> Result<Self, Self::Error> {
        if (value.flags & !V4L2_EVENT_MD_FL_HAVE_FRAME_SEQ) != 0 {
            return Err(Self::Error::InvalidValue(format!("{:x}", value.flags)));
        }

        Ok(Self {
            flags: value.flags,
            frame_sequence: value.frame_sequence,
            region_mask: value.region_mask,
        })
    }
}

impl From<v4l2_event_motion_det> for raw::v4l2_event_motion_det {
    fn from(value: v4l2_event_motion_det) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_event_motion_det, Self>(value) }
    }
}

#[cfg(test)]
mod tests_v4l2_event_motion_det {
    use crate::{raw, wrapper};

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_event_motion_det>(),
            size_of::<raw::v4l2_event_motion_det>()
        );

        assert_eq!(
            align_of::<wrapper::v4l2_event_motion_det>(),
            align_of::<raw::v4l2_event_motion_det>()
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event_motion_det, flags),
            std::mem::offset_of!(raw::v4l2_event_motion_det, flags)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event_motion_det, frame_sequence),
            std::mem::offset_of!(raw::v4l2_event_motion_det, frame_sequence)
        );
        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event_motion_det, region_mask),
            std::mem::offset_of!(raw::v4l2_event_motion_det, region_mask)
        );
    }
}

/// Kind of Event being reported.
#[repr(C, u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum v4l2_event_type {
    /// Vertical Sync Event
    Vsync(v4l2_event_vsync) = V4L2_EVENT_VSYNC,

    /// End of Stream Event. Typically for MPEG decoders to report the last frame has been decoded.
    EndOfStream = V4L2_EVENT_EOS,

    /// Control Value Event. The associated value is the control ID from which you want to receive
    /// events.
    Control(v4l2_event_ctrl) = V4L2_EVENT_CTRL,

    /// Reception of a Frame has begun
    FrameSync(v4l2_event_frame_sync) = V4L2_EVENT_FRAME_SYNC,

    /// A source parameter has changed.
    SourceChange(v4l2_event_src_change) = V4L2_EVENT_SOURCE_CHANGE,

    /// The motion detection state for one region or more has changed.
    MotionDetection(v4l2_event_motion_det) = V4L2_EVENT_MOTION_DET,

    /// Private Event Data
    Private([u8; 64]),
}

/// An Event
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct v4l2_event {
    kind: v4l2_event_type,
    pending: u32,
    sequence: u32,
    timestamp: Timespec,
    id: u32,
    _reserved: [u32; 8],
}

impl v4l2_event {
    /// Returns the kind of event we represent.
    #[must_use]
    pub fn kind(&self) -> v4l2_event_type {
        self.kind
    }

    /// Returns the number of pending events left to dequeue.
    #[must_use]
    pub fn pending(&self) -> u32 {
        self.pending
    }

    /// Returns the event sequence number.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.sequence
    }
}

impl TryFrom<raw::v4l2_event> for v4l2_event {
    type Error = ConversionError;

    fn try_from(value: raw::v4l2_event) -> Result<Self, Self::Error> {
        Ok(Self {
            kind: match value.type_ {
                V4L2_EVENT_VSYNC => {
                    // SAFETY: We just checked the event type, so we know the union variant to
                    // expect
                    let val = unsafe { value.u.vsync };

                    v4l2_event_type::Vsync(val.try_into()?)
                }
                V4L2_EVENT_EOS => v4l2_event_type::EndOfStream,
                V4L2_EVENT_CTRL => {
                    // SAFETY: We just checked the event type, so we know the union variant to
                    // expect
                    let val = unsafe { value.u.ctrl };

                    v4l2_event_type::Control(val.try_into()?)
                }
                V4L2_EVENT_FRAME_SYNC => {
                    // SAFETY: We just checked the event type, so we know the union variant to
                    // expect
                    let val = unsafe { value.u.frame_sync };

                    v4l2_event_type::FrameSync(val.try_into()?)
                }
                V4L2_EVENT_SOURCE_CHANGE => {
                    // SAFETY: We just checked the event type, so we know the union variant to
                    // expect
                    let val = unsafe { value.u.src_change };

                    v4l2_event_type::SourceChange(val.try_into()?)
                }
                V4L2_EVENT_MOTION_DET => {
                    // SAFETY: We just checked the event type, so we know the union variant to
                    // expect
                    let val = unsafe { value.u.motion_det };

                    v4l2_event_type::MotionDetection(val.try_into()?)
                }
                _ => return Err(Self::Error::InvalidValue(format!("{}", value.type_))),
            },
            pending: value.pending,
            sequence: value.sequence,
            timestamp: Timespec {
                tv_sec: value.timestamp.tv_sec,
                tv_nsec: value.timestamp.tv_nsec,
            },
            id: value.id,
            _reserved: [0; 8],
        })
    }
}

impl From<v4l2_event> for raw::v4l2_event {
    fn from(value: v4l2_event) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_event, Self>(value) }
    }
}

#[cfg(test)]
mod tests_v4l2_event {
    use crate::{raw, wrapper};

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_event>(),
            size_of::<raw::v4l2_event>()
        );

        assert_eq!(
            align_of::<wrapper::v4l2_event>(),
            align_of::<raw::v4l2_event>()
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event, kind),
            std::mem::offset_of!(raw::v4l2_event, type_)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event, pending),
            std::mem::offset_of!(raw::v4l2_event, pending)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event, sequence),
            std::mem::offset_of!(raw::v4l2_event, sequence)
        );
        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event, timestamp),
            std::mem::offset_of!(raw::v4l2_event, timestamp)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event, id),
            std::mem::offset_of!(raw::v4l2_event, id)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event, _reserved),
            std::mem::offset_of!(raw::v4l2_event, reserved)
        );
    }
}

/// Dequeue event
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
#[instrument(level = "trace")]
pub fn v4l2_ioctl_dqevent(fd: BorrowedFd<'_>) -> io::Result<v4l2_event> {
    raw::v4l2_ioctl_dqevent(fd).map(|f| {
        f.try_into()
            .expect("The kernel returned an unexpected type")
    })
}

/// Kind of Event Subscription
#[repr(C, u32)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum v4l2_event_subscription_type {
    /// All Events. Only valid for unsubscription.
    All = V4L2_EVENT_ALL,

    /// Vertical Sync Event
    Vsync = V4L2_EVENT_VSYNC,

    /// End of Stream Event. Typically for MPEG decoders to report the last frame has been decoded.
    EndOfStream = V4L2_EVENT_EOS,

    /// Control Value Event. The associated value is the control ID from which you want to receive
    /// events.
    Control(u32) = V4L2_EVENT_CTRL,

    /// Reception of a Frame has begun.
    FrameSync = V4L2_EVENT_FRAME_SYNC,

    /// A source parameter has changed.
    SourceChange = V4L2_EVENT_SOURCE_CHANGE,

    /// The motion detection state for one region or more has changed.
    MotionDetection = V4L2_EVENT_MOTION_DET,
}

/// Event Subscription
#[repr(C)]
#[derive(Debug)]
pub struct v4l2_event_subscription {
    kind: v4l2_event_subscription_type,
    flags: u32,
    _reserved: [u32; 5],
}

impl v4l2_event_subscription {
    /// Creates a new event subscription structure
    #[must_use]
    pub fn new(kind: v4l2_event_subscription_type) -> Self {
        Self {
            kind,
            flags: 0,
            _reserved: [0; 5],
        }
    }
}

impl From<v4l2_event_subscription> for raw::v4l2_event_subscription {
    fn from(value: v4l2_event_subscription) -> Self {
        // SAFETY: We know from Rust layout rules and our tests that the layouts between the two
        // structures are identical. We also know that all the fields in the Rust union are in a
        // valid state. We can safely transmute.
        unsafe { core::mem::transmute::<v4l2_event_subscription, Self>(value) }
    }
}

#[cfg(test)]
mod tests_v4l2_event_subscription {
    use crate::{raw, wrapper};

    #[test]
    fn layout() {
        assert_eq!(
            size_of::<wrapper::v4l2_event_subscription>(),
            size_of::<raw::v4l2_event_subscription>()
        );

        assert_eq!(
            align_of::<wrapper::v4l2_event_subscription>(),
            align_of::<raw::v4l2_event_subscription>()
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event_subscription, kind),
            std::mem::offset_of!(raw::v4l2_event_subscription, type_)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event_subscription, flags),
            std::mem::offset_of!(raw::v4l2_event_subscription, flags)
        );

        assert_eq!(
            std::mem::offset_of!(wrapper::v4l2_event_subscription, _reserved),
            std::mem::offset_of!(raw::v4l2_event_subscription, reserved)
        );
    }
}

/// Subscribes to events
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
#[instrument(level = "trace")]
pub fn v4l2_ioctl_subscribe_event(
    fd: BorrowedFd<'_>,
    sub: v4l2_event_subscription,
) -> io::Result<()> {
    if let v4l2_event_subscription_type::All = sub.kind {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "One does not simply subscribe to all events.",
        ));
    }

    raw::v4l2_ioctl_subscribe_event(fd, sub.into())
}

/// Unsubscribes from events
///
/// # Errors
///
/// If there's an I/O Error while accessing the file descriptor.
#[instrument(level = "trace")]
pub fn v4l2_ioctl_unsubscribe_event(
    fd: BorrowedFd<'_>,
    sub: v4l2_event_subscription,
) -> io::Result<()> {
    raw::v4l2_ioctl_unsubscribe_event(fd, sub.into())
}
