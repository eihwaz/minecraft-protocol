use crate::decoder::Decoder;
use crate::error::DecodeError;
use minecraft_protocol_derive::{Decoder, Encoder};
use std::io::Read;

pub enum HandshakeServerBoundPacket {
    Handshake(Handshake),
}

impl HandshakeServerBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            HandshakeServerBoundPacket::Handshake(_) => 0x00,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x00 => {
                let handshake = Handshake::decode(reader)?;
                Ok(HandshakeServerBoundPacket::Handshake(handshake))
            }
            _ => Err(DecodeError::UnknownPacketType { type_id }),
        }
    }
}

#[derive(Encoder, Decoder, Debug)]
pub struct Handshake {
    #[data_type(with = "var_int")]
    pub protocol_version: i32,
    #[data_type(max_length = 255)]
    pub server_addr: String,
    pub server_port: u16,
    #[data_type(with = "var_int")]
    pub next_state: i32,
}

impl Handshake {
    pub fn new(
        protocol_version: i32,
        server_addr: String,
        server_port: u16,
        next_state: i32,
    ) -> HandshakeServerBoundPacket {
        let handshake = Handshake {
            protocol_version,
            server_addr,
            server_port,
            next_state,
        };

        HandshakeServerBoundPacket::Handshake(handshake)
    }
}
