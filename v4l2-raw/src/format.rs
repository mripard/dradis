#![allow(deprecated)]

use core::fmt;

use facet::Facet;
use facet_enum_repr::FacetEnumRepr;
use facet_reflect::Peek;

use crate::ConversionError;

/// Macro to create a fourcc u32 representation
#[macro_export]
macro_rules! v4l2_fourcc {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        (($a as u32) | (($b as u32) << 8) | (($c as u32) << 16) | (($d as u32) << 24))
    };
}

/// Macro to create a big-endian fourcc u32 representation
#[macro_export]
macro_rules! v4l2_fourcc_be {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        v4l2_fourcc!($a, $b, $c, $d) | (1 << 31)
    };
}

/// V4L2 Pixel Format Representation
///
/// See the kernel [documentation] for more details.
///
/// [documentation]: https://www.kernel.org/doc/html/latest/userspace-api/media/v4l/pixfmt.html#image-formats
#[repr(u32)]
#[derive(Clone, Copy, Debug, Facet, FacetEnumRepr, PartialEq)]
pub enum v4l2_pix_fmt {
    //  RGB formats (1 or 2 bytes per pixel)
    /// 8-bit RGB 3-3-2
    V4L2_PIX_FMT_RGB332 = v4l2_fourcc!('R', 'G', 'B', '1'),
    /// 16-bit A/XRGB 4-4-4-4
    #[deprecated]
    V4L2_PIX_FMT_RGB444 = v4l2_fourcc!('R', '4', '4', '4'),
    /// 16-bit ARGB 4-4-4-4
    V4L2_PIX_FMT_ARGB444 = v4l2_fourcc!('A', 'R', '1', '2'),
    /// 16-bit XRGB 4-4-4-4
    V4L2_PIX_FMT_XRGB444 = v4l2_fourcc!('X', 'R', '1', '2'),
    /// 16-bit RGBA 4-4-4-4
    V4L2_PIX_FMT_RGBA444 = v4l2_fourcc!('R', 'A', '1', '2'),
    /// 16-bit RGBX 4-4-4-4
    V4L2_PIX_FMT_RGBX444 = v4l2_fourcc!('R', 'X', '1', '2'),
    /// 16-bit ABGR 4-4-4-4
    V4L2_PIX_FMT_ABGR444 = v4l2_fourcc!('A', 'B', '1', '2'),
    /// 16-bit XBGR 4-4-4-4
    V4L2_PIX_FMT_XBGR444 = v4l2_fourcc!('X', 'B', '1', '2'),
    /// 16-bit BGRA 4-4-4-4
    V4L2_PIX_FMT_BGRA444 = v4l2_fourcc!('G', 'A', '1', '2'),
    /// 16-bit BGRX 4-4-4-4
    V4L2_PIX_FMT_BGRX444 = v4l2_fourcc!('B', 'X', '1', '2'),
    /// 16-bit A/XRGB 1-5-5-5
    #[deprecated]
    V4L2_PIX_FMT_RGB555 = v4l2_fourcc!('R', 'G', 'B', 'O'),
    /// 16-bit ARGB 1-5-5-5
    V4L2_PIX_FMT_ARGB555 = v4l2_fourcc!('A', 'R', '1', '5'),
    /// 16-bit XRGB 1-5-5-5
    V4L2_PIX_FMT_XRGB555 = v4l2_fourcc!('X', 'R', '1', '5'),
    /// 16-bit RGBA 5-5-5-1
    V4L2_PIX_FMT_RGBA555 = v4l2_fourcc!('R', 'A', '1', '5'),
    /// 16-bit RGBX 5-5-5-1
    V4L2_PIX_FMT_RGBX555 = v4l2_fourcc!('R', 'X', '1', '5'),
    /// 16-bit ABGR 1-5-5-5
    V4L2_PIX_FMT_ABGR555 = v4l2_fourcc!('A', 'B', '1', '5'),
    /// 16-bit XBGR 1-5-5-5
    V4L2_PIX_FMT_XBGR555 = v4l2_fourcc!('X', 'B', '1', '5'),
    /// 16-bit BGRA 5-5-5-1
    V4L2_PIX_FMT_BGRA555 = v4l2_fourcc!('B', 'A', '1', '5'),
    /// 16-bit BGRX 5-5-5-1
    V4L2_PIX_FMT_BGRX555 = v4l2_fourcc!('B', 'X', '1', '5'),
    /// 16-bit RGB 5-6-5
    V4L2_PIX_FMT_RGB565 = v4l2_fourcc!('R', 'G', 'B', 'P'),
    /// 16-bit A/XRGB 1-5-5-5 BE
    #[deprecated]
    V4L2_PIX_FMT_RGB555X = v4l2_fourcc!('R', 'G', 'B', 'Q'),
    /// 16-bit ARGB 1-5-5-5 BE
    V4L2_PIX_FMT_ARGB555X = v4l2_fourcc_be!('A', 'R', '1', '5'),
    /// 16-bit XRGB 1-5-5-5 BE
    V4L2_PIX_FMT_XRGB555X = v4l2_fourcc_be!('X', 'R', '1', '5'),
    /// 16-bit RGB 5-6-5 BE
    V4L2_PIX_FMT_RGB565X = v4l2_fourcc!('R', 'G', 'B', 'R'),

    //  RGB formats (3 or 4 bytes per pixel)
    /// 18-bit BGRX 6-6-6-14
    V4L2_PIX_FMT_BGR666 = v4l2_fourcc!('B', 'G', 'R', 'H'),
    /// 24-bit BGR 8-8-8
    V4L2_PIX_FMT_BGR24 = v4l2_fourcc!('B', 'G', 'R', '3'),

