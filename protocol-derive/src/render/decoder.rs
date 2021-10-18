use crate::parse::{AttributeData, BitfieldPosition, DiscriminantType, FieldData, VariantData};
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

pub(crate) fn render_enum_decoder(
    name: &Ident,
    discriminant_type: &DiscriminantType,
    variants: &Vec<VariantData>,
) -> TokenStream2 {
    let render_variants = render_variants(discriminant_type, variants);
    let render_discriminant_type = render_discriminant_type(discriminant_type);

    quote! {
        #[automatically_derived]
        impl crate::decoder::Decoder for #name {
            type Output = Self;

            fn decode<R: std::io::Read>(reader: &mut R) -> Result<Self::Output, crate::error::DecodeError> {
                let type_id = #render_discriminant_type;

                match type_id {
                    #render_variants
                    _ => Err(DecodeError::UnknownEnumType { type_id: type_id as usize, }),
                }
            }
        }
    }
}

fn render_variants(
    discriminant_type: &DiscriminantType,
    variants: &Vec<VariantData>,
) -> TokenStream2 {
    variants
        .iter()
        .map(|v| render_variant(discriminant_type, v))
        .collect()
}

fn render_variant(discriminant_type: &DiscriminantType, variant: &VariantData) -> TokenStream2 {
    if variant.fields.is_empty() {
        render_unit_variant(discriminant_type, variant)
    } else {
        render_struct_variant(discriminant_type, variant)
    }
}

fn render_unit_variant(
    discriminant_type: &DiscriminantType,
    variant: &VariantData,
) -> TokenStream2 {
    let discriminant = render_discriminant(discriminant_type, variant.discriminant);
    let name = variant.name;

    quote! {
        #discriminant => Ok(Self::#name),
    }
}

fn render_struct_variant(
    discriminant_type: &DiscriminantType,
    variant: &VariantData,
) -> TokenStream2 {
    let discriminant = render_discriminant(discriminant_type, variant.discriminant);
    let name = variant.name;
    let fields = &variant.fields;

    let field_names_joined_comma = render_field_names_joined_comma(fields);
    let render_fields = render_fields(fields);

    quote! {
        #discriminant => {
            #render_fields

            Ok(Self::#name {
                #field_names_joined_comma
            })
        }
    }
}

fn render_discriminant_type(discriminant_type: &DiscriminantType) -> TokenStream2 {
    match discriminant_type {
        DiscriminantType::UnsignedByte => {
            quote!(reader.read_u8()?;)
        }
        DiscriminantType::VarInt => {
            quote!(reader.read_var_i32()?;)
        }
    }
}

fn render_discriminant(discriminant_type: &DiscriminantType, discriminant: usize) -> TokenStream2 {
    match discriminant_type {
        DiscriminantType::UnsignedByte => {
            let u8 = discriminant as u8;
            quote!(#u8)
        }
        DiscriminantType::VarInt => {
            let i32 = discriminant as i32;
            quote!(#i32)
        }
    }
}

fn render_field_names_joined_comma(fields: &Vec<FieldData>) -> TokenStream2 {
    fields.iter().map(|f| f.name).map(|n| quote!(#n,)).collect()
}

fn render_fields(fields: &Vec<FieldData>) -> TokenStream2 {
    fields.iter().map(|f| render_field(f)).collect()
}

fn render_field(field: &FieldData) -> TokenStream2 {
    let name = field.name;
    let ty = field.ty;

    match &field.attribute {
        AttributeData::With { module } => render_with_field(name, module),
        AttributeData::MaxLength { length } => render_max_length_field(name, *length as u16),
        AttributeData::Bitfield { idx, position } => render_bitfield(name, *idx, position),
        AttributeData::Empty => render_simple_field(name, ty),
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

fn render_bitfield(name: &Ident, idx: u8, position: &BitfieldPosition) -> TokenStream2 {
    let mask = 1u8 << idx;

    let render_mask = quote! {
        let #name = flags & #mask > 0;
    };

    match position {
        BitfieldPosition::Start => {
            quote! {
              let flags = reader.read_u8()?;

              #render_mask
            }
        }
        _ => render_mask,
    }
}
