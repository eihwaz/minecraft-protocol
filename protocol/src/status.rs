use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{DecodeError, EncodeError, PacketParser, PacketWrite, STRING_MAX_LENGTH};

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

pub struct PingRequest {
    pub time: u64,
}

impl PingRequest {
    pub fn new(time: u64) -> StatusServerBoundPacket {
        let ping_request = PingRequest { time };

        StatusServerBoundPacket::PingRequest(ping_request)
    }
}

impl PacketParser for PingRequest {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_u64::<BigEndian>(self.time)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let time = reader.read_u64::<BigEndian>()?;

        Ok(PingRequest { time })
    }
}

pub struct PingResponse {
    pub time: u64,
}

impl PingResponse {
    pub fn new(time: u64) -> StatusClientBoundPacket {
        let ping_response = PingResponse { time };

        StatusClientBoundPacket::PingResponse(ping_response)
    }
}

impl PacketParser for PingResponse {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_u64::<BigEndian>(self.time)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let time = reader.read_u64::<BigEndian>()?;

        Ok(PingResponse { time })
    }
}

#[derive(Serialize, Deserialize)]
pub struct ServerStatus {
    pub version: ServerVersion,
    pub description: String,
    pub players: OnlinePlayers,
}

#[derive(Serialize, Deserialize)]
pub struct ServerVersion {
    pub name: String,
    pub protocol: u32,
}

#[derive(Serialize, Deserialize)]
pub struct OnlinePlayers {
    pub online: u32,
    pub max: u32,
    pub sample: Vec<OnlinePlayer>,
}

#[derive(Serialize, Deserialize)]
pub struct OnlinePlayer {
    pub id: Uuid,
    pub name: String,
}

pub struct StatusResponse {
    pub server_status: ServerStatus,
}

impl StatusResponse {
    pub fn new(server_status: ServerStatus) -> StatusClientBoundPacket {
        let status_response = StatusResponse { server_status };

        StatusClientBoundPacket::StatusResponse(status_response)
    }
}

impl PacketParser for StatusResponse {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        let json = serde_json::to_string(&self.server_status)?;
        writer.write_string(&json, STRING_MAX_LENGTH)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let server_status = serde_json::from_reader(reader)?;
        let status_response = StatusResponse { server_status };

        Ok(status_response)
    }
}
