//! This crate implements Minecraft protocol.
//!
//! Information about protocol can be found at https://wiki.vg/Protocol.
pub mod data;
pub mod decoder;
pub mod encoder;
pub mod error;
pub mod version;

/// Protocol limits maximum string length.
const STRING_MAX_LENGTH: u16 = 32_768;

#[macro_export]
macro_rules! impl_enum_encoder_decoder (
    ($ty: ident) => (
        impl crate::encoder::Encoder for $ty {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<(), crate::error::EncodeError> {
                Ok(crate::encoder::EncoderWriteExt::write_enum(writer, self)?)
            }
        }

        impl crate::decoder::Decoder for $ty {
            type Output = Self;

            fn decode<R: std::io::Read>(reader: &mut R) -> Result<Self::Output, crate::error::DecodeError> {
                Ok(crate::decoder::DecoderReadExt::read_enum(reader)?)
            }
        }
   );
);

#[macro_export]
macro_rules! impl_json_encoder_decoder (
    ($ty: ident) => (
        impl crate::encoder::Encoder for $ty {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<(), crate::error::EncodeError> {
                let json = serde_json::to_string(self)?;
                crate::encoder::EncoderWriteExt::write_string(writer, &json, crate::STRING_MAX_LENGTH)?;

                Ok(())
            }
        }

        impl crate::decoder::Decoder for $ty {
            type Output = Self;

            fn decode<R: std::io::Read>(reader: &mut R) -> Result<Self::Output, crate::error::DecodeError> {
                let json = crate::decoder::DecoderReadExt::read_string(reader, crate::STRING_MAX_LENGTH)?;

                Ok(serde_json::from_str(&json)?)
            }
        }
   );
);
