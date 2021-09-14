use crate::parse::{FieldData, PacketFieldMeta};
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use quote::quote;

pub(crate) fn render_encoder_trait(name: &Ident, fields: &Vec<FieldData>) -> TokenStream2 {
    let render_fields = render_fields(fields);

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

fn render_fields(fields: &Vec<FieldData>) -> TokenStream2 {
    fields.iter().map(|f| render_field(f)).flatten().collect()
}

fn render_field(field: &FieldData) -> TokenStream2 {
    let name = field.name;

    match &field.meta {
        PacketFieldMeta::With { module } => render_with_field(name, module),
        PacketFieldMeta::MaxLength { length } => render_max_length_field(name, *length as u16),
        PacketFieldMeta::Empty => render_simple_field(name),
    }
}

fn render_simple_field(name: &Ident) -> TokenStream2 {
    render_with_field(name, "Encoder")
}

fn render_with_field(name: &Ident, module: &str) -> TokenStream2 {
    let module_ident = Ident::new(module, Span::call_site());

    quote! {
        crate::encoder::#module_ident::encode(&self.#name, writer)?;
    }
}

fn render_max_length_field(name: &Ident, max_length: u16) -> TokenStream2 {
    quote! {
        crate::encoder::EncoderWriteExt::write_string(writer, &self.#name, #max_length)?;
    }
}
