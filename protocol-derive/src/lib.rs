extern crate proc_macro;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::Ident;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, TokenStreamExt};
use std::iter::FromIterator;
use syn::export::Span;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Lit, Meta, NestedMeta};

#[proc_macro_derive(Packet, attributes(packet))]
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
        _ => panic!("Expected only structures"),
    }
}

fn impl_encoder_trait(name: &Ident, fields: &Fields) -> TokenStream2 {
    let encode = quote_field(fields, |field| {
        let name = &field.ident;

        let unparsed_meta = get_packet_field_meta(field);
        let parsed_meta = parse_packet_field_meta(&unparsed_meta);

        match parsed_meta.module {
            Some(module) => {
                let module_ident = Ident::new(&module, Span::call_site());

                quote! {
                    crate::#module_ident::encode(&self.#name, writer)?;
                }
            }
            None => {
                quote! {
                    crate::Encoder::encode(&self.#name, writer)?;
                }
            }
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
    let decode = quote_field(fields, |_field| {
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

#[derive(Debug)]
struct PacketFieldMeta {
    module: Option<String>,
    max_length: Option<u16>,
}

fn parse_packet_field_meta(meta_list: &Vec<NestedMeta>) -> PacketFieldMeta {
    let mut module = None;
    let mut max_length = None;

    for meta in meta_list {
        match meta {
            NestedMeta::Meta(Meta::NameValue(named_meta)) => match &named_meta.path {
                path if path.is_ident("with") => match &named_meta.lit {
                    Lit::Str(lit_str) => module = Some(lit_str.value()),
                    _ => panic!("\"with\" attribute value must be string"),
                },
                path if path.is_ident("max_length") => match &named_meta.lit {
                    Lit::Int(lit_int) => {
                        max_length = Some(
                            lit_int
                                .base10_parse::<u16>()
                                .expect("Failed to parse max length attribute"),
                        )
                    }
                    _ => panic!("\"max_length\" attribute value must be integer"),
                },
                path => panic!(
                    "Received unrecognized attribute : \"{}\"",
                    path.get_ident().unwrap()
                ),
            },
            _ => panic!("Expected only named meta values"),
        }
    }

    PacketFieldMeta { module, max_length }
}

fn get_packet_field_meta(field: &Field) -> Vec<NestedMeta> {
    field
        .attrs
        .iter()
        .filter(|a| a.path.is_ident("packet"))
        .map(|a| a.parse_meta().expect("Failed to parse field attribute"))
        .map(|m| match m {
            Meta::List(meta_list) => Vec::from_iter(meta_list.nested),
            _ => panic!("Expected only list attributes"),
        })
        .flatten()
        .collect()
}

fn quote_field<F: Fn(&Field) -> TokenStream2>(fields: &Fields, func: F) -> TokenStream2 {
    let mut output = quote!();

    match fields {
        Fields::Named(named_fields) => {
            output.append_all(named_fields.named.iter().map(|f| func(f)))
        }
        _ => panic!("Expected only for named fields"),
    }

    output
}
