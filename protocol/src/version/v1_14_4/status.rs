use crate::data::server_status::*;
use crate::decoder::Decoder;
use crate::error::DecodeError;
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

impl PingRequest {
    pub fn new(time: u64) -> StatusServerBoundPacket {
        let ping_request = PingRequest { time };

        StatusServerBoundPacket::PingRequest(ping_request)
    }
}

#[derive(Packet, Debug)]
pub struct PingResponse {
    pub time: u64,
}

impl PingResponse {
    pub fn new(time: u64) -> StatusClientBoundPacket {
        let ping_response = PingResponse { time };

        StatusClientBoundPacket::PingResponse(ping_response)
    }
}

#[derive(Packet, Debug)]
pub struct StatusResponse {
    pub server_status: ServerStatus,
}

impl StatusResponse {
    pub fn new(server_status: ServerStatus) -> StatusClientBoundPacket {
        let status_response = StatusResponse { server_status };

        StatusClientBoundPacket::StatusResponse(status_response)
    }
}

#[cfg(test)]
mod tests {
    use crate::data::chat::{Message, Payload};
    use crate::decoder::Decoder;
    use crate::encoder::Encoder;
    use crate::version::v1_14_4::status::*;
    use std::io::Cursor;
    use uuid::Uuid;

    #[test]
    fn test_ping_request_encode() {
        let ping_request = PingRequest {
            time: 1577735845610,
        };

        let mut vec = Vec::new();
        ping_request.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../../../test/packet/status/ping_request.dat").to_vec()
        );
    }

    #[test]
    fn test_status_ping_request_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../../../test/packet/status/ping_request.dat").to_vec());
        let ping_request = PingRequest::decode(&mut cursor).unwrap();

        assert_eq!(ping_request.time, 1577735845610);
    }

    #[test]
    fn test_ping_response_encode() {
        let ping_response = PingResponse {
            time: 1577735845610,
        };

        let mut vec = Vec::new();
        ping_response.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../../../test/packet/status/ping_response.dat").to_vec()
        );
    }

    #[test]
    fn test_status_ping_response_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../../../test/packet/status/ping_response.dat").to_vec());
        let ping_response = PingResponse::decode(&mut cursor).unwrap();

        assert_eq!(ping_response.time, 1577735845610);
    }

    #[test]
    fn test_status_response_encode() {
        let version = ServerVersion {
            name: String::from("1.15.1"),
            protocol: 575,
        };

        let player = OnlinePlayer {
            id: Uuid::parse_str("2a1e1912-7103-4add-80fc-91ebc346cbce").unwrap(),
            name: String::from("Username"),
        };

        let players = OnlinePlayers {
            online: 10,
            max: 100,
            sample: vec![player],
        };

        let server_status = ServerStatus {
            version,
            description: Message::new(Payload::text("Description")),
            players,
        };

        let status_response = StatusResponse { server_status };

        let mut vec = Vec::new();
        status_response.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../../../test/packet/status/status_response.dat").to_vec()
        );
    }

    #[test]
    fn test_status_response_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../../../test/packet/status/status_response.dat").to_vec());
        let status_response = StatusResponse::decode(&mut cursor).unwrap();
        let server_status = status_response.server_status;

        let player = OnlinePlayer {
            id: Uuid::parse_str("2a1e1912-7103-4add-80fc-91ebc346cbce").unwrap(),
            name: String::from("Username"),
        };

        assert_eq!(server_status.version.name, String::from("1.15.1"));
        assert_eq!(server_status.version.protocol, 575);
        assert_eq!(server_status.players.max, 100);
        assert_eq!(server_status.players.online, 10);
        assert_eq!(server_status.players.sample, vec![player]);
        assert_eq!(
            server_status.description,
            Message::new(Payload::text("Description"))
        );
    }
}
