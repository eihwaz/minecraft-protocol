//! This crate implements Minecraft protocol.
//!
//! Information about protocol can be found at https://wiki.vg/Protocol.
use io::Error as IoError;
use std::io;
use std::io::{Cursor, Read, Write};
use std::string::FromUtf8Error;

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde_json::error::Error as JsonError;
use uuid::parser::ParseError as UuidParseError;

use crate::chat::Message;
use nbt::decode::TagDecodeError;
use nbt::CompoundTag;
use num_traits::{FromPrimitive, ToPrimitive};
use uuid::Uuid;

pub mod chat;
pub mod game;
pub mod login;
pub mod status;

/// Current supported protocol version.
pub const PROTOCOL_VERSION: u32 = 498;
/// Protocol limits maximum string length.
const STRING_MAX_LENGTH: u16 = 32_768;
const HYPHENATED_UUID_LENGTH: u16 = 36;

/// Possible errors while encoding packet.
#[derive(Debug)]
pub enum EncodeError {
    /// String length can't be more than provided value.
    StringTooLong {
        /// String length.
        length: usize,
        /// Max string length.
        max_length: u16,
    },
    IOError {
        io_error: IoError,
    },
    JsonError {
        json_error: JsonError,
    },
}

impl From<IoError> for EncodeError {
    fn from(io_error: IoError) -> Self {
        EncodeError::IOError { io_error }
    }
}

impl From<JsonError> for EncodeError {
    fn from(json_error: JsonError) -> Self {
        EncodeError::JsonError { json_error }
    }
}

/// Possible errors while decoding packet.
#[derive(Debug)]
pub enum DecodeError {
    /// Packet was not recognized. Invalid data or wrong protocol version.
    UnknownPacketType {
        type_id: u8,
    },
    /// String length can't be more than provided value.
    StringTooLong {
        /// String length.
        length: usize,
        /// Max string length.
        max_length: u16,
    },
    IOError {
        io_error: IoError,
    },
    JsonError {
        json_error: JsonError,
    },
    /// Byte array was not recognized as valid UTF-8 string.
    Utf8Error {
        utf8_error: FromUtf8Error,
    },
    /// Boolean are parsed from byte. Valid byte value are 0 or 1.
    NonBoolValue,
    UuidParseError {
        uuid_parse_error: UuidParseError,
    },
    // Type id was not parsed as valid enum value.
    UnknownEnumType {
        type_id: u8,
    },
    TagDecodeError {
        tag_decode_error: TagDecodeError,
    },
    VarIntTooLong {
        max_bytes: usize,
    },
}

impl From<IoError> for DecodeError {
    fn from(io_error: IoError) -> Self {
        DecodeError::IOError { io_error }
    }
}

impl From<JsonError> for DecodeError {
    fn from(json_error: JsonError) -> Self {
        DecodeError::JsonError { json_error }
    }
}

impl From<FromUtf8Error> for DecodeError {
    fn from(utf8_error: FromUtf8Error) -> Self {
        DecodeError::Utf8Error { utf8_error }
    }
}

impl From<UuidParseError> for DecodeError {
    fn from(uuid_parse_error: UuidParseError) -> Self {
        DecodeError::UuidParseError { uuid_parse_error }
    }
}

impl From<TagDecodeError> for DecodeError {
    fn from(tag_decode_error: TagDecodeError) -> Self {
        DecodeError::TagDecodeError { tag_decode_error }
    }
}

trait Encoder {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError>;
}

trait Decoder {
    type Output;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError>;
}

macro_rules! write_signed_var_int (
    ($type: ident, $name: ident) => (
        fn $name(&mut self, mut value: $type) -> Result<(), EncodeError> {
            loop {
                let mut byte = (value & 0b01111111) as u8;
                value = value >> 7;

                if value != 0 {
                    byte |= 0b10000000;
                }

                self.write_u8(byte)?;

                if value == 0 {
                   break;
                }
            }

            Ok(())
        }
    )
);

