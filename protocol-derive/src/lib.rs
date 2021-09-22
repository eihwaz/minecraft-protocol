extern crate proc_macro;

use crate::parse::{parse_derive_input, DeriveInputParseResult};
use crate::render::decoder::{render_enum_decoder, render_struct_decoder};
use crate::render::encoder::{render_enum_encoder, render_struct_encoder};
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

    TokenStream::from(match derive_parse_result {
        DeriveInputParseResult::Struct { name, fields } => render_struct_encoder(name, &fields),
        DeriveInputParseResult::Enum {
            name,
            discriminant_type,
            variants,
        } => render_enum_encoder(name, &discriminant_type, &variants),
    })
}

#[proc_macro_derive(Decoder, attributes(data_type))]
pub fn derive_decoder(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let derive_parse_result = parse_derive_input(&input).expect("Failed to parse derive input");

    TokenStream::from(match derive_parse_result {
        DeriveInputParseResult::Struct { name, fields } => render_struct_decoder(name, &fields),
        DeriveInputParseResult::Enum {
            name,
            discriminant_type,
            variants,
        } => render_enum_decoder(name, &discriminant_type, &variants),
    })
}
