use crate::error::{DeriveInputParserError, FieldError};
use proc_macro2::Ident;
use std::iter::FromIterator;
use syn::Error as SynError;
use syn::{Data, DeriveInput, Field, Fields, FieldsNamed, Lit, Meta, NestedMeta, Type};

pub(crate) struct FieldData<'a> {
    pub(crate) name: &'a Ident,
    pub(crate) ty: &'a Type,
    pub(crate) attribute: Attribute,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum Attribute {
    With { module: String },
    MaxLength { length: usize },
    Empty,
}

pub(crate) fn parse_derive_input(
    input: &DeriveInput,
) -> Result<(&Ident, Vec<FieldData>), DeriveInputParserError> {
    let name = &input.ident;

    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named_fields) => Ok((name, parse_fields(named_fields)?)),
            _ => Err(DeriveInputParserError::UnnamedDataFields),
        },
        _ => Err(DeriveInputParserError::UnsupportedData),
    }
}

fn parse_fields(named_fields: &FieldsNamed) -> Result<Vec<FieldData>, DeriveInputParserError> {
    let mut fields_data = Vec::new();

    for field in named_fields.named.iter() {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let nested_metas = parse_field_nested_metas(field)?;
        let attribute = parse_attribute(nested_metas)?;

        fields_data.push(FieldData {
            name,
            ty,
            attribute,
        })
    }

    Ok(fields_data)
}

fn parse_field_nested_metas(field: &Field) -> Result<Vec<NestedMeta>, DeriveInputParserError> {
    let parsed_metas = field
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("data_type"))
        .map(|a| a.parse_meta())
        .collect::<Result<Vec<Meta>, SynError>>()?;

    let nested_metas = parsed_metas
        .into_iter()
        .map(|m| match m {
            Meta::List(meta_list) => Ok(Vec::from_iter(meta_list.nested)),
            _ => Err(FieldError::UnsupportedAttribute),
        })
        .collect::<Result<Vec<Vec<NestedMeta>>, FieldError>>()?;

    Ok(nested_metas.into_iter().flatten().collect())
}

fn parse_attribute(nested_metas: Vec<NestedMeta>) -> Result<Attribute, DeriveInputParserError> {
    let attribute_parsers: Vec<fn(&NestedMeta) -> Result<Attribute, FieldError>> =
        vec![get_module_attribute, get_max_length_attribute];

    for nested_meta in nested_metas.iter() {
        for attribute_parser in attribute_parsers.iter() {
            let attribute = attribute_parser(nested_meta)?;

            if attribute != Attribute::Empty {
                return Ok(attribute);
            }
        }
    }

    Ok(Attribute::Empty)
}

fn get_module_attribute(nested_meta: &NestedMeta) -> Result<Attribute, FieldError> {
    if let NestedMeta::Meta(Meta::NameValue(named_meta)) = nested_meta {
        if matches!(&named_meta.path, path if path.is_ident("with")) {
            return match &named_meta.lit {
                Lit::Str(lit_str) => Ok(Attribute::With {
                    module: lit_str.value(),
                }),
                _ => Err(FieldError::AttributeWrongValueType),
            };
        }
    }

    Ok(Attribute::Empty)
}

fn get_max_length_attribute(nested_meta: &NestedMeta) -> Result<Attribute, FieldError> {
    if let NestedMeta::Meta(Meta::NameValue(named_meta)) = nested_meta {
        if matches!(&named_meta.path, path if path.is_ident("max_length")) {
            return match &named_meta.lit {
                Lit::Int(lit_int) => Ok(Attribute::MaxLength {
                    length: lit_int.base10_parse::<usize>()?,
                }),
                _ => Err(FieldError::AttributeWrongValueType),
            };
        }
    }

    Ok(Attribute::Empty)
}