macro_rules! read_signed_var_int (
    ($type: ident, $name: ident, $max_bytes: expr) => (
        fn $name(&mut self) -> Result<$type, DecodeError> {
            let mut bytes = 0;
            let mut output = 0;

            loop {
                let byte = self.read_u8()?;
                let value = (byte & 0b01111111) as $type;

                output |= value << 7 * bytes;
                bytes += 1;

                if bytes > $max_bytes {
                    return Err(DecodeError::VarIntTooLong { max_bytes: $max_bytes })
                }

                if (byte & 0b10000000) == 0 {
                    break;
                }
            }

            Ok(output)
        }
   );
);

/// Trait adds additional helper methods for `Write` to write protocol data.
trait EncoderWriteExt {
    fn write_bool(&mut self, value: bool) -> Result<(), EncodeError>;

    fn write_string(&mut self, value: &str, max_length: u16) -> Result<(), EncodeError>;

    fn write_byte_array(&mut self, value: &[u8]) -> Result<(), EncodeError>;

    fn write_chat_message(&mut self, value: &Message) -> Result<(), EncodeError>;

    fn write_enum<T: ToPrimitive>(&mut self, value: &T) -> Result<(), EncodeError>;

    fn write_compound_tag(&mut self, value: &CompoundTag) -> Result<(), EncodeError>;

    fn write_var_i32(&mut self, value: i32) -> Result<(), EncodeError>;

    fn write_var_i64(&mut self, value: i64) -> Result<(), EncodeError>;
}

/// Trait adds additional helper methods for `Read` to read protocol data.
trait DecoderReadExt {
    fn read_bool(&mut self) -> Result<bool, DecodeError>;

    fn read_string(&mut self, max_length: u16) -> Result<String, DecodeError>;

    fn read_byte_array(&mut self) -> Result<Vec<u8>, DecodeError>;

    fn read_chat_message(&mut self) -> Result<Message, DecodeError>;

    fn read_enum<T: FromPrimitive>(&mut self) -> Result<T, DecodeError>;

    fn read_compound_tag(&mut self) -> Result<CompoundTag, DecodeError>;

    fn read_var_i32(&mut self) -> Result<i32, DecodeError>;

    fn read_var_i64(&mut self) -> Result<i64, DecodeError>;
}

impl<W: Write> EncoderWriteExt for W {
    fn write_bool(&mut self, value: bool) -> Result<(), EncodeError> {
        if value {
            self.write_u8(1)?;
        } else {
            self.write_u8(0)?;
        }

        Ok(())
    }

    fn write_string(&mut self, value: &str, max_length: u16) -> Result<(), EncodeError> {
        let length = value.len();

        if length > max_length as usize {
            return Err(EncodeError::StringTooLong { length, max_length });
        }

        self.write_var_i32(value.len() as i32)?;
        self.write_all(value.as_bytes())?;

        Ok(())
    }

    fn write_byte_array(&mut self, value: &[u8]) -> Result<(), EncodeError> {
        self.write_var_i32(value.len() as i32)?;
        self.write_all(value)?;

        Ok(())
    }

    fn write_chat_message(&mut self, value: &Message) -> Result<(), EncodeError> {
        self.write_string(&value.to_json()?, STRING_MAX_LENGTH)
    }

    fn write_enum<T: ToPrimitive>(&mut self, value: &T) -> Result<(), EncodeError> {
        let type_value = ToPrimitive::to_u8(value).unwrap();
        self.write_u8(type_value)?;

        Ok(())
    }

    fn write_compound_tag(&mut self, value: &CompoundTag) -> Result<(), EncodeError> {
        nbt::encode::write_compound_tag(self, value.clone())?;

        Ok(())
    }

    write_signed_var_int!(i32, write_var_i32);
    write_signed_var_int!(i64, write_var_i64);
}

