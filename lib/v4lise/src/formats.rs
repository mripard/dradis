macro_rules! fourcc_code {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        (($a as u32) | (($b as u32) << 8) | (($c as u32) << 16) | (($d as u32) << 24)) as u32
    };
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u32)]
pub enum Format {
    MJPG = fourcc_code!('M', 'J', 'P', 'G'),
    YUYV = fourcc_code!('Y', 'U', 'Y', 'V'),
    RGB24 = fourcc_code!('R', 'G', 'B', '3'),
}
