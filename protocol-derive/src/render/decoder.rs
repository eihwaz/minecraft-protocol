use crate::parse::{Attribute, FieldData, VariantData};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::Type;

pub(crate) fn render_struct_decoder(name: &Ident, fields: &Vec<FieldData>) -> TokenStream2 {
    let field_names_joined_comma = render_field_names_joined_comma(fields);
    let render_fields = render_fields(fields);

    quote! {
        #[automatically_derived]
        impl crate::decoder::Decoder for #name {
            type Output = Self;

            fn decode<R: std::io::Read>(reader: &mut R) -> Result<Self::Output, crate::error::DecodeError> {
                #render_fields

                Ok(#name {
                    #field_names_joined_comma
                })
            }
        }
    }
}

pub(crate) fn render_struct_variant_decoder(
    name: &Ident,
    variants: &Vec<VariantData>,
) -> TokenStream2 {
    let render_variants = render_variants(variants);

    quote! {
        #[automatically_derived]
        impl crate::decoder::Decoder for #name {
            type Output = Self;

            fn decode<R: std::io::Read>(reader: &mut R) -> Result<Self::Output, crate::error::DecodeError> {
                let type_id = reader.read_u8()?;

                match type_id {
                    #render_variants
                    _ => Err(DecodeError::UnknownEnumType { type_id }),
                }
            }
        }
    }
}

fn render_variants(variants: &Vec<VariantData>) -> TokenStream2 {
    variants
        .iter()
        .map(|v| render_variant(v))
        .flatten()
        .collect()
}

fn render_variant(variant: &VariantData) -> TokenStream2 {
    let idx = variant.idx;
    let name = variant.name;
    let fields = &variant.fields;

    if fields.is_empty() {
        quote! {
            #idx => Ok(Self::#name),
        }
    } else {
        let field_names_joined_comma = render_field_names_joined_comma(fields);
        let render_fields = render_fields(fields);

        quote! {
            #idx => {
                #render_fields

                Ok(Self::#name {
                    #field_names_joined_comma
                })
            }
        }
    }
}

fn render_field_names_joined_comma(fields: &Vec<FieldData>) -> TokenStream2 {
    fields.iter().map(|f| f.name).map(|n| quote!(#n,)).collect()
}

fn render_fields(fields: &Vec<FieldData>) -> TokenStream2 {
    fields.iter().map(|f| render_field(f)).flatten().collect()
}

fn render_field(field: &FieldData) -> TokenStream2 {
    let name = field.name;
    let ty = field.ty;

    match &field.attribute {
        Attribute::With { module } => render_with_field(name, module),
        Attribute::MaxLength { length } => render_max_length_field(name, *length as u16),
        Attribute::Empty => render_simple_field(name, ty),
    }
}

fn render_simple_field(name: &Ident, ty: &Type) -> TokenStream2 {
    quote! {
        let #name = <#ty as crate::decoder::Decoder>::decode(reader)?;
    }
}

fn render_with_field(name: &Ident, module: &str) -> TokenStream2 {
    let module_ident = Ident::new(module, Span::call_site());

    quote! {
        let #name = crate::decoder::#module_ident::decode(reader)?;
    }
}

fn render_max_length_field(name: &Ident, max_length: u16) -> TokenStream2 {
    quote! {
        let #name = crate::decoder::DecoderReadExt::read_string(reader, #max_length)?;
    }
}
