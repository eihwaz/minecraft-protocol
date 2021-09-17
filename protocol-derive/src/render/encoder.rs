use crate::parse::{Attribute, FieldData, VariantData};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use quote::quote;

pub(crate) fn render_struct_encoder(name: &Ident, fields: &Vec<FieldData>) -> TokenStream2 {
    let render_fields = render_fields(fields, true);

    quote! {
        #[automatically_derived]
        impl crate::encoder::Encoder for #name {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<(), crate::error::EncodeError> {
                #render_fields

                Ok(())
            }
        }
    }
}

pub(crate) fn render_struct_variant_encoder(
    name: &Ident,
    variants: &Vec<VariantData>,
) -> TokenStream2 {
    let render_variants = render_variants(variants);

    quote! {
        #[automatically_derived]
        impl crate::encoder::Encoder for #name {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<(), crate::error::EncodeError> {
                match self {
                    #render_variants
                }

                Ok(())
            }
        }
    }
}

fn render_variants(variants: &Vec<VariantData>) -> TokenStream2 {
    variants.iter().map(|v| render_variant(v)).collect()
}

fn render_variant(variant: &VariantData) -> TokenStream2 {
    let idx = variant.idx;
    let name = variant.name;
    let fields = &variant.fields;

    if fields.is_empty() {
        quote! {
            Self::#name => {
                writer.write_u8(#idx)?;
            }
        }
    } else {
        let field_names_joined_comma = render_field_names_joined_comma(fields);
        let render_fields = render_fields(fields, false);

        quote! {
            Self::#name {
                #field_names_joined_comma
            } => {
                writer.write_u8(#idx)?;

                #render_fields
            }
        }
    }
}

fn render_field_names_joined_comma(fields: &Vec<FieldData>) -> TokenStream2 {
    fields.iter().map(|f| f.name).map(|n| quote!(#n,)).collect()
}

fn render_fields(fields: &Vec<FieldData>, with_self: bool) -> TokenStream2 {
    fields.iter().map(|f| render_field(f, with_self)).collect()
}

fn render_field(field: &FieldData, with_self: bool) -> TokenStream2 {
    let name = field.name;

    match &field.attribute {
        Attribute::With { module } => render_with_field(name, module, with_self),
        Attribute::MaxLength { length } => render_max_length_field(name, *length as u16, with_self),
        Attribute::Empty => render_simple_field(name, with_self),
    }
}

fn render_simple_field(name: &Ident, with_self: bool) -> TokenStream2 {
    render_with_field(name, "Encoder", with_self)
}

fn render_with_field(name: &Ident, module: &str, with_self: bool) -> TokenStream2 {
    let module_ident = Ident::new(module, Span::call_site());
    let final_name = get_field_final_name(name, with_self);

    quote! {
        crate::encoder::#module_ident::encode(#final_name, writer)?;
    }
}

fn render_max_length_field(name: &Ident, max_length: u16, with_self: bool) -> TokenStream2 {
    let final_name = get_field_final_name(name, with_self);

    quote! {
        crate::encoder::EncoderWriteExt::write_string(writer, &#final_name, #max_length)?;
    }
}

fn get_field_final_name(name: &Ident, with_self: bool) -> TokenStream2 {
    if with_self {
        quote!(&self.#name)
    } else {
        quote!(#name)
    }
}
