use crate::DecodeError;
use crate::Decoder;
use minecraft_protocol_derive::Packet;
use std::io::Read;

pub enum HandshakeServerBoundPacket {
    SetProtocol(SetProtocol),
}

impl HandshakeServerBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            Self::SetProtocol(_) => 0x00,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x00 => {
                let set_protocol = SetProtocol::decode(reader)?;

                Ok(Self::SetProtocol(set_protocol))
            }
            _ => Err(DecodeError::UnknownPacketType { type_id }),
        }
    }

    pub fn set_protocol(
        protocol_version: i32,
        server_host: String,
        server_port: u16,
        next_state: i32,
    ) -> Self {
        let set_protocol = SetProtocol {
            protocol_version,
            server_host,
            server_port,
            next_state,
        };

        Self::SetProtocol(set_protocol)
    }
}
#[derive(Packet, Debug)]
pub struct SetProtocol {
    #[packet(with = "var_int")]
    pub protocol_version: i32,
    pub server_host: String,
    pub server_port: u16,
    #[packet(with = "var_int")]
    pub next_state: i32,
}
