use crate::error::{FieldError, ParseError};
use proc_macro2::Ident;
use std::iter::FromIterator;
use syn::Error as SynError;
use syn::{Data, DeriveInput, Field, Fields, FieldsNamed, Lit, Meta, NestedMeta, Type};

pub(crate) struct FieldData<'a> {
    pub(crate) name: &'a Ident,
    pub(crate) ty: &'a Type,
    pub(crate) meta: PacketFieldMeta,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum PacketFieldMeta {
    With { module: String },
    MaxLength { length: usize },
    Empty,
}

pub(crate) fn parse_derive_input(
    input: &DeriveInput,
) -> Result<(&Ident, Vec<FieldData>), ParseError> {
    let name = &input.ident;

    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(named_fields) => Ok((name, parse_fields(named_fields)?)),
            _ => Err(ParseError::UnnamedFields),
        },
        _ => Err(ParseError::NotStruct { name }),
    }
}

fn parse_fields(named_fields: &FieldsNamed) -> Result<Vec<FieldData>, ParseError> {
    let mut fields_data = Vec::new();

    for field in named_fields.named.iter() {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let nested_metas = parse_field_nested_metas(field)?;
        let meta = parse_packet_field_meta(nested_metas)?;

        fields_data.push(FieldData { name, ty, meta })
    }

    Ok(fields_data)
}

fn parse_packet_field_meta(
    nested_metas: Vec<NestedMeta>,
) -> Result<PacketFieldMeta, ParseError<'static>> {
    let meta_parsers: Vec<fn(&NestedMeta) -> Result<PacketFieldMeta, FieldError>> =
        vec![get_module_field_meta, get_max_length_field_meta];

    for nested_meta in nested_metas.iter() {
        for meta_parser in meta_parsers.iter() {
            let packet_field_meta = meta_parser(nested_meta)?;

            if packet_field_meta != PacketFieldMeta::Empty {
                return Ok(packet_field_meta);
            }
        }
    }

    Ok(PacketFieldMeta::Empty)
}

fn parse_field_nested_metas(field: &Field) -> Result<Vec<NestedMeta>, ParseError<'_>> {
    let parsed_metas = field
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("packet"))
        .map(|a| a.parse_meta())
        .collect::<Result<Vec<Meta>, SynError>>()?;

    let nested_metas = parsed_metas
        .into_iter()
        .map(|m| match m {
            Meta::List(meta_list) => Ok(Vec::from_iter(meta_list.nested)),
            _ => Err(FieldError::NonListAttributes),
        })
        .collect::<Result<Vec<Vec<NestedMeta>>, FieldError>>()?;

    Ok(nested_metas.into_iter().flatten().collect())
}

fn get_module_field_meta(nested_meta: &NestedMeta) -> Result<PacketFieldMeta, FieldError> {
    if let NestedMeta::Meta(Meta::NameValue(named_meta)) = nested_meta {
        if matches!(&named_meta.path, path if path.is_ident("with")) {
            return match &named_meta.lit {
                Lit::Str(lit_str) => Ok(PacketFieldMeta::With {
                    module: lit_str.value(),
                }),
                _ => Err(FieldError::AttributeValueNotString),
            };
        }
    }

    Ok(PacketFieldMeta::Empty)
}

fn get_max_length_field_meta(nested_meta: &NestedMeta) -> Result<PacketFieldMeta, FieldError> {
    if let NestedMeta::Meta(Meta::NameValue(named_meta)) = nested_meta {
        if matches!(&named_meta.path, path if path.is_ident("max_length")) {
            return match &named_meta.lit {
                Lit::Int(lit_int) => Ok(PacketFieldMeta::MaxLength {
                    length: lit_int.base10_parse::<usize>()?,
                }),
                _ => Err(FieldError::AttributeValueNotInteger),
            };
        }
    }

    Ok(PacketFieldMeta::Empty)
}
