use std::io;
use std::io::{Read, Write};

pub mod status;

/// Current supported protocol version.
pub const PROTOCOL_VERSION: usize = 498;

/// Possible errors while decoding packet.
pub enum DecodePacketError {
    UnknownPacketType { type_id: u8 },
    IOError { io_error: io::Error },
}

impl From<io::Error> for DecodePacketError {
    fn from(io_error: io::Error) -> Self {
        DecodePacketError::IOError { io_error }
    }
}

trait Packet {
    type Output;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), io::Error>;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodePacketError>;
}
