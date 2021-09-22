use crate::error::{AttributeError, DeriveInputParserError};
use proc_macro2::Ident;
use std::iter::FromIterator;
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Data, DeriveInput, ExprLit, Fields, FieldsNamed, Lit, Meta, NestedMeta, Type,
};
use syn::{Error as SynError, Variant};
use syn::{Expr, Token};

pub(crate) enum DeriveInputParseResult<'a> {
    Struct {
        name: &'a Ident,
        fields: Vec<FieldData<'a>>,
    },
    Enum {
        name: &'a Ident,
        discriminant_type: DiscriminantType,
        variants: Vec<VariantData<'a>>,
    },
}

pub(crate) struct VariantData<'a> {
    pub(crate) discriminant: usize,
    pub(crate) name: &'a Ident,
    pub(crate) fields: Vec<FieldData<'a>>,
}

pub(crate) struct FieldData<'a> {
    pub(crate) name: &'a Ident,
    pub(crate) ty: &'a Type,
    pub(crate) attribute: AttributeData,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AttributeData {
    With { module: String },
    MaxLength { length: usize },
    Empty,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum DiscriminantType {
    UnsignedByte,
    VarInt,
}

pub(crate) fn parse_derive_input(
    input: &DeriveInput,
) -> Result<DeriveInputParseResult, DeriveInputParserError> {
    let name = &input.ident;

    match &input.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(named_fields) => {
                let fields = parse_fields(named_fields)?;

                Ok(DeriveInputParseResult::Struct { name, fields })
            }
            _ => Err(DeriveInputParserError::UnnamedDataFields),
        },
        Data::Enum(data_enum) => {
            let variants = parse_variants(&data_enum.variants)?;
            let discriminant_type = parse_discriminant_type(&input.attrs)?;

            Ok(DeriveInputParseResult::Enum {
                name,
                discriminant_type,
                variants,
            })
        }
        _ => Err(DeriveInputParserError::UnsupportedData),
    }
}

fn parse_discriminant_type(
    attributes: &Vec<Attribute>,
) -> Result<DiscriminantType, DeriveInputParserError> {
    let nested_metas = parse_attributes_nested_metas(attributes)?;
    let attribute = parse_attribute(nested_metas)?;

    match attribute {
        AttributeData::With { module } if module == "var_int" => Ok(DiscriminantType::VarInt),
        _ => Ok(DiscriminantType::UnsignedByte),
    }
}

fn parse_variants(
    variants: &Punctuated<Variant, Token![,]>,
) -> Result<Vec<VariantData>, DeriveInputParserError> {
    variants
        .iter()
        .enumerate()
        .map(|(idx, v)| parse_variant(idx, v))
        .collect()
}

fn parse_variant(idx: usize, variant: &Variant) -> Result<VariantData, DeriveInputParserError> {
    let discriminant = parse_variant_discriminant(variant).unwrap_or(idx);
    let name = &variant.ident;

    let fields = match &variant.fields {
        Fields::Named(named_fields) => parse_fields(named_fields),
        Fields::Unit => Ok(Vec::new()),
        _ => Err(DeriveInputParserError::UnnamedDataFields),
    }?;

    Ok(VariantData {
        discriminant,
        name,
        fields,
    })
}

fn parse_variant_discriminant(variant: &Variant) -> Option<usize> {
    variant
        .discriminant
        .as_ref()
        .and_then(|(_, expr)| match expr {
            Expr::Lit(ExprLit {
                lit: Lit::Int(lit_int),
                ..
            }) => lit_int.base10_parse().ok(),
            _ => None,
        })
}

fn parse_fields(named_fields: &FieldsNamed) -> Result<Vec<FieldData>, DeriveInputParserError> {
    let mut fields_data = Vec::new();

    for field in named_fields.named.iter() {
        let name = field.ident.as_ref().unwrap();
        let ty = &field.ty;

        let nested_metas = parse_attributes_nested_metas(&field.attrs)?;
        let attribute = parse_attribute(nested_metas)?;

        fields_data.push(FieldData {
            name,
            ty,
            attribute,
        })
    }

    Ok(fields_data)
}

fn parse_attributes_nested_metas(
    attributes: &Vec<Attribute>,
) -> Result<Vec<NestedMeta>, DeriveInputParserError> {
    let parsed_metas = attributes
        .iter()
        .filter(|a| a.path.is_ident("data_type"))
        .map(|a| a.parse_meta())
        .collect::<Result<Vec<Meta>, SynError>>()?;

    let nested_metas = parsed_metas
        .into_iter()
        .map(|m| match m {
            Meta::List(meta_list) => Ok(Vec::from_iter(meta_list.nested)),
            _ => Err(AttributeError::UnsupportedAttribute),
        })
        .collect::<Result<Vec<Vec<NestedMeta>>, AttributeError>>()?;

    Ok(nested_metas.into_iter().flatten().collect())
}

fn parse_attribute(nested_metas: Vec<NestedMeta>) -> Result<AttributeData, DeriveInputParserError> {
    let attribute_parsers: Vec<fn(&NestedMeta) -> Result<AttributeData, AttributeError>> =
        vec![get_module_attribute, get_max_length_attribute];

    for nested_meta in nested_metas.iter() {
        for attribute_parser in attribute_parsers.iter() {
            let attribute = attribute_parser(nested_meta)?;

            if attribute != AttributeData::Empty {
                return Ok(attribute);
            }
        }
    }

    Ok(AttributeData::Empty)
}

fn get_module_attribute(nested_meta: &NestedMeta) -> Result<AttributeData, AttributeError> {
    if let NestedMeta::Meta(Meta::NameValue(named_meta)) = nested_meta {
        if matches!(&named_meta.path, path if path.is_ident("with")) {
            return match &named_meta.lit {
                Lit::Str(lit_str) => Ok(AttributeData::With {
                    module: lit_str.value(),
                }),
                _ => Err(AttributeError::AttributeWrongValueType),
            };
        }
    }

    Ok(AttributeData::Empty)
}

fn get_max_length_attribute(nested_meta: &NestedMeta) -> Result<AttributeData, AttributeError> {
    if let NestedMeta::Meta(Meta::NameValue(named_meta)) = nested_meta {
        if matches!(&named_meta.path, path if path.is_ident("max_length")) {
            return match &named_meta.lit {
                Lit::Int(lit_int) => Ok(AttributeData::MaxLength {
                    length: lit_int.base10_parse()?,
                }),
                _ => Err(AttributeError::AttributeWrongValueType),
            };
        }
    }

    Ok(AttributeData::Empty)
}
