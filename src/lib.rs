//! This crate implements Minecraft protocol.
//!
//! Information about protocol can be found at https://wiki.vg/Protocol.
use io::Error as IoError;
use mc_varint::{VarIntRead, VarIntWrite};
use serde_json::error::Error as JsonError;
use std::io;
use std::io::{Read, Write};
use std::string::FromUtf8Error;

pub mod chat;
pub mod status;

/// Current supported protocol version.
pub const PROTOCOL_VERSION: usize = 498;
/// String maximum length.
const MAX_STRING_LENGTH: usize = 32_768;

/// Possible errors while encoding packet.
pub enum EncodeError {
    /// String length can't be more than `MAX_STRING_LENGTH` value.
    StringTooLong,
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
pub enum DecodeError {
    /// Packet was not recognized. Invalid data or wrong protocol version.
    UnknownPacketType {
        type_id: u8,
    },
    /// String length can't be more than `MAX_STRING_LENGTH` value.
    StringTooLong,
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

trait Packet {
    type Output;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError>;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError>;
}

/// Trait adds additional helper methods for `Read` to read protocol data.
trait PacketRead {
    fn read_string(&mut self) -> Result<String, DecodeError>;
}

/// Trait adds additional helper methods for `Write` to write protocol data.
trait PacketWrite {
    fn write_string(&mut self, value: &str) -> Result<(), EncodeError>;
}

impl<R: Read> PacketRead for R {
    fn read_string(&mut self) -> Result<String, DecodeError> {
        let length = self.read_var_u32()?;

        if length > MAX_STRING_LENGTH as u32 {
            return Err(DecodeError::StringTooLong);
        }

        let mut buf = vec![0; length as usize];
        self.read_exact(&mut buf)?;

        Ok(String::from_utf8(buf)?)
    }
}

impl<W: Write> PacketWrite for W {
    fn write_string(&mut self, value: &str) -> Result<(), EncodeError> {
        if value.len() > MAX_STRING_LENGTH {
            return Err(EncodeError::StringTooLong);
        }

        self.write_var_u32(value.len() as u32)?;
        self.write_all(value.as_bytes())?;

        Ok(())
    }
}
