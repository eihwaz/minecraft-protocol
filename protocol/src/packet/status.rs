use crate::data::status::ServerStatus;
use crate::DecodeError;
use crate::Decoder;
use minecraft_protocol_derive::Packet;
use std::io::Read;

pub enum StatusServerBoundPacket {
    StatusRequest,
    PingRequest(PingRequest),
}

pub enum StatusClientBoundPacket {
    StatusResponse(StatusResponse),
    PingResponse(PingResponse),
}

impl StatusServerBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            StatusServerBoundPacket::StatusRequest => 0x00,
            StatusServerBoundPacket::PingRequest(_) => 0x01,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x00 => Ok(StatusServerBoundPacket::StatusRequest),
            0x01 => {
                let ping_request = PingRequest::decode(reader)?;

                Ok(StatusServerBoundPacket::PingRequest(ping_request))
            }
            _ => Err(DecodeError::UnknownPacketType { type_id }),
        }
    }
}

impl StatusClientBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            StatusClientBoundPacket::StatusResponse(_) => 0x00,
            StatusClientBoundPacket::PingResponse(_) => 0x01,
        }
    }
}

#[derive(Packet, Debug)]
pub struct PingRequest {
    pub time: u64,
}

#[derive(Packet, Debug)]
pub struct PingResponse {
    pub time: u64,
}

#[derive(Packet, Debug)]
pub struct StatusResponse {
    pub server_status: ServerStatus,
}
