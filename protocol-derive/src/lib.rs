extern crate proc_macro;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use quote::{quote, TokenStreamExt};
use std::iter::FromIterator;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields, Lit, Meta, NestedMeta, Type};

#[proc_macro_derive(Packet, attributes(packet))]
pub fn derive_packet(input: proc_macro::TokenStream) -> TokenStream1 {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    match input.data {
        Data::Struct(data) => {
            let fields = &data.fields;

            let encoder = impl_encoder_trait(name, fields);
            let decoder = impl_decoder_trait(name, fields);

            println!("{}", decoder);

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

        // This is special case because max length are used only for strings.
        if let Some(max_length) = parsed_meta.max_length {
            return quote! {
                crate::EncoderWriteExt::write_string(writer, &self.#name, #max_length)?;
            };
        }

        let module = parsed_meta.module.as_deref().unwrap_or("Encoder");
        let module_ident = Ident::new(&module, Span::call_site());

        quote! {
            crate::#module_ident::encode(&self.#name, writer)?;
        }
    });

    quote! {
        #[automatically_derived]
        impl crate::Encoder for #name {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<(), crate::EncodeError> {
                #encode

                Ok(())
            }
        }
    }
}

fn impl_decoder_trait(name: &Ident, fields: &Fields) -> TokenStream2 {
    let mut bitfields = vec![];
    let total_fields = fields.iter().count();
    let mut current = 0;

    let decode = quote_field(fields, |field| {
        let name = &field.ident;
        let ty = &field.ty;

        let unparsed_meta = get_packet_field_meta(field);
        let parsed_meta = parse_packet_field_meta(&unparsed_meta);

        let mut result = quote!();

        // When we encounter a bitfield, we skip writing until we get
        // to the last bit field or the last field.
        match &parsed_meta.bitfield {
            Some(bitfield) => {
                bitfields.push((field.clone(), bitfield.clone()));
            }
            None => {
                result.append_all(quote_decoder_bitfield(&bitfields));
                bitfields.clear();
            }
        }

        current += 1;

        if !bitfields.is_empty() {
            return if current == total_fields {
                result.append_all(quote_decoder_bitfield(&bitfields));
                bitfields.clear();
                result
            } else {
                quote!()
            };
        }

        // This is special case because max length are used only for strings.
        if let Some(max_length) = parsed_meta.max_length {
            result.append_all(quote! {
                let #name = crate::DecoderReadExt::read_string(reader, #max_length)?;
            });

            return result;
        }

        match &parsed_meta.module {
            Some(module) => {
                let module_ident = Ident::new(module, Span::call_site());

                result.append_all(quote! {
                    let #name = crate::#module_ident::decode(reader)?;
                })
            }
            None => {
                result.append_all(quote! {
                    let #name = <#ty as crate::Decoder>::decode(reader)?;
                });
            }
        }

        return result;
    });

    let create = quote_field(fields, |field| {
        let name = &field.ident;

        quote! {
             #name,
        }
    });

    quote! {
        #[automatically_derived]
        impl crate::Decoder for #name {
            type Output = Self;

            fn decode<R: std::io::Read>(reader: &mut R) -> Result<Self::Output, crate::DecodeError> {
                #decode

                Ok(#name {
                    #create
                })
            }
        }
    }
}

#[derive(Debug)]
struct PacketFieldMeta {
    module: Option<String>,
    max_length: Option<u16>,
    bitfield: Option<Bitfield>,
}

#[derive(Debug, Clone)]
struct Bitfield {
    size: usize,
}

macro_rules! decode_bitmask_value (
    ($value: expr, $name: ident) => (
        fn $name(
            result: &mut TokenStream2,
            name: &Option<Ident>,
            ty: &Type,
            bitfield_size: usize,
            current: usize,
            signed: bool,
        ) {
            let mask = (2 << (bitfield_size - 1)) - $value;
            let bool = match ty {
                Type::Path(type_path) => type_path
                    .path
                    .get_ident()
                    .expect("Failed to get ident")
                    .to_string() == "bool",
                _ => false,
            };

            if bool {
              if current > 0 {
                result.append_all(quote! {
                  let mut #name = ((encoded >> #current) & #mask) != 0;
                });
              } else {
                result.append_all(quote! {
                  let mut #name = (encoded & #mask) != 0;
                });
              }
            } else {
              if current > 0 {
                result.append_all(quote! {
                  let mut #name = ((encoded >> #current) & #mask) as #ty;
                });
              } else {
                result.append_all(quote! {
                  let mut #name = (encoded & #mask) as #ty;
                });
              }
            }

            if signed {
              let subtract = mask + 1;
              let overflow = subtract >> 1;

              result.append_all(quote! {
                  if #name > #overflow as #ty {
                    #name -= #subtract as #ty;
                  }
              });
            }
        }
    )
);

