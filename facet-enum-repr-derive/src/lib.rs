#![allow(missing_docs)]
#![allow(clippy::missing_panics_doc)]

use core::fmt::Write as _;
use std::collections::HashMap;

use facet_macros_parse::{AttributeInner, Parse as _};
use proc_macro::TokenStream;
use unsynn::{ToTokens as _, TokenTree};

fn find_enum_repr(val: &facet_macros_parse::Enum) -> String {
    val.attributes
        .iter()
        .filter_map(|attr| {
            if let AttributeInner::Repr(repr_attr) = &attr.body.content {
                if repr_attr.attr.content.0.is_empty() {
                    // treat empty repr as non-existent
                    // (this shouldn't be possible, but just in case)
                    None
                } else {
                    Some(repr_attr)
                }
            } else {
                None
            }
        })
        .flat_map(|repr_attr| repr_attr.attr.content.0.iter())
        .next()
        .map_or(String::from("isize"), |a| a.value.to_string())
}

fn generate_output(
    name: &str,
    repr: &str,
    attrs_map: &HashMap<String, Vec<String>>,
) -> TokenStream {
    let mut output = format!(
        "
#[automatically_derived]
impl TryFrom<{repr}> for {name} {{
    type Error = facet_enum_repr::TryFromReprError<{repr}>;

    fn try_from(value: {repr}) -> Result<Self, Self::Error> {{
        let shape = <{name} as facet_enum_repr::Facet>::SHAPE;
        let enum_repr = facet_enum_repr::peek_enum(shape).unwrap();

        for variant in enum_repr.variants {{
            let disc_repr: {repr} = variant
                .discriminant
                .unwrap()
                .try_into()
                .expect(\"Our discriminant value must fit into its enum repr type.\");

            if disc_repr == value {{
                return Ok(unsafe {{ std::mem::transmute(value) }});
            }}
        }}

        Err(Self::Error::UnknownValue(value))
    }}
}}

#[automatically_derived]
impl From<{name}> for {repr} {{
    fn from(value: {name}) -> Self {{
        value as Self
    }}
}}
"
    );

    for (attr, values) in attrs_map {
        match attr.as_str() {
            "panic_into" => {
                for repr_type in values {
                    write!(
                        output,
                        "
#[automatically_derived]
impl From<{name}> for {repr_type} {{
    fn from(value: {name}) -> Self {{
        {repr}::from(value)
            .try_into()
            .expect(\"All {name} values fit into a {repr_type}.\")
    }}
}}
                    ",
                    )
                    .expect("write!() never fails.");
                }
            }
            _ => unimplemented!(),
        }
    }

    output.into_token_stream().into()
}

#[proc_macro_derive(FacetEnumRepr, attributes(facet_enum_repr))]
pub fn facet_enum_repr_derive(input: TokenStream) -> TokenStream {
    let utstream = unsynn::TokenStream::from(input.clone());
    let mut i = utstream.to_token_iter();

    let Ok(val) = facet_macros_parse::Enum::parse(&mut i) else {
        return r#"
compile_error!("FacetEnumRepr only works on enums.")
            "#
        .into_token_stream()
        .into();
    };
    let name = val.name.to_string();
    let repr = find_enum_repr(&val);

    let attrs_trees = val
        .attributes
        .iter()
        .filter_map(|attr| {
            if let AttributeInner::Any(any_attr) = &attr.body.content {
                Some(any_attr)
            } else {
                None
            }
        })
        .flat_map(|any_attr| any_attr.chunks(2))
        .find_map(|any_attr| {
            assert_eq!(
                any_attr.len(),
                2,
                "We must process attributes chunks two by two."
            );

            let ident_name = if let TokenTree::Ident(ident) = &any_attr[0] {
                ident.to_string()
            } else {
                return None;
            };

            if ident_name == "facet_enum_repr" {
                if let TokenTree::Group(group) = &any_attr[1] {
                    Some(group.stream().into_iter().collect::<Vec<TokenTree>>())
                } else {
                    unimplemented!()
                }
            } else {
                None
            }
        })
        .unwrap_or(Vec::new());

    let macro_attributes = attrs_trees
        .chunks(2)
        .filter_map(|chunk| {
            assert_eq!(
                chunk.len(),
                2,
                "We must process attributes chunks two by two."
            );

            if let TokenTree::Ident(ident) = &chunk[0] {
                Some((ident.to_string(), chunk[1].clone()))
            } else {
                None
            }
        })
        .filter_map(|(name, attr)| {
            if let TokenTree::Group(group) = attr {
                Some((
                    name,
                    group
                        .stream()
                        .into_iter()
                        .filter_map(|t| {
                            if let TokenTree::Ident(ident) = t {
                                Some(ident.to_string())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>(),
                ))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    generate_output(&name, &repr, &HashMap::from_iter(macro_attributes))
}
