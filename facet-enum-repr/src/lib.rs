//! Implements the `FacetEnumRepr` derive trait, which implements `TryFrom` and `From` traits for
//! their internal representation

#[doc(hidden)]
pub use facet::Facet;
#[doc(hidden)]
pub use facet_reflect::peek_enum;
use thiserror::Error;

/// The error type returned by the `TryFrom` implementations
#[derive(Debug, Error, PartialEq)]
pub enum TryFromReprError<T>
where
    T: core::fmt::Debug,
{
    /// The value being converted doesn't match any variant discriminant.
    #[error("Unknown value {0}")]
    UnknownValue(T),
}

pub use facet_enum_repr_derive::FacetEnumRepr;
