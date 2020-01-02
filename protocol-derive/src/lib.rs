extern crate proc_macro;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, TokenStreamExt};
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields};

#[proc_macro_derive(Packet)]
pub fn derive_packet(input: proc_macro::TokenStream) -> TokenStream1 {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    match input.data {
        Data::Struct(data) => {
            let fields = &data.fields;

            let encoder = impl_encoder_trait(name, fields);
            let decoder = impl_decoder_trait(name, fields);

            TokenStream1::from(quote! {
                #encoder

                #decoder
            })
        }
        _ => panic!("Packet derive are available only for structures"),
    }
}

fn impl_encoder_trait(name: &Ident, fields: &Fields) -> TokenStream2 {
    let encode = quote_field(fields, |field| {
        let name = &field.ident;

        quote! {
           Encoder::encode(&self.#name, writer);
        }
    });

    quote! {
        impl crate::Encoder for #name {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<(), crate::EncodeError> {
                #encode

                Ok(())
            }
        }
    }
}

fn impl_decoder_trait(name: &Ident, fields: &Fields) -> TokenStream2 {
    let decode = quote_field(fields, |field| {
        quote! {
           todo!();
        }
    });

    quote! {
        impl crate::Decoder for #name {
            type Output = Self;

            fn decode<R: std::io::Read>(reader: &mut R) -> Result<Self::Output, crate::DecodeError> {
                #decode
            }
        }
    }
}

fn quote_field<F: Fn(&Field) -> TokenStream2>(fields: &Fields, func: F) -> TokenStream2 {
    let mut output = quote!();

    match fields {
        Fields::Named(named_fields) => {
            output.append_all(named_fields.named.iter().map(|f| func(f)))
        }
        _ => panic!("Packet derive are available only for named fields"),
    }

    output
}