impl<R: Read> DecoderReadExt for R {
    fn read_bool(&mut self) -> Result<bool, DecodeError> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::NonBoolValue),
        }
    }

    fn read_string(&mut self, max_length: u16) -> Result<String, DecodeError> {
        let length = self.read_var_i32()? as usize;

        if length as u16 > max_length {
            return Err(DecodeError::StringTooLong { length, max_length });
        }

        let mut buf = vec![0; length as usize];
        self.read_exact(&mut buf)?;

        Ok(String::from_utf8(buf)?)
    }

    fn read_byte_array(&mut self) -> Result<Vec<u8>, DecodeError> {
        let length = self.read_var_i32()?;

        let mut buf = vec![0; length as usize];
        self.read_exact(&mut buf)?;

        Ok(buf)
    }

    fn read_chat_message(&mut self) -> Result<Message, DecodeError> {
        let json = self.read_string(STRING_MAX_LENGTH)?;
        let message = Message::from_json(&json)?;

        Ok(message)
    }

    fn read_enum<T: FromPrimitive>(&mut self) -> Result<T, DecodeError> {
        let type_id = self.read_u8()?;
        let result = FromPrimitive::from_u8(type_id);

        result.ok_or_else(|| DecodeError::UnknownEnumType { type_id })
    }

    fn read_compound_tag(&mut self) -> Result<CompoundTag, DecodeError> {
        Ok(nbt::decode::read_compound_tag(self)?)
    }

    read_signed_var_int!(i32, read_var_i32, 5);
    read_signed_var_int!(i64, read_var_i64, 10);
}

impl Encoder for u8 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_u8(*self)?)
    }
}

impl Decoder for u8 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_u8()?)
    }
}

impl Encoder for i32 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_i32::<BigEndian>(*self)?)
    }
}

impl Decoder for i32 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_i32::<BigEndian>()?)
    }
}

impl Encoder for u32 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_u32::<BigEndian>(*self)?)
    }
}

impl Decoder for u32 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_u32::<BigEndian>()?)
    }
}

impl Encoder for i64 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_i64::<BigEndian>(*self)?)
    }
}

impl Decoder for i64 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_i64::<BigEndian>()?)
    }
}

impl Encoder for u64 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_u64::<BigEndian>(*self)?)
    }
}

impl Decoder for u64 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_u64::<BigEndian>()?)
    }
}

impl Encoder for String {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_string(self, STRING_MAX_LENGTH)?)
    }
}

impl Decoder for String {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_string(STRING_MAX_LENGTH)?)
    }
}

impl Encoder for bool {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_bool(*self)?)
    }
}

impl Decoder for bool {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_bool()?)
    }
}

impl Encoder for Vec<u8> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_byte_array(self)?)
    }
}

impl Decoder for Vec<u8> {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_byte_array()?)
    }
}

impl Encoder for Uuid {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_all(self.as_bytes())?)
    }
}

impl Decoder for Uuid {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let mut buf = [0; 16];
        reader.read_exact(&mut buf)?;

        Ok(Uuid::from_bytes(buf))
    }
}

impl Encoder for CompoundTag {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_compound_tag(self)?)
    }
}

impl Decoder for CompoundTag {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_compound_tag()?)
    }
}

impl Encoder for Vec<CompoundTag> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_var_i32(self.len() as i32)?;

        for compound_tag in self {
            writer.write_compound_tag(&compound_tag)?;
        }

        Ok(())
    }
}

impl Decoder for Vec<CompoundTag> {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let length = reader.read_var_i32()? as usize;
        let mut vec = Vec::with_capacity(length);

        for _ in 0..length {
            let compound_tag = reader.read_compound_tag()?;
            vec.push(compound_tag);
        }

        Ok(vec)
    }
}

#[macro_export]
macro_rules! impl_enum_encoder_decoder (
    ($ty: ident) => (
        impl crate::Encoder for $ty {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<(), crate::EncodeError> {
                Ok(crate::EncoderWriteExt::write_enum(writer, self)?)
            }
        }

        impl crate::Decoder for $ty {
            type Output = Self;

            fn decode<R: std::io::Read>(reader: &mut R) -> Result<Self::Output, crate::DecodeError> {
                Ok(crate::DecoderReadExt::read_enum(reader)?)
            }
        }
   );
);

