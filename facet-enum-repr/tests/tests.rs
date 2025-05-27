#![allow(missing_docs)]
#![allow(unused_crate_dependencies)]

use facet::Facet;
use facet_enum_repr::{FacetEnumRepr, TryFromReprError};

#[repr(u32)]
#[derive(Debug, Facet, FacetEnumRepr, PartialEq)]
enum SimpleTestEnum {
    ValA,
    ValB,
}

#[test]
fn simple() {
    assert_eq!(SimpleTestEnum::try_from(0).unwrap(), SimpleTestEnum::ValA);
    assert_eq!(SimpleTestEnum::try_from(1).unwrap(), SimpleTestEnum::ValB);
    assert_eq!(
        SimpleTestEnum::try_from(2).unwrap_err(),
        TryFromReprError::<u32>::UnknownValue(2)
    );

    assert_eq!(u32::from(SimpleTestEnum::ValA), 0);
    assert_eq!(u32::from(SimpleTestEnum::ValB), 1);
}

#[repr(u32)]
#[derive(Debug, Facet, FacetEnumRepr, PartialEq)]
#[facet_enum_repr(panic_into(u8, u16))]
enum AttrsTestEnum {
    ValA,
    ValB,
}

#[test]
fn attributes() {
    assert_eq!(AttrsTestEnum::try_from(0).unwrap(), AttrsTestEnum::ValA);
    assert_eq!(AttrsTestEnum::try_from(1).unwrap(), AttrsTestEnum::ValB);
    assert_eq!(
        AttrsTestEnum::try_from(2).unwrap_err(),
        TryFromReprError::<u32>::UnknownValue(2)
    );

    assert_eq!(u32::from(AttrsTestEnum::ValA), 0);
    assert_eq!(u32::from(AttrsTestEnum::ValB), 1);

    assert_eq!(u16::from(AttrsTestEnum::ValA), 0);
    assert_eq!(u16::from(AttrsTestEnum::ValB), 1);

    assert_eq!(u8::from(AttrsTestEnum::ValA), 0);
    assert_eq!(u8::from(AttrsTestEnum::ValB), 1);
}

#[repr(u32)]
#[derive(Debug, Facet, FacetEnumRepr, PartialEq)]
#[facet_enum_repr(panic_into(u16))]
enum BrokenTestEnum {
    Val = 65_536, // This is u16::MAX + 1
}

#[test]
#[should_panic]
fn panics() {
    let _ = u16::from(BrokenTestEnum::Val);
}
