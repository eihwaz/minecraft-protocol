extern crate proc_macro;

use crate::parse::parse_derive_input;
use crate::render::decoder::render_decoder_trait;
use crate::render::encoder::render_encoder_trait;
use proc_macro::TokenStream as TokenStream1;
use quote::quote;
use syn::parse_macro_input;
use syn::DeriveInput;

mod error;
mod parse;
mod render;

#[proc_macro_derive(Packet, attributes(packet))]
pub fn derive_packet(tokens: TokenStream1) -> TokenStream1 {
    let input = parse_macro_input!(tokens as DeriveInput);
    let (name, fields) = parse_derive_input(&input).expect("Failed to parse derive input");

    let encoder = render_encoder_trait(name, &fields);
    let decoder = render_decoder_trait(name, &fields);

    TokenStream1::from(quote! {
        #encoder

        #decoder
    })
}