    /// RGB Format, in that order from left to right, with 8 bits per component
    /// 24-bit RGB 8-8-8
    V4L2_PIX_FMT_RGB24 = v4l2_fourcc!('R', 'G', 'B', '3'),
    /// 32-bit BGRA/X 8-8-8-8
    #[deprecated]
    V4L2_PIX_FMT_BGR32 = v4l2_fourcc!('B', 'G', 'R', '4'),
    /// 32-bit BGRA 8-8-8-8
    V4L2_PIX_FMT_ABGR32 = v4l2_fourcc!('A', 'R', '2', '4'),
    /// 32-bit BGRX 8-8-8-8
    V4L2_PIX_FMT_XBGR32 = v4l2_fourcc!('X', 'R', '2', '4'),
    /// 32-bit ABGR 8-8-8-8
    V4L2_PIX_FMT_BGRA32 = v4l2_fourcc!('R', 'A', '2', '4'),
    /// 32-bit XBGR 8-8-8-8
    V4L2_PIX_FMT_BGRX32 = v4l2_fourcc!('R', 'X', '2', '4'),
    /// 32-bit A/XRGB 8-8-8-8
    #[deprecated]
    V4L2_PIX_FMT_RGB32 = v4l2_fourcc!('R', 'G', 'B', '4'),
    /// 32-bit RGBA 8-8-8-8
    V4L2_PIX_FMT_RGBA32 = v4l2_fourcc!('A', 'B', '2', '4'),
    /// 32-bit RGBX 8-8-8-8
    V4L2_PIX_FMT_RGBX32 = v4l2_fourcc!('X', 'B', '2', '4'),
    /// 32-bit ARGB 8-8-8-8
    V4L2_PIX_FMT_ARGB32 = v4l2_fourcc!('B', 'A', '2', '4'),
    /// 32-bit XRGB 8-8-8-8
    V4L2_PIX_FMT_XRGB32 = v4l2_fourcc!('B', 'X', '2', '4'),
    /// 32-bit RGBX 10-10-10-2
    V4L2_PIX_FMT_RGBX1010102 = v4l2_fourcc!('R', 'X', '3', '0'),
    /// 32-bit RGBA 10-10-10-2
    V4L2_PIX_FMT_RGBA1010102 = v4l2_fourcc!('R', 'A', '3', '0'),
    /// 32-bit ARGB 2-10-10-10
    V4L2_PIX_FMT_ARGB2101010 = v4l2_fourcc!('A', 'R', '3', '0'),

    //  RGB formats (6 or 8 bytes per pixel)
    /// 12-bit Depth BGR
    V4L2_PIX_FMT_BGR48_12 = v4l2_fourcc!('B', '3', '1', '2'),
    /// 48-bit BGR 16-16-16
    V4L2_PIX_FMT_BGR48 = v4l2_fourcc!('B', 'G', 'R', '6'),
    /// 48-bit RGB 16-16-16
    V4L2_PIX_FMT_RGB48 = v4l2_fourcc!('R', 'G', 'B', '6'),
    /// 12-bit Depth BGRA
    V4L2_PIX_FMT_ABGR64_12 = v4l2_fourcc!('B', '4', '1', '2'),

    //  Grey formats
    /// 8-bit Greyscale
    V4L2_PIX_FMT_GREY = v4l2_fourcc!('G', 'R', 'E', 'Y'),
    /// 4-bit Greyscale
    V4L2_PIX_FMT_Y4 = v4l2_fourcc!('Y', '0', '4', ' '),
    /// 6-bit Greyscale
    V4L2_PIX_FMT_Y6 = v4l2_fourcc!('Y', '0', '6', ' '),
    /// 10-bit Greyscale
    V4L2_PIX_FMT_Y10 = v4l2_fourcc!('Y', '1', '0', ' '),
    /// 12-bit Greyscale
    V4L2_PIX_FMT_Y12 = v4l2_fourcc!('Y', '1', '2', ' '),
    /// 12-bit Greyscale (bits 15-4)
    V4L2_PIX_FMT_Y012 = v4l2_fourcc!('Y', '0', '1', '2'),
    /// 14-bit Greyscale
    V4L2_PIX_FMT_Y14 = v4l2_fourcc!('Y', '1', '4', ' '),
    /// 16-bit Greyscale
    V4L2_PIX_FMT_Y16 = v4l2_fourcc!('Y', '1', '6', ' '),
    /// 16-bit Greyscale BE
    V4L2_PIX_FMT_Y16_BE = v4l2_fourcc_be!('Y', '1', '6', ' '),

    //  Grey bit-packed formats
    /// 10-bit Greyscale (Packed)
    V4L2_PIX_FMT_Y10BPACK = v4l2_fourcc!('Y', '1', '0', 'B'),
    /// 10-bit Greyscale (MIPI Packed)
    V4L2_PIX_FMT_Y10P = v4l2_fourcc!('Y', '1', '0', 'P'),
    /// 10-bit greyscale (IPU3 Packed)
    V4L2_PIX_FMT_IPU3_Y10 = v4l2_fourcc!('i', 'p', '3', 'y'),
    /// 12-bit Greyscale (MIPI Packed)
    V4L2_PIX_FMT_Y12P = v4l2_fourcc!('Y', '1', '2', 'P'),
    /// 14-bit Greyscale (MIPI Packed)
    V4L2_PIX_FMT_Y14P = v4l2_fourcc!('Y', '1', '4', 'P'),

    //  Palette formats
    /// 8-bit Palette
    V4L2_PIX_FMT_PAL8 = v4l2_fourcc!('P', 'A', 'L', '8'),

    //  Chrominance formats
    /// 8-bit Chrominance UV 4-4
    V4L2_PIX_FMT_UV8 = v4l2_fourcc!('U', 'V', '8', ' '),

