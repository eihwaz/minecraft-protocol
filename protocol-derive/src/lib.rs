extern crate proc_macro;

use crate::parse::parse_derive_input;
use crate::render::decoder::render_decoder;
use crate::render::encoder::render_encoder;
use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::DeriveInput;

mod error;
mod parse;
mod render;

#[proc_macro_derive(Encoder, attributes(data_type))]
pub fn derive_encoder(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let (name, fields) = parse_derive_input(&input).expect("Failed to parse derive input");

    TokenStream::from(render_encoder(name, &fields))
}

#[proc_macro_derive(Decoder, attributes(data_type))]
pub fn derive_decoder(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as DeriveInput);
    let (name, fields) = parse_derive_input(&input).expect("Failed to parse derive input");

    TokenStream::from(render_decoder(name, &fields))
}
