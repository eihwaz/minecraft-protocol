use crate::{DecodePacketError, Packet};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io;
use std::io::{Read, Write};

pub enum StatusServerBoundPacket {
    StatusRequest,
    PingRequest(PingRequest),
}

pub enum StatusClientBoundPacket {
    StatusResponse,
    PingResponse(PingResponse),
}

impl StatusServerBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            StatusServerBoundPacket::StatusRequest => 0x0,
            StatusServerBoundPacket::PingRequest(_) => 0x1,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodePacketError> {
        match type_id {
            0x0 => Ok(StatusServerBoundPacket::StatusRequest),
            0x1 => {
                let ping_request = PingRequest::decode(reader)?;

                Ok(StatusServerBoundPacket::PingRequest(ping_request))
            }
            _ => Err(DecodePacketError::UnknownPacketType { type_id }),
        }
    }
}

impl StatusClientBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            StatusClientBoundPacket::StatusResponse => 0x0,
            StatusClientBoundPacket::PingResponse(_) => 0x1,
        }
    }
}

pub struct PingRequest {
    time: u64,
}

impl PingRequest {
    pub fn new(time: u64) -> StatusServerBoundPacket {
        let ping_request = PingRequest { time };

        StatusServerBoundPacket::PingRequest(ping_request)
    }
}

impl Packet for PingRequest {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u64::<BigEndian>(self.time)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodePacketError> {
        let time = reader.read_u64::<BigEndian>()?;

        Ok(PingRequest { time })
    }
}

pub struct PingResponse {
    time: u64,
}

impl PingResponse {
    pub fn new(time: u64) -> StatusClientBoundPacket {
        let ping_response = PingResponse { time };

        StatusClientBoundPacket::PingResponse(ping_response)
    }
}

impl Packet for PingResponse {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), io::Error> {
        writer.write_u64::<BigEndian>(self.time)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodePacketError> {
        let time = reader.read_u64::<BigEndian>()?;

        Ok(PingResponse { time })
    }
}
