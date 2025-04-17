#![allow(non_camel_case_types)]
#![allow(unsafe_code)]
#![doc = include_str!("../README.md")]

use core::{error, fmt};

use facet_enum_repr::TryFromReprError;

/// An Error Happened when converting values from on type to another
#[derive(Debug)]
pub enum ConversionError {
    /// An invalid Value
    InvalidValue(String),

    /// An error occurred while converting multiple fields in a struct
    InvalidStructField {
        /// Name of the field the error occurred with
        name: String,
        /// Value the error occurred withs
        value: String,
    },
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            match self {
                ConversionError::InvalidValue(v) => format!("Invalid Value: {v}"),
                ConversionError::InvalidStructField { name, value } => {
                    format!("Invalid Field {name} Value {value}")
                }
            }
            .as_str(),
        )
    }
}

impl error::Error for ConversionError {}

impl<T> From<TryFromReprError<T>> for ConversionError
where
    T: fmt::Debug,
{
    fn from(value: TryFromReprError<T>) -> Self {
        Self::InvalidValue(format!("{value:#?}"))
    }
}

/// V4L2 Pixel Formats
pub mod format;

/// Raw, unsafe, abstraction
pub mod raw;

/// Safe abstraction
pub mod wrapper;
