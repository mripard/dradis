pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Invalid,
    Empty,
    FileNotFound,
    NotSupported,
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_err: std::str::Utf8Error) -> Self {
        Error::Invalid
    }
}

impl From<nix::Error> for Error {
    fn from(err: nix::Error) -> Self {
        match err {
            nix::Error::Sys(e) => {
                Error::Io(std::io::Error::from_raw_os_error(e as i32))
            },
            nix::Error::InvalidPath => {
                Error::Invalid
            },
            nix::Error::InvalidUtf8 => {
                Error::Invalid
            },
            nix::Error::UnsupportedOperation => {
                Error::NotSupported
            },
        }
    }
}