use crate::data::chat::Message;
use crate::DecodeError;
use crate::Decoder;
use std::io::Read;
use uuid::Uuid;

use minecraft_protocol_derive::Packet;

pub enum LoginServerBoundPacket {
    LoginStart(LoginStart),
    EncryptionResponse(EncryptionResponse),
    LoginPluginResponse(LoginPluginResponse),
}

pub enum LoginClientBoundPacket {
    LoginDisconnect(LoginDisconnect),
    EncryptionRequest(EncryptionRequest),
    LoginSuccess(LoginSuccess),
    SetCompression(SetCompression),
    LoginPluginRequest(LoginPluginRequest),
}

impl LoginServerBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            LoginServerBoundPacket::LoginStart(_) => 0x00,
            LoginServerBoundPacket::EncryptionResponse(_) => 0x01,
            LoginServerBoundPacket::LoginPluginResponse(_) => 0x02,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x00 => {
                let login_start = LoginStart::decode(reader)?;

                Ok(LoginServerBoundPacket::LoginStart(login_start))
            }
            0x01 => {
                let encryption_response = EncryptionResponse::decode(reader)?;

                Ok(LoginServerBoundPacket::EncryptionResponse(
                    encryption_response,
                ))
            }
            0x02 => {
                let login_plugin_response = LoginPluginResponse::decode(reader)?;

                Ok(LoginServerBoundPacket::LoginPluginResponse(
                    login_plugin_response,
                ))
            }
            _ => Err(DecodeError::UnknownPacketType { type_id }),
        }
    }
}

impl LoginClientBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            LoginClientBoundPacket::LoginDisconnect(_) => 0x00,
            LoginClientBoundPacket::EncryptionRequest(_) => 0x01,
            LoginClientBoundPacket::LoginSuccess(_) => 0x02,
            LoginClientBoundPacket::SetCompression(_) => 0x03,
            LoginClientBoundPacket::LoginPluginRequest(_) => 0x04,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x00 => {
                let login_disconnect = LoginDisconnect::decode(reader)?;

                Ok(LoginClientBoundPacket::LoginDisconnect(login_disconnect))
            }
            0x01 => {
                let encryption_request = EncryptionRequest::decode(reader)?;

                Ok(LoginClientBoundPacket::EncryptionRequest(
                    encryption_request,
                ))
            }
            0x02 => {
                let login_success = LoginSuccess::decode(reader)?;

                Ok(LoginClientBoundPacket::LoginSuccess(login_success))
            }
            0x03 => {
                let set_compression = SetCompression::decode(reader)?;

                Ok(LoginClientBoundPacket::SetCompression(set_compression))
            }
            0x04 => {
                let login_plugin_request = LoginPluginRequest::decode(reader)?;

                Ok(LoginClientBoundPacket::LoginPluginRequest(
                    login_plugin_request,
                ))
            }
            _ => Err(DecodeError::UnknownPacketType { type_id }),
        }
    }
}

#[derive(Packet, Debug)]
pub struct LoginStart {
    pub name: String,
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
    pub successful: bool,
    #[packet(with = "rest")]
    pub data: Vec<u8>,
}

#[derive(Packet, Debug)]
pub struct LoginDisconnect {
    pub reason: Message,
}

#[derive(Packet, Debug)]
pub struct EncryptionRequest {
    #[packet(max_length = 20)]
    pub server_id: String,
    pub public_key: Vec<u8>,
    pub verify_token: Vec<u8>,
}

#[derive(Packet, Debug)]
pub struct LoginSuccess {
    #[packet(with = "uuid_hyp_str")]
    pub uuid: Uuid,
    #[packet(max_length = 16)]
    pub username: String,
}

#[derive(Packet, Debug)]
pub struct SetCompression {
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