    //  Luminance+Chrominance formats
    /// YUYV 4:2:2
    V4L2_PIX_FMT_YUYV = v4l2_fourcc!('Y', 'U', 'Y', 'V'),
    /// YYUV 4:2:2
    V4L2_PIX_FMT_YYUV = v4l2_fourcc!('Y', 'Y', 'U', 'V'),
    /// YVYU 4:2:2
    V4L2_PIX_FMT_YVYU = v4l2_fourcc!('Y', 'V', 'Y', 'U'),
    /// UYVY 4:2:2
    V4L2_PIX_FMT_UYVY = v4l2_fourcc!('U', 'Y', 'V', 'Y'),
    /// VYUY 4:2:2
    V4L2_PIX_FMT_VYUY = v4l2_fourcc!('V', 'Y', 'U', 'Y'),
    /// YUV 4:1:1 (Packed)
    V4L2_PIX_FMT_Y41P = v4l2_fourcc!('Y', '4', '1', 'P'),
    /// 16-bit A/XYUV 4-4-4-4
    V4L2_PIX_FMT_YUV444 = v4l2_fourcc!('Y', '4', '4', '4'),
    /// 16-bit A/XYUV 1-5-5-5
    V4L2_PIX_FMT_YUV555 = v4l2_fourcc!('Y', 'U', 'V', 'O'),
    /// 16-bit YUV 5-6-5
    V4L2_PIX_FMT_YUV565 = v4l2_fourcc!('Y', 'U', 'V', 'P'),
    /// 24-bit YUV 4:4:4 8-8-8
    V4L2_PIX_FMT_YUV24 = v4l2_fourcc!('Y', 'U', 'V', '3'),
    /// 32-bit A/XYUV 8-8-8-8
    V4L2_PIX_FMT_YUV32 = v4l2_fourcc!('Y', 'U', 'V', '4'),
    /// 32-bit AYUV 8-8-8-8
    V4L2_PIX_FMT_AYUV32 = v4l2_fourcc!('A', 'Y', 'U', 'V'),
    /// 32-bit XYUV 8-8-8-8
    V4L2_PIX_FMT_XYUV32 = v4l2_fourcc!('X', 'Y', 'U', 'V'),
    /// 32-bit VUYA 8-8-8-8
    V4L2_PIX_FMT_VUYA32 = v4l2_fourcc!('V', 'U', 'Y', 'A'),
    /// 32-bit VUYX 8-8-8-8
    V4L2_PIX_FMT_VUYX32 = v4l2_fourcc!('V', 'U', 'Y', 'X'),
    /// 32-bit YUVA 8-8-8-8
    V4L2_PIX_FMT_YUVA32 = v4l2_fourcc!('Y', 'U', 'V', 'A'),
    /// 32-bit YUVX 8-8-8-8
    V4L2_PIX_FMT_YUVX32 = v4l2_fourcc!('Y', 'U', 'V', 'X'),
    /// YUV 4:2:0 (M420)
    V4L2_PIX_FMT_M420 = v4l2_fourcc!('M', '4', '2', '0'),
    /// 12-bit YUV 4:4:4 Packed
    V4L2_PIX_FMT_YUV48_12 = v4l2_fourcc!('Y', '3', '1', '2'),

    //
    // YCbCr packed format. For each Y2xx format, xx bits of valid data occupy the MSBs
    // of the 16 bit components, and 16-xx bits of zero padding occupy the LSBs.
    /// 10-bit YUYV Packed
    V4L2_PIX_FMT_Y210 = v4l2_fourcc!('Y', '2', '1', '0'),
    /// 12-bit YUYV Packed
    V4L2_PIX_FMT_Y212 = v4l2_fourcc!('Y', '2', '1', '2'),
    /// 16-bit YUYV Packed
    V4L2_PIX_FMT_Y216 = v4l2_fourcc!('Y', '2', '1', '6'),

    //  two planes -- one Y, one Cr + Cb interleaved
    /// Y/UV 4:2:0
    V4L2_PIX_FMT_NV12 = v4l2_fourcc!('N', 'V', '1', '2'),
    /// Y/VU 4:2:0
    V4L2_PIX_FMT_NV21 = v4l2_fourcc!('N', 'V', '2', '1'),
    /// Y/UV 4:2:2
    V4L2_PIX_FMT_NV16 = v4l2_fourcc!('N', 'V', '1', '6'),
    /// Y/VU 4:2:2
    V4L2_PIX_FMT_NV61 = v4l2_fourcc!('N', 'V', '6', '1'),
    /// Y/UV 4:4:4
    V4L2_PIX_FMT_NV24 = v4l2_fourcc!('N', 'V', '2', '4'),
    /// Y/VU 4:4:4
    V4L2_PIX_FMT_NV42 = v4l2_fourcc!('N', 'V', '4', '2'),
    /// 10-bit Y/UV 4:2:0
    V4L2_PIX_FMT_P010 = v4l2_fourcc!('P', '0', '1', '0'),
    /// 12-bit Y/UV 4:2:0
    V4L2_PIX_FMT_P012 = v4l2_fourcc!('P', '0', '1', '2'),

    //  two non contiguous planes - one Y, one Cr + Cb interleaved
    /// Y/UV 4:2:0 (N-C)
    V4L2_PIX_FMT_NV12M = v4l2_fourcc!('N', 'M', '1', '2'),
    /// Y/VU 4:2:0 (N-C)
    V4L2_PIX_FMT_NV21M = v4l2_fourcc!('N', 'M', '2', '1'),
    /// Y/UV 4:2:2 (N-C)
    V4L2_PIX_FMT_NV16M = v4l2_fourcc!('N', 'M', '1', '6'),
    /// Y/VU 4:2:2 (N-C)
    V4L2_PIX_FMT_NV61M = v4l2_fourcc!('N', 'M', '6', '1'),
    /// 12-bit Y/UV 4:2:0 (N-C)
    V4L2_PIX_FMT_P012M = v4l2_fourcc!('P', 'M', '1', '2'),

    //  three planes - Y Cb, Cr
    /// Planar YUV 4:1:0
    V4L2_PIX_FMT_YUV410 = v4l2_fourcc!('Y', 'U', 'V', '9'),
    /// Planar YVU 4:1:0
    V4L2_PIX_FMT_YVU410 = v4l2_fourcc!('Y', 'V', 'U', '9'),
    /// Planar YVU 4:1:1
    V4L2_PIX_FMT_YUV411P = v4l2_fourcc!('4', '1', '1', 'P'),
    /// Planar YUV 4:2:0
    V4L2_PIX_FMT_YUV420 = v4l2_fourcc!('Y', 'U', '1', '2'),
    /// Planar YVU 4:2:0
    V4L2_PIX_FMT_YVU420 = v4l2_fourcc!('Y', 'V', '1', '2'),
    /// Planar YVU 4:2:2
    V4L2_PIX_FMT_YUV422P = v4l2_fourcc!('4', '2', '2', 'P'),

