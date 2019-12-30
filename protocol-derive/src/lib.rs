extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(MinecraftPacket)]
pub fn derive_packet(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let output = quote! {
        impl minecraft_protocol::Packet for #name {
            type Output = Self;

            fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {

            }

            fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {

            }
        }
    };

    TokenStream::from(output)
}
