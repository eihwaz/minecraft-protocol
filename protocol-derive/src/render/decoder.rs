use crate::parse::{Attribute, FieldData};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use quote::quote;
use syn::Type;

pub(crate) fn render_decoder(name: &Ident, fields: &Vec<FieldData>) -> TokenStream2 {
    let struct_create = render_struct_create(name, fields);
    let render_fields = render_fields(fields);

    quote! {
        #[automatically_derived]
        impl crate::decoder::Decoder for #name {
            type Output = Self;

            fn decode<R: std::io::Read>(reader: &mut R) -> Result<Self::Output, crate::error::DecodeError> {
                #render_fields

                Ok(#struct_create)
            }
        }
    }
}

fn render_struct_create(name: &Ident, fields: &Vec<FieldData>) -> TokenStream2 {
    let struct_fields = fields
        .iter()
        .map(|f| f.name)
        .map(|n| quote!(#n,))
        .collect::<TokenStream2>();

    quote! {
        #name {
          #struct_fields
        }
    }
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