    //  three non contiguous planes - Y, Cb, Cr
    /// Planar YUV 4:2:0 (N-C)
    V4L2_PIX_FMT_YUV420M = v4l2_fourcc!('Y', 'M', '1', '2'),
    /// Planar YVU 4:2:0 (N-C)
    V4L2_PIX_FMT_YVU420M = v4l2_fourcc!('Y', 'M', '2', '1'),
    /// Planar YUV 4:2:2 (N-C)
    V4L2_PIX_FMT_YUV422M = v4l2_fourcc!('Y', 'M', '1', '6'),
    /// Planar YVU 4:2:2 (N-C)
    V4L2_PIX_FMT_YVU422M = v4l2_fourcc!('Y', 'M', '6', '1'),
    /// Planar YUV 4:4:4 (N-C)
    V4L2_PIX_FMT_YUV444M = v4l2_fourcc!('Y', 'M', '2', '4'),
    /// Planar YVU 4:4:4 (N-C)
    V4L2_PIX_FMT_YVU444M = v4l2_fourcc!('Y', 'M', '4', '2'),

    //  Tiled YUV formats
    /// Y/UV 4:2:0 (4x4 Linear)
    V4L2_PIX_FMT_NV12_4L4 = v4l2_fourcc!('V', 'T', '1', '2'),
    /// Y/UV 4:2:0 (16x16 Linear)
    V4L2_PIX_FMT_NV12_16L16 = v4l2_fourcc!('H', 'M', '1', '2'),
    /// Y/UV 4:2:0 (32x32 Linear)
    V4L2_PIX_FMT_NV12_32L32 = v4l2_fourcc!('S', 'T', '1', '2'),
    /// 10-bit Y/UV 4:2:0 (4x4 Linear)
    V4L2_PIX_FMT_NV15_4L4 = v4l2_fourcc!('V', 'T', '1', '5'),
    /// 10-bit Y/UV 4:2:0 (4x4 Linear)
    V4L2_PIX_FMT_P010_4L4 = v4l2_fourcc!('T', '0', '1', '0'),
    /// NV12 (8x128 Linear)
    V4L2_PIX_FMT_NV12_8L128 = v4l2_fourcc!('A', 'T', '1', '2'),
    /// 10-bit NV12 (8x128 Linear, BE)
    V4L2_PIX_FMT_NV12_10BE_8L128 = v4l2_fourcc_be!('A', 'X', '1', '2'),

    //  Tiled YUV formats, non contiguous planes
    /// Y/UV 4:2:0 (64x32 MB, N-C)
    V4L2_PIX_FMT_NV12MT = v4l2_fourcc!('T', 'M', '1', '2'),
    /// Y/UV 4:2:0 (16x16 MB, N-C)
    V4L2_PIX_FMT_NV12MT_16X16 = v4l2_fourcc!('V', 'M', '1', '2'),
    /// NV12M (8x128 Linear)
    V4L2_PIX_FMT_NV12M_8L128 = v4l2_fourcc!('N', 'A', '1', '2'),
    /// 10-bit NV12M (8x128 Linear, BE)
    V4L2_PIX_FMT_NV12M_10BE_8L128 = v4l2_fourcc_be!('N', 'T', '1', '2'),

    //  Bayer formats - see http://www.siliconimaging.com/RGB%20Bayer.htm
    /// 8-bit Bayer BGBG/GRGR
    V4L2_PIX_FMT_SBGGR8 = v4l2_fourcc!('B', 'A', '8', '1'),
    /// 8-bit Bayer GBGB/RGRG
    V4L2_PIX_FMT_SGBRG8 = v4l2_fourcc!('G', 'B', 'R', 'G'),
    /// 8-bit Bayer GRGR/BGBG
    V4L2_PIX_FMT_SGRBG8 = v4l2_fourcc!('G', 'R', 'B', 'G'),
    /// 8-bit Bayer RGRG/GBGB
    V4L2_PIX_FMT_SRGGB8 = v4l2_fourcc!('R', 'G', 'G', 'B'),
    /// 10-bit Bayer BGBG/GRGR
    V4L2_PIX_FMT_SBGGR10 = v4l2_fourcc!('B', 'G', '1', '0'),
    /// 10-bit Bayer GBGB/RGRG
    V4L2_PIX_FMT_SGBRG10 = v4l2_fourcc!('G', 'B', '1', '0'),
    /// 10-bit Bayer GRGR/BGBG
    V4L2_PIX_FMT_SGRBG10 = v4l2_fourcc!('B', 'A', '1', '0'),

    // 10bit raw bayer packed, 5 bytes for every 4 pixels
    /// 10-bit Bayer Packed
    V4L2_PIX_FMT_SBGGR10P = v4l2_fourcc!('p', 'B', 'A', 'A'),
    /// 10-bit Bayer Packed
    V4L2_PIX_FMT_SGBRG10P = v4l2_fourcc!('p', 'G', 'A', 'A'),
    /// 10-bit Bayer Packed
    V4L2_PIX_FMT_SGRBG10P = v4l2_fourcc!('p', 'g', 'A', 'A'),
    /// 10-bit Bayer Packed
    V4L2_PIX_FMT_SRGGB10P = v4l2_fourcc!('p', 'R', 'A', 'A'),

    // 10bit raw bayer a-law compressed to 8 bits
    /// 8-bit Bayer (A-law from 10-bit)
    V4L2_PIX_FMT_SBGGR10ALAW8 = v4l2_fourcc!('a', 'B', 'A', '8'),
    /// 8-bit Bayer (A-law from 10-bit)
    V4L2_PIX_FMT_SGBRG10ALAW8 = v4l2_fourcc!('a', 'G', 'A', '8'),
    /// 8-bit Bayer (A-law from 10-bit)
    V4L2_PIX_FMT_SGRBG10ALAW8 = v4l2_fourcc!('a', 'g', 'A', '8'),
    /// 8-bit Bayer (A-law from 10-bit)
    V4L2_PIX_FMT_SRGGB10ALAW8 = v4l2_fourcc!('a', 'R', 'A', '8'),

