// This file is automatically generated.
// It is not intended for manual editing.
use crate::DecodeError;
use crate::Decoder;
use minecraft_protocol_derive::Packet;
use std::io::Read;

pub enum ServerBoundLoginPacket {
    LoginStart(LoginStart),
    EncryptionResponse(EncryptionResponse),
    LoginPluginResponse(LoginPluginResponse),
}

impl ServerBoundLoginPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            Self::LoginStart(_) => 0x00,
            Self::EncryptionResponse(_) => 0x01,
            Self::LoginPluginResponse(_) => 0x02,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x00 => {
                let login_start = LoginStart::decode(reader)?;

                Ok(Self::LoginStart(login_start))
            }
            0x01 => {
                let encryption_response = EncryptionResponse::decode(reader)?;

                Ok(Self::EncryptionResponse(encryption_response))
            }
            0x02 => {
                let login_plugin_response = LoginPluginResponse::decode(reader)?;

                Ok(Self::LoginPluginResponse(login_plugin_response))
            }
            _ => Err(DecodeError::UnknownPacketType { type_id }),
        }
    }

    pub fn login_start(username: String) -> Self {
        let login_start = LoginStart { username };

        Self::LoginStart(login_start)
    }

    pub fn encryption_response(shared_secret: Vec<u8>, verify_token: Vec<u8>) -> Self {
        let encryption_response = EncryptionResponse {
            shared_secret,
            verify_token,
        };

        Self::EncryptionResponse(encryption_response)
    }

    pub fn login_plugin_response(message_id: i32, data: Vec<u8>) -> Self {
        let login_plugin_response = LoginPluginResponse { message_id, data };

        Self::LoginPluginResponse(login_plugin_response)
    }
}

pub enum ClientBoundLoginPacket {
    Disconnect(Disconnect),
    EncryptionRequest(EncryptionRequest),
    Success(Success),
    Compress(Compress),
    LoginPluginRequest(LoginPluginRequest),
}

impl ClientBoundLoginPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            Self::Disconnect(_) => 0x00,
            Self::EncryptionRequest(_) => 0x01,
            Self::Success(_) => 0x02,
            Self::Compress(_) => 0x03,
            Self::LoginPluginRequest(_) => 0x04,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x00 => {
                let disconnect = Disconnect::decode(reader)?;

                Ok(Self::Disconnect(disconnect))
            }
            0x01 => {
                let encryption_request = EncryptionRequest::decode(reader)?;

                Ok(Self::EncryptionRequest(encryption_request))
            }
            0x02 => {
                let success = Success::decode(reader)?;

                Ok(Self::Success(success))
            }
            0x03 => {
                let compress = Compress::decode(reader)?;

                Ok(Self::Compress(compress))
            }
            0x04 => {
                let login_plugin_request = LoginPluginRequest::decode(reader)?;

                Ok(Self::LoginPluginRequest(login_plugin_request))
            }
            _ => Err(DecodeError::UnknownPacketType { type_id }),
        }
    }

    pub fn disconnect(reason: String) -> Self {
        let disconnect = Disconnect { reason };

        Self::Disconnect(disconnect)
    }

    pub fn encryption_request(
        server_id: String,
        public_key: Vec<u8>,
        verify_token: Vec<u8>,
    ) -> Self {
        let encryption_request = EncryptionRequest {
            server_id,
            public_key,
            verify_token,
        };

        Self::EncryptionRequest(encryption_request)
    }

    pub fn success(uuid: String, username: String) -> Self {
        let success = Success { uuid, username };

        Self::Success(success)
    }

    pub fn compress(threshold: i32) -> Self {
        let compress = Compress { threshold };

        Self::Compress(compress)
    }

    pub fn login_plugin_request(message_id: i32, channel: String, data: Vec<u8>) -> Self {
        let login_plugin_request = LoginPluginRequest {
            message_id,
            channel,
            data,
        };

        Self::LoginPluginRequest(login_plugin_request)
    }
}

#[derive(Packet, Debug)]
pub struct LoginStart {
    pub username: String,
}

#[derive(Packet, Debug)]
pub struct EncryptionResponse {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

#[derive(Packet, Debug)]
pub struct LoginPluginResponse {
    #[packet(with = "var_int")]
    pub message_id: i32,
    #[packet(with = "rest")]
    pub data: Vec<u8>,
}

#[derive(Packet, Debug)]
pub struct Disconnect {
    pub reason: String,
}

#[derive(Packet, Debug)]
pub struct EncryptionRequest {
    pub server_id: String,
    pub public_key: Vec<u8>,
    pub verify_token: Vec<u8>,
}

#[derive(Packet, Debug)]
pub struct Success {
    pub uuid: String,
    pub username: String,
}

#[derive(Packet, Debug)]
pub struct Compress {
    #[packet(with = "var_int")]
    pub threshold: i32,
}

#[derive(Packet, Debug)]
pub struct LoginPluginRequest {
    #[packet(with = "var_int")]
    pub message_id: i32,
    pub channel: String,
    #[packet(with = "rest")]
    pub data: Vec<u8>,
}