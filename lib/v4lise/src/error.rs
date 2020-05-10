pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Invalid,
    Empty,
    FileNotFound,
    NotSupported,
}

impl From<std::io::Error> for Error {
    fn from(_err: std::io::Error) -> Self {
        Error::FileNotFound
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_err: std::str::Utf8Error) -> Self {
        Error::Invalid
    }
}