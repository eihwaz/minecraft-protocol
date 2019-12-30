//! This crate implements Minecraft protocol.
//!
//! Information about protocol can be found at https://wiki.vg/Protocol.
use io::Error as IoError;
use std::io;
use std::io::{Read, Write};
use std::string::FromUtf8Error;

use byteorder::ReadBytesExt;
use byteorder::WriteBytesExt;
use mc_varint::{VarIntRead, VarIntWrite};
use serde_json::error::Error as JsonError;
use uuid::parser::ParseError as UuidParseError;

use crate::chat::Message;
use nbt::decode::TagDecodeError;
use nbt::CompoundTag;
use num_traits::{FromPrimitive, ToPrimitive};

pub mod chat;
pub mod game;
pub mod login;
pub mod status;

/// Current supported protocol version.
pub const PROTOCOL_VERSION: u32 = 575;
/// String maximum length.
const STRING_MAX_LENGTH: u32 = 32_768;

/// Possible errors while encoding packet.
#[derive(Debug)]
pub enum EncodeError {
    /// String length can't be more than provided value.
    StringTooLong {
        /// String length.
        length: usize,
        /// Max string length.
        max_length: u32,
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
        length: u32,
        /// Max string length.
        max_length: u32,
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

trait PacketParser {
    type Output;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError>;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError>;
}

/// Trait adds additional helper methods for `Read` to read protocol data.
trait PacketRead {
    fn read_bool(&mut self) -> Result<bool, DecodeError>;

    fn read_string(&mut self, max_length: u32) -> Result<String, DecodeError>;

    fn read_byte_array(&mut self) -> Result<Vec<u8>, DecodeError>;

    fn read_chat_message(&mut self) -> Result<Message, DecodeError>;

    fn read_enum<T: FromPrimitive>(&mut self) -> Result<T, DecodeError>;

    fn read_compound_tag(&mut self) -> Result<CompoundTag, DecodeError>;
}

/// Trait adds additional helper methods for `Write` to write protocol data.
trait PacketWrite {
    fn write_bool(&mut self, value: bool) -> Result<(), EncodeError>;

    fn write_string(&mut self, value: &str, max_length: u32) -> Result<(), EncodeError>;

    fn write_byte_array(&mut self, value: &[u8]) -> Result<(), EncodeError>;

    fn write_chat_message(&mut self, value: &Message) -> Result<(), EncodeError>;

    fn write_enum<T: ToPrimitive>(&mut self, value: &T) -> Result<(), EncodeError>;

    fn write_compound_tag(&mut self, value: &CompoundTag) -> Result<(), EncodeError>;
}

impl<R: Read> PacketRead for R {
    fn read_bool(&mut self) -> Result<bool, DecodeError> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::NonBoolValue),
        }
    }

    fn read_string(&mut self, max_length: u32) -> Result<String, DecodeError> {
        let length = self.read_var_i32()? as u32;

        if length > max_length as u32 {
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
}

impl<W: Write> PacketWrite for W {
    fn write_bool(&mut self, value: bool) -> Result<(), EncodeError> {
        if value {
            self.write_u8(1)?;
        } else {
            self.write_u8(0)?;
        }

        Ok(())
    }

    fn write_string(&mut self, value: &str, max_length: u32) -> Result<(), EncodeError> {
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
}