    // 10bit raw bayer DPCM compressed to 8 bits
    /// 8-bit Bayer (DPCM from 10-bit)
    V4L2_PIX_FMT_SBGGR10DPCM8 = v4l2_fourcc!('b', 'B', 'A', '8'),
    /// 8-bit Bayer (DPCM from 10-bit)
    V4L2_PIX_FMT_SGBRG10DPCM8 = v4l2_fourcc!('b', 'G', 'A', '8'),
    /// 8-bit Bayer (DPCM from 10-bit)
    V4L2_PIX_FMT_SGRBG10DPCM8 = v4l2_fourcc!('B', 'D', '1', '0'),
    /// 8-bit Bayer (DPCM from 10-bit)
    V4L2_PIX_FMT_SRGGB10DPCM8 = v4l2_fourcc!('b', 'R', 'A', '8'),
    /// 12-bit Bayer BGBG/GRGR
    V4L2_PIX_FMT_SBGGR12 = v4l2_fourcc!('B', 'G', '1', '2'),
    /// 12-bit Bayer GBGB/RGRG
    V4L2_PIX_FMT_SGBRG12 = v4l2_fourcc!('G', 'B', '1', '2'),
    /// 12-bit Bayer GRGR/BGBG
    V4L2_PIX_FMT_SGRBG12 = v4l2_fourcc!('B', 'A', '1', '2'),
    /// 12-bit Bayer RGRG/GBGB
    V4L2_PIX_FMT_SRGGB12 = v4l2_fourcc!('R', 'G', '1', '2'),

    // 12bit raw bayer packed, 6 bytes for every 4 pixels
    /// 12-bit Bayer BGBG/GRGR Packed
    V4L2_PIX_FMT_SBGGR12P = v4l2_fourcc!('p', 'B', 'C', 'C'),
    /// 12-bit Bayer GBGB/RGRG Packed
    V4L2_PIX_FMT_SGBRG12P = v4l2_fourcc!('p', 'G', 'C', 'C'),
    /// 12-bit Bayer GRGR/BGBG Packed
    V4L2_PIX_FMT_SGRBG12P = v4l2_fourcc!('p', 'g', 'C', 'C'),
    /// 12-bit Bayer RGRG/GBGB Packed
    V4L2_PIX_FMT_SRGGB12P = v4l2_fourcc!('p', 'R', 'C', 'C'),
    /// 14-bit Bayer BGBG/GRGR
    V4L2_PIX_FMT_SBGGR14 = v4l2_fourcc!('B', 'G', '1', '4'),
    /// 14-bit Bayer GBGB/RGRG
    V4L2_PIX_FMT_SGBRG14 = v4l2_fourcc!('G', 'B', '1', '4'),
    /// 14-bit Bayer GRGR/BGBG
    V4L2_PIX_FMT_SGRBG14 = v4l2_fourcc!('G', 'R', '1', '4'),
    /// 14-bit Bayer RGRG/GBGB
    V4L2_PIX_FMT_SRGGB14 = v4l2_fourcc!('R', 'G', '1', '4'),

    // 14bit raw bayer packed, 7 bytes for every 4 pixels
    /// 14-bit Bayer BGBG/GRGR Packed
    V4L2_PIX_FMT_SBGGR14P = v4l2_fourcc!('p', 'B', 'E', 'E'),
    /// 14-bit Bayer GBGB/RGRG Packed
    V4L2_PIX_FMT_SGBRG14P = v4l2_fourcc!('p', 'G', 'E', 'E'),
    /// 14-bit Bayer GRGR/BGBG Packed
    V4L2_PIX_FMT_SGRBG14P = v4l2_fourcc!('p', 'g', 'E', 'E'),
    /// 14-bit Bayer RGRG/GBGB Packed
    V4L2_PIX_FMT_SRGGB14P = v4l2_fourcc!('p', 'R', 'E', 'E'),
    /// 16-bit Bayer BGBG/GRGR
    V4L2_PIX_FMT_SBGGR16 = v4l2_fourcc!('B', 'Y', 'R', '2'),
    /// 16-bit Bayer GBGB/RGRG
    V4L2_PIX_FMT_SGBRG16 = v4l2_fourcc!('G', 'B', '1', '6'),
    /// 16-bit Bayer GRGR/BGBG
    V4L2_PIX_FMT_SGRBG16 = v4l2_fourcc!('G', 'R', '1', '6'),
    /// 16-bit Bayer RGRG/GBGB
    V4L2_PIX_FMT_SRGGB16 = v4l2_fourcc!('R', 'G', '1', '6'),

    //  HSV formats
    /// 24-bit HSV 8-8-8
    V4L2_PIX_FMT_HSV24 = v4l2_fourcc!('H', 'S', 'V', '3'),
    /// 32-bit XHSV 8-8-8-8
    V4L2_PIX_FMT_HSV32 = v4l2_fourcc!('H', 'S', 'V', '4'),

