use nbt::decode::TagDecodeError;
use serde_json::error::Error as JsonError;
use std::io::Error as IoError;
use std::string::FromUtf8Error;
use uuid::parser::ParseError as UuidParseError;

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
    /// Type id was not parsed as valid enum value.
    UnknownEnumType {
        type_id: usize,
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
