extern crate proc_macro;

use crate::parse::{parse_derive_input, DeriveInputParseResult};
use crate::render::decoder::render_struct_decoder;
use crate::render::encoder::render_struct_encoder;
use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::DeriveInput;

mod error;
mod parse;
mod render;

#[proc_macro_derive(Encoder, attributes(data_type))]
pub fn derive_encoder(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let derive_parse_result = parse_derive_input(&input).expect("Failed to parse derive input");

    match derive_parse_result {
        DeriveInputParseResult::Struct { name, fields } => {
            TokenStream::from(render_struct_encoder(name, &fields))
        }
        DeriveInputParseResult::Enum { name, variants } => TokenStream::new(),
    }
}

#[proc_macro_derive(Decoder, attributes(data_type))]
pub fn derive_decoder(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let derive_parse_result = parse_derive_input(&input).expect("Failed to parse derive input");

    match derive_parse_result {
        DeriveInputParseResult::Struct { name, fields } => {
            TokenStream::from(render_struct_decoder(name, &fields))
        }
        DeriveInputParseResult::Enum { name, variants } => TokenStream::new(),
    }
}
