use mc_varint::{VarIntRead, VarIntWrite};
use std::io;
use std::io::{Read, Write};
use std::string::FromUtf8Error;

pub mod status;

/// Current supported protocol version.
pub const PROTOCOL_VERSION: usize = 498;

/// Possible errors while encoding packet.
pub enum EncodeError {
    StringTooLong,
    IOError {
        io_error: io::Error,
    },
    JsonError {
        json_error: serde_json::error::Error,
    },
}

impl From<io::Error> for EncodeError {
    fn from(io_error: io::Error) -> Self {
        EncodeError::IOError { io_error }
    }
}

impl From<serde_json::error::Error> for EncodeError {
    fn from(json_error: serde_json::error::Error) -> Self {
        EncodeError::JsonError { json_error }
    }
}

/// Possible errors while decoding packet.
pub enum DecodeError {
    UnknownPacketType {
        type_id: u8,
    },
    StringTooLong,
    IOError {
        io_error: io::Error,
    },
    JsonError {
        json_error: serde_json::error::Error,
    },
    Utf8Error {
        utf8_error: FromUtf8Error,
    },
}

impl From<io::Error> for DecodeError {
    fn from(io_error: io::Error) -> Self {
        DecodeError::IOError { io_error }
    }
}

impl From<serde_json::error::Error> for DecodeError {
    fn from(json_error: serde_json::error::Error) -> Self {
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

trait PacketRead {
    fn read_string(&mut self) -> Result<String, DecodeError>;
}

trait PacketWrite {
    fn write_string(&mut self, value: &str) -> Result<(), EncodeError>;
}

impl<R: Read> PacketRead for R {
    fn read_string(&mut self) -> Result<String, DecodeError> {
        let length = self.read_var_u32()?;

        if length > 32_767 {
            return Err(DecodeError::StringTooLong);
        }

        let mut buf = vec![0; length as usize];
        self.read_exact(&mut buf)?;

        Ok(String::from_utf8(buf)?)
    }
}

impl<W: Write> PacketWrite for W {
    fn write_string(&mut self, value: &str) -> Result<(), EncodeError> {
        if value.len() > 32_767 {
            return Err(EncodeError::StringTooLong);
        }

        self.write_var_u32(value.len() as u32)?;
        self.write_all(value.as_bytes())?;

        Ok(())
    }
}