    //  compressed formats
    /// Motion-JPEG
    V4L2_PIX_FMT_MJPEG = v4l2_fourcc!('M', 'J', 'P', 'G'),
    /// JFIF JPEG
    V4L2_PIX_FMT_JPEG = v4l2_fourcc!('J', 'P', 'E', 'G'),
    /// DV over 1394
    V4L2_PIX_FMT_DV = v4l2_fourcc!('d', 'v', 's', 'd'),
    /// MPEG-1/2/4 Multiplexed
    V4L2_PIX_FMT_MPEG = v4l2_fourcc!('M', 'P', 'E', 'G'),
    /// H.264 with start codes
    V4L2_PIX_FMT_H264 = v4l2_fourcc!('H', '2', '6', '4'),
    /// H.264 without start codes (Annex B byte stream)
    V4L2_PIX_FMT_H264_NO_SC = v4l2_fourcc!('A', 'V', 'C', '1'),
    /// H.264 MVC (Multiview Video Coding)
    V4L2_PIX_FMT_H264_MVC = v4l2_fourcc!('M', '2', '6', '4'),
    /// H.263
    V4L2_PIX_FMT_H263 = v4l2_fourcc!('H', '2', '6', '3'),
    /// MPEG-1 Elementary Stream
    V4L2_PIX_FMT_MPEG1 = v4l2_fourcc!('M', 'P', 'G', '1'),
    /// MPEG-2 Elementary Stream
    V4L2_PIX_FMT_MPEG2 = v4l2_fourcc!('M', 'P', 'G', '2'),
    /// MPEG-2 Parsed Slice Data
    V4L2_PIX_FMT_MPEG2_SLICE = v4l2_fourcc!('M', 'G', '2', 'S'),
    /// MPEG-4 Part 2 Elementary Stream
    V4L2_PIX_FMT_MPEG4 = v4l2_fourcc!('M', 'P', 'G', '4'),
    /// Xvid
    V4L2_PIX_FMT_XVID = v4l2_fourcc!('X', 'V', 'I', 'D'),
    /// VC-1 (SMPTE 421M Annex G compliant stream)
    V4L2_PIX_FMT_VC1_ANNEX_G = v4l2_fourcc!('V', 'C', '1', 'G'),
    /// VC-1 (SMPTE 421M Annex L compliant stream)
    V4L2_PIX_FMT_VC1_ANNEX_L = v4l2_fourcc!('V', 'C', '1', 'L'),
    /// VP8
    V4L2_PIX_FMT_VP8 = v4l2_fourcc!('V', 'P', '8', '0'),
    /// VP8 Parsed Frame
    V4L2_PIX_FMT_VP8_FRAME = v4l2_fourcc!('V', 'P', '8', 'F'),
    /// VP9
    V4L2_PIX_FMT_VP9 = v4l2_fourcc!('V', 'P', '9', '0'),
    /// VP9 Parsed Frame
    V4L2_PIX_FMT_VP9_FRAME = v4l2_fourcc!('V', 'P', '9', 'F'),
    /// HEVC (H.265)
    V4L2_PIX_FMT_HEVC = v4l2_fourcc!('H', 'E', 'V', 'C'),
    /// Fast Walsh Hadamard Transform (vicodec)
    V4L2_PIX_FMT_FWHT = v4l2_fourcc!('F', 'W', 'H', 'T'),
    /// Stateless FWHT (vicodec)
    V4L2_PIX_FMT_FWHT_STATELESS = v4l2_fourcc!('S', 'F', 'W', 'H'),
    /// H.264 Parsed Slice Data
    V4L2_PIX_FMT_H264_SLICE = v4l2_fourcc!('S', '2', '6', '4'),
    /// HEVC Parsed Slice Data
    V4L2_PIX_FMT_HEVC_SLICE = v4l2_fourcc!('S', '2', '6', '5'),
    /// AV1 Parsed Frame
    V4L2_PIX_FMT_AV1_FRAME = v4l2_fourcc!('A', 'V', '1', 'F'),
    /// Sorenson Spark
    V4L2_PIX_FMT_SPK = v4l2_fourcc!('S', 'P', 'K', '0'),
    /// `RealVideo` 8
    V4L2_PIX_FMT_RV30 = v4l2_fourcc!('R', 'V', '3', '0'),
    /// `RealVideo` 9 & 10
    V4L2_PIX_FMT_RV40 = v4l2_fourcc!('R', 'V', '4', '0'),