#[macro_export]
macro_rules! impl_json_encoder_decoder (
    ($ty: ident) => (
        impl crate::Encoder for $ty {
            fn encode<W: std::io::Write>(&self, writer: &mut W) -> Result<(), crate::EncodeError> {
                let json = serde_json::to_string(self)?;
                crate::EncoderWriteExt::write_string(writer, &json, crate::STRING_MAX_LENGTH)?;

                Ok(())
            }
        }

        impl crate::Decoder for $ty {
            type Output = Self;

            fn decode<R: std::io::Read>(reader: &mut R) -> Result<Self::Output, crate::DecodeError> {
                let json = crate::DecoderReadExt::read_string(reader, crate::STRING_MAX_LENGTH)?;

                Ok(serde_json::from_str(&json)?)
            }
        }
   );
);

mod var_int {
    use crate::{DecodeError, EncodeError};
    use crate::{DecoderReadExt, EncoderWriteExt};
    use std::io::{Read, Write};

    pub fn encode<W: Write>(value: &i32, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_var_i32(*value)?;

        Ok(())
    }

    pub fn decode<R: Read>(reader: &mut R) -> Result<i32, DecodeError> {
        Ok(reader.read_var_i32()?)
    }
}

mod var_long {
    use crate::{DecodeError, EncodeError};
    use crate::{DecoderReadExt, EncoderWriteExt};
    use std::io::{Read, Write};

    pub fn encode<W: Write>(value: &i64, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_var_i64(*value)?;

        Ok(())
    }

    pub fn decode<R: Read>(reader: &mut R) -> Result<i64, DecodeError> {
        Ok(reader.read_var_i64()?)
    }
}

mod rest {
    use crate::{DecodeError, EncodeError};
    use std::io::{Read, Write};

    pub fn encode<W: Write>(value: &[u8], writer: &mut W) -> Result<(), EncodeError> {
        writer.write_all(value)?;

        Ok(())
    }

    pub fn decode<R: Read>(reader: &mut R) -> Result<Vec<u8>, DecodeError> {
        let mut data = Vec::new();
        reader.read_to_end(data.as_mut())?;

        Ok(data)
    }
}

mod uuid_hyp_str {
    use crate::{
        DecodeError, DecoderReadExt, EncodeError, EncoderWriteExt, HYPHENATED_UUID_LENGTH,
    };
    use std::io::{Read, Write};
    use uuid::Uuid;

    pub fn encode<W: Write>(value: &Uuid, writer: &mut W) -> Result<(), EncodeError> {
        let uuid_hyphenated_string = value.to_hyphenated().to_string();
        writer.write_string(&uuid_hyphenated_string, HYPHENATED_UUID_LENGTH)?;

        Ok(())
    }

    pub fn decode<R: Read>(reader: &mut R) -> Result<Uuid, DecodeError> {
        let uuid_hyphenated_string = reader.read_string(HYPHENATED_UUID_LENGTH)?;
        let uuid = Uuid::parse_str(&uuid_hyphenated_string)?;

        Ok(uuid)
    }
}

#[test]
fn test_read_variable_i32_2_bytes_value() {
    let mut cursor = Cursor::new(vec![0b10101100, 0b00000010]);
    let value = cursor.read_var_i32().unwrap();

    assert_eq!(value, 300);
}

#[test]
fn test_read_variable_i32_5_bytes_value() {
    let mut cursor = Cursor::new(vec![0xff, 0xff, 0xff, 0xff, 0x07]);
    let value = cursor.read_var_i32().unwrap();

    assert_eq!(value, 2147483647);
}

#[test]
fn test_write_variable_i32_2_bytes_value() {
    let mut cursor = Cursor::new(Vec::with_capacity(5));
    cursor.write_var_i32(300).unwrap();

    assert_eq!(cursor.into_inner(), vec![0b10101100, 0b00000010]);
}

#[test]
fn test_write_variable_i32_5_bytes_value() {
    let mut cursor = Cursor::new(Vec::with_capacity(5));
    cursor.write_var_i32(2147483647).unwrap();

    assert_eq!(cursor.into_inner(), vec![0xff, 0xff, 0xff, 0xff, 0x07]);
}