decode_bitmask_value!(1i64, decode_i64_bitmask_value);
decode_bitmask_value!(1i32, decode_i32_bitmask_value);
decode_bitmask_value!(1i16, decode_i16_bitmask_value);
decode_bitmask_value!(1i8, decode_i8_bitmask_value);

fn quote_decoder_bitfield(bitfields: &Vec<(Field, Bitfield)>) -> TokenStream2 {
    if bitfields.is_empty() {
        return quote!();
    }

    let total_size: usize = bitfields.iter().map(|(_, b)| b.size).sum();

    let mut result = read_decoder_bitfield_quote(total_size);
    let mut current = total_size;

    for (field, bitfield) in bitfields {
        current -= bitfield.size;

        let name = &field.ident;
        let ty = &field.ty;

        let signed = match ty {
            Type::Path(type_path) => type_path
                .path
                .get_ident()
                .expect("Failed to get ident")
                .to_string()
                .starts_with("i"),
            _ => panic!("Unexpected bitfield type"),
        };

        match total_size {
            _ if total_size == 64 => {
                decode_i64_bitmask_value(&mut result, name, ty, bitfield.size, current, signed)
            }
            _ if total_size == 32 => {
                decode_i32_bitmask_value(&mut result, name, ty, bitfield.size, current, signed)
            }
            _ if total_size == 16 => {
                decode_i16_bitmask_value(&mut result, name, ty, bitfield.size, current, signed)
            }
            _ => decode_i8_bitmask_value(&mut result, name, ty, bitfield.size, current, signed),
        }
    }

    return result;
}

fn read_decoder_bitfield_quote(total_size: usize) -> TokenStream2 {
    match total_size {
        _ if total_size == 64 => {
            quote! {
                let encoded = ::byteorder::ReadBytesExt::read_i64::<::byteorder::BigEndian>(reader)?;
            }
        }
        _ if total_size == 32 => {
            quote! {
                let encoded = ::byteorder::ReadBytesExt::read_i32::<::byteorder::BigEndian>(reader)?;
            }
        }
        _ if total_size == 16 => {
            quote! {
                let encoded = ::byteorder::ReadBytesExt::read_i16::<::byteorder::BigEndian>(reader)?;
            }
        }
        _ if total_size == 8 => {
            quote! {
                let encoded = ::byteorder::ReadBytesExt::read_i8(reader)?;
            }
        }
        _ => panic!("Bitfield size must be aligned to 8, 16, 32 or 64"),
    }
}

fn parse_packet_field_meta(meta_list: &Vec<NestedMeta>) -> PacketFieldMeta {
    let mut module = None;
    let mut max_length = None;
    let mut bitfield = None;

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
                    path.get_ident().expect("Failed to get ident")
                ),
            },
            NestedMeta::Meta(Meta::List(meta_list)) if meta_list.path.is_ident("bitfield") => {
                let mut size = None;

                for meta in &meta_list.nested {
                    match meta {
                        NestedMeta::Meta(Meta::NameValue(named_meta)) => match &named_meta.path {
                            path if path.is_ident("size") => match &named_meta.lit {
                                Lit::Int(lit_int) => {
                                    size = Some(
                                        lit_int
                                            .base10_parse::<usize>()
                                            .expect("Failed to parse size attribute"),
                                    )
                                }
                                _ => panic!("\"size\" attribute value must be integer"),
                            },
                            path => panic!(
                                "Received unrecognized attribute : \"{}\"",
                                path.get_ident().expect("Failed to get ident")
                            ),
                        },
                        _ => panic!("Unexpected field meta"),
                    }
                }

                let size = size.unwrap_or_else(|| panic!("Size must be specified in bitfield"));
                bitfield = Some(Bitfield { size })
            }
            _ => panic!("Unexpected field meta"),
        }
    }

    PacketFieldMeta {
        module,
        max_length,
        bitfield,
    }
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

fn quote_field<F: FnMut(&Field) -> TokenStream2>(fields: &Fields, mut func: F) -> TokenStream2 {
    let mut output = quote!();

    match fields {
        Fields::Named(named_fields) => {
            output.append_all(named_fields.named.iter().map(|f| func(f)))
        }
        _ => panic!("Expected only for named fields"),
    }

    output
}