    //   Vendor-specific formats
    /// GSPCA `CPiA` YUV
    V4L2_PIX_FMT_CPIA1 = v4l2_fourcc!('C', 'P', 'I', 'A'),
    /// Winnov hw compress
    V4L2_PIX_FMT_WNVA = v4l2_fourcc!('W', 'N', 'V', 'A'),
    /// GSPCA SN9C10X
    V4L2_PIX_FMT_SN9C10X = v4l2_fourcc!('S', '9', '1', '0'),
    /// GSPCA SN9C20X I420
    V4L2_PIX_FMT_SN9C20X_I420 = v4l2_fourcc!('S', '9', '2', '0'),
    /// Raw Philips Webcam Type (Old)
    V4L2_PIX_FMT_PWC1 = v4l2_fourcc!('P', 'W', 'C', '1'),
    /// Raw Philips Webcam Type (New)
    V4L2_PIX_FMT_PWC2 = v4l2_fourcc!('P', 'W', 'C', '2'),
    /// GSPCA ET61X251
    V4L2_PIX_FMT_ET61X251 = v4l2_fourcc!('E', '6', '2', '5'),
    /// GSPCA SPCA501
    V4L2_PIX_FMT_SPCA501 = v4l2_fourcc!('S', '5', '0', '1'),
    /// GSPCA SPCA505
    V4L2_PIX_FMT_SPCA505 = v4l2_fourcc!('S', '5', '0', '5'),
    /// GSPCA SPCA508
    V4L2_PIX_FMT_SPCA508 = v4l2_fourcc!('S', '5', '0', '8'),
    /// GSPCA SPCA561
    V4L2_PIX_FMT_SPCA561 = v4l2_fourcc!('S', '5', '6', '1'),
    /// GSPCA PAC207
    V4L2_PIX_FMT_PAC207 = v4l2_fourcc!('P', '2', '0', '7'),
    /// GSPCA MR97310A
    V4L2_PIX_FMT_MR97310A = v4l2_fourcc!('M', '3', '1', '0'),
    /// GSPCA JL2005BCD
    V4L2_PIX_FMT_JL2005BCD = v4l2_fourcc!('J', 'L', '2', '0'),
    /// GSPCA SN9C2028
    V4L2_PIX_FMT_SN9C2028 = v4l2_fourcc!('S', 'O', 'N', 'X'),
    /// GSPCA SQ905C
    V4L2_PIX_FMT_SQ905C = v4l2_fourcc!('9', '0', '5', 'C'),
    /// GSPCA PJPG
    V4L2_PIX_FMT_PJPG = v4l2_fourcc!('P', 'J', 'P', 'G'),
    /// GSPCA OV511
    V4L2_PIX_FMT_OV511 = v4l2_fourcc!('O', '5', '1', '1'),
    /// GSPCA OV518
    V4L2_PIX_FMT_OV518 = v4l2_fourcc!('O', '5', '1', '8'),
    /// GSPCA STV0680
    V4L2_PIX_FMT_STV0680 = v4l2_fourcc!('S', '6', '8', '0'),
    /// A/V + VBI Mux Packet
    V4L2_PIX_FMT_TM6000 = v4l2_fourcc!('T', 'M', '6', '0'),
    /// GSPCA CIT YYVYUY
    V4L2_PIX_FMT_CIT_YYVYUY = v4l2_fourcc!('C', 'I', 'T', 'V'),
    /// GSPCA KONICA420
    V4L2_PIX_FMT_KONICA420 = v4l2_fourcc!('K', 'O', 'N', 'I'),
    /// JPEG Lite
    V4L2_PIX_FMT_JPGL = v4l2_fourcc!('J', 'P', 'G', 'L'),
    /// GSPCA SE401
    V4L2_PIX_FMT_SE401 = v4l2_fourcc!('S', '4', '0', '1'),
    /// S5C73MX interleaved UYVY/JPEG
    V4L2_PIX_FMT_S5C_UYVY_JPG = v4l2_fourcc!('S', '5', 'C', 'I'),
    /// Interleaved 8-bit Greyscale
    V4L2_PIX_FMT_Y8I = v4l2_fourcc!('Y', '8', 'I', ' '),
    /// Interleaved 12-bit Greyscale
    V4L2_PIX_FMT_Y12I = v4l2_fourcc!('Y', '1', '2', 'I'),
    /// Interleaved 16-bit Greyscale
    V4L2_PIX_FMT_Y16I = v4l2_fourcc!('Y', '1', '6', 'I'),
    /// 16-bit Depth
    V4L2_PIX_FMT_Z16 = v4l2_fourcc!('Z', '1', '6', ' '),
    /// Mediatek Compressed Format
    V4L2_PIX_FMT_MT21C = v4l2_fourcc!('M', 'T', '2', '1'),
    /// Mediatek 8-bit Block Format
    V4L2_PIX_FMT_MM21 = v4l2_fourcc!('M', 'M', '2', '1'),
    /// Mediatek 10bit Tile Mode
    V4L2_PIX_FMT_MT2110T = v4l2_fourcc!('M', 'T', '2', 'T'),
    /// Mediatek 10bit Raster Mode
    V4L2_PIX_FMT_MT2110R = v4l2_fourcc!('M', 'T', '2', 'R'),
    /// Planar 10:16 Greyscale Depth
    V4L2_PIX_FMT_INZI = v4l2_fourcc!('I', 'N', 'Z', 'I'),
    /// 4-bit Depth Confidence (Packed)
    V4L2_PIX_FMT_CNF4 = v4l2_fourcc!('C', 'N', 'F', '4'),
    /// 8-bit Dithered RGB (BTTV)
    V4L2_PIX_FMT_HI240 = v4l2_fourcc!('H', 'I', '2', '4'),
    /// QCOM Compressed 8-bit Format
    V4L2_PIX_FMT_QC08C = v4l2_fourcc!('Q', '0', '8', 'C'),
    /// QCOM Compressed 10-bit Format
    V4L2_PIX_FMT_QC10C = v4l2_fourcc!('Q', '1', '0', 'C'),
    /// Aspeed JPEG
    V4L2_PIX_FMT_AJPG = v4l2_fourcc!('A', 'J', 'P', 'G'),
    /// Hextile Compressed Format
    V4L2_PIX_FMT_HEXTILE = v4l2_fourcc!('H', 'X', 'T', 'L'),

    //  10bit raw packed, 32 bytes for every 25 pixels, last LSB 6 bits unused
    /// IPU3 packed 10-bit BGGR bayer
    V4L2_PIX_FMT_IPU3_SBGGR10 = v4l2_fourcc!('i', 'p', '3', 'b'),
    /// IPU3 packed 10-bit GBRG bayer
    V4L2_PIX_FMT_IPU3_SGBRG10 = v4l2_fourcc!('i', 'p', '3', 'g'),
    /// IPU3 packed 10-bit GRBG bayer
    V4L2_PIX_FMT_IPU3_SGRBG10 = v4l2_fourcc!('i', 'p', '3', 'G'),
    /// IPU3 packed 10-bit RGGB bayer
    V4L2_PIX_FMT_IPU3_SRGGB10 = v4l2_fourcc!('i', 'p', '3', 'r'),

    //  Raspberry Pi PiSP compressed formats.
    /// `PiSP` 8-bit mode 1 compressed RGGB bayer
    V4L2_PIX_FMT_PISP_COMP1_RGGB = v4l2_fourcc!('P', 'C', '1', 'R'),
    /// `PiSP` 8-bit mode 1 compressed GRBG bayer
    V4L2_PIX_FMT_PISP_COMP1_GRBG = v4l2_fourcc!('P', 'C', '1', 'G'),
    /// `PiSP` 8-bit mode 1 compressed GBRG bayer
    V4L2_PIX_FMT_PISP_COMP1_GBRG = v4l2_fourcc!('P', 'C', '1', 'g'),
    /// `PiSP` 8-bit mode 1 compressed BGGR bayer
    V4L2_PIX_FMT_PISP_COMP1_BGGR = v4l2_fourcc!('P', 'C', '1', 'B'),
    /// `PiSP` 8-bit mode 1 compressed monochrome
    V4L2_PIX_FMT_PISP_COMP1_MONO = v4l2_fourcc!('P', 'C', '1', 'M'),
    /// `PiSP` 8-bit mode 2 compressed RGGB bayer
    V4L2_PIX_FMT_PISP_COMP2_RGGB = v4l2_fourcc!('P', 'C', '2', 'R'),
    /// `PiSP` 8-bit mode 2 compressed GRBG bayer
    V4L2_PIX_FMT_PISP_COMP2_GRBG = v4l2_fourcc!('P', 'C', '2', 'G'),
    /// `PiSP` 8-bit mode 2 compressed GBRG bayer
    V4L2_PIX_FMT_PISP_COMP2_GBRG = v4l2_fourcc!('P', 'C', '2', 'g'),
    /// `PiSP` 8-bit mode 2 compressed BGGR bayer
    V4L2_PIX_FMT_PISP_COMP2_BGGR = v4l2_fourcc!('P', 'C', '2', 'B'),
    /// `PiSP` 8-bit mode 2 compressed monochrome
    V4L2_PIX_FMT_PISP_COMP2_MONO = v4l2_fourcc!('P', 'C', '2', 'M'),

    //  SDR formats - used only for Software Defined Radio devices
    /// IQ unsigned 8-bit
    V4L2_SDR_FMT_CU8 = v4l2_fourcc!('C', 'U', '0', '8'),
    /// IQ unsigned 16-bit Little Endian
    V4L2_SDR_FMT_CU16LE = v4l2_fourcc!('C', 'U', '1', '6'),
    /// Complex signed 8-bit
    V4L2_SDR_FMT_CS8 = v4l2_fourcc!('C', 'S', '0', '8'),
    /// Complex signed 14-bit Little Endian
    V4L2_SDR_FMT_CS14LE = v4l2_fourcc!('C', 'S', '1', '4'),
    /// Real unsigned 12-bit Little Endian
    V4L2_SDR_FMT_RU12LE = v4l2_fourcc!('R', 'U', '1', '2'),
    /// Planar Complex unsigned 16-bit Big Endian
    V4L2_SDR_FMT_PCU16BE = v4l2_fourcc!('P', 'C', '1', '6'),
    /// Planar Complex unsigned 18-bit Big Endian
    V4L2_SDR_FMT_PCU18BE = v4l2_fourcc!('P', 'C', '1', '8'),
    /// Planar Complex unsigned 20-bit Big Endian
    V4L2_SDR_FMT_PCU20BE = v4l2_fourcc!('P', 'C', '2', '0'),

    //  Touch formats - used for Touch devices
    /// 16-bit signed deltas
    V4L2_TCH_FMT_DELTA_TD16 = v4l2_fourcc!('T', 'D', '1', '6'),
    /// 8-bit signed deltas
    V4L2_TCH_FMT_DELTA_TD08 = v4l2_fourcc!('T', 'D', '0', '8'),
    /// 16-bit unsigned touch data
    V4L2_TCH_FMT_TU16 = v4l2_fourcc!('T', 'U', '1', '6'),
    /// 8-bit unsigned touch data
    V4L2_TCH_FMT_TU08 = v4l2_fourcc!('T', 'U', '0', '8'),

    //  Meta-data formats
    /// R-Car VSP1 1-D Histogram
    V4L2_META_FMT_VSP1_HGO = v4l2_fourcc!('V', 'S', 'P', 'H'),
    /// R-Car VSP1 2-D Histogram
    V4L2_META_FMT_VSP1_HGT = v4l2_fourcc!('V', 'S', 'P', 'T'),
    /// UVC Payload Header metadata
    V4L2_META_FMT_UVC = v4l2_fourcc!('U', 'V', 'C', 'H'),
    /// D4XX Payload Header metadata
    V4L2_META_FMT_D4XX = v4l2_fourcc!('D', '4', 'X', 'X'),
    /// Vivid Metadata
    V4L2_META_FMT_VIVID = v4l2_fourcc!('V', 'I', 'V', 'D'),

    //  Vendor specific - used for RK_ISP1 camera sub-system
    /// Rockchip ISP1 3A Parameters
    V4L2_META_FMT_RK_ISP1_PARAMS = v4l2_fourcc!('R', 'K', '1', 'P'),
    /// Rockchip ISP1 3A Statistics
    V4L2_META_FMT_RK_ISP1_STAT_3A = v4l2_fourcc!('R', 'K', '1', 'S'),
    /// Rockchip ISP1 Extensible 3A Parameters
    V4L2_META_FMT_RK_ISP1_EXT_PARAMS = v4l2_fourcc!('R', 'K', '1', 'E'),

    //  Vendor specific - used for RaspberryPi PiSP
    /// `PiSP` Backend (BE) configuration
    V4L2_META_FMT_RPI_BE_CFG = v4l2_fourcc!('R', 'P', 'B', 'C'),
    /// `PiSP` Frontend (FE) configuration
    V4L2_META_FMT_RPI_FE_CFG = v4l2_fourcc!('R', 'P', 'F', 'C'),
    /// `PiSP` Frontend (FE) statistics
    V4L2_META_FMT_RPI_FE_STATS = v4l2_fourcc!('R', 'P', 'F', 'S'),
}

#[cfg(test)]
mod tests_v4l2_pix_fmt {
    use crate::format;

    #[test]
    fn layout() {
        assert_eq!(size_of::<format::v4l2_pix_fmt>(), size_of::<u32>());

        assert_eq!(align_of::<format::v4l2_pix_fmt>(), align_of::<u32>());
    }
}

pub use crate::raw::v4l2_mbus_pixelcode;

impl TryFrom<v4l2_pix_fmt> for v4l2_mbus_pixelcode {
    type Error = ConversionError;

    fn try_from(value: v4l2_pix_fmt) -> Result<Self, Self::Error> {
        #[expect(
            clippy::wildcard_enum_match_arm,
            reason = "Not all v4l2 pixel formats have media bus equivalents"
        )]
        match value {
            v4l2_pix_fmt::V4L2_PIX_FMT_RGB24 => Ok(Self::V4L2_MBUS_FMT_RGB888_1X24),
            _ => Err(Self::Error::InvalidValue(format!("{value:#?}"))),
        }
    }
}

impl fmt::Display for v4l2_mbus_pixelcode {
    #[expect(
        clippy::unwrap_in_result,
        reason = "We know that we're working with an enum here."
    )]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let peek = Peek::new(self)
            .into_enum()
            .expect("We know we have an enum");

        if let Ok(name) = peek.variant_name_active() {
            f.write_fmt(format_args!("{name}/{:x}", u32::from(*self)))
        } else {
            f.write_fmt(format_args!("Unknown/{:x}", u32::from(*self)))
        }
    }
}
