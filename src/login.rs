use crate::{DecodeError, EncodeError, Packet, PacketRead, PacketWrite};
use mc_varint::{VarIntRead, VarIntWrite};
use std::io::{Read, Write};

/// Login maximum length.
const LOGIN_MAX_LENGTH: u32 = 16;

pub enum LoginServerBoundPacket {
    LoginStart(LoginStart),
    EncryptionResponse(EncryptionResponse),
    LoginPluginResponse(LoginPluginResponse),
}

pub enum LoginClientBoundPacket {
    Disconnect,
    EncryptionRequest,
    LoginSuccess,
    SetCompression,
    LoginPluginRequest,
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

pub struct LoginStart {
    pub name: String,
}

impl LoginStart {
    pub fn new(name: String) -> LoginServerBoundPacket {
        let login_start = LoginStart { name };

        LoginServerBoundPacket::LoginStart(login_start)
    }
}

impl Packet for LoginStart {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_string(&self.name, LOGIN_MAX_LENGTH)
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let name = reader.read_string(LOGIN_MAX_LENGTH)?;

        Ok(LoginStart { name })
    }
}

pub struct EncryptionResponse {
    pub shared_secret: Vec<u8>,
    pub verify_token: Vec<u8>,
}

impl EncryptionResponse {
    pub fn new(shared_secret: Vec<u8>, verify_token: Vec<u8>) -> LoginServerBoundPacket {
        let encryption_response = EncryptionResponse {
            shared_secret,
            verify_token,
        };

        LoginServerBoundPacket::EncryptionResponse(encryption_response)
    }
}

impl Packet for EncryptionResponse {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_byte_array(&self.shared_secret)?;
        writer.write_byte_array(&self.verify_token)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let shared_secret = reader.read_byte_array()?;
        let verify_token = reader.read_byte_array()?;

        Ok(EncryptionResponse {
            shared_secret,
            verify_token,
        })
    }
}

pub struct LoginPluginResponse {
    pub message_id: i32,
    pub successful: bool,
    pub data: Vec<u8>,
}

impl LoginPluginResponse {
    pub fn new(message_id: i32, successful: bool, data: Vec<u8>) -> LoginServerBoundPacket {
        let login_plugin_response = LoginPluginResponse {
            message_id,
            successful,
            data,
        };

        LoginServerBoundPacket::LoginPluginResponse(login_plugin_response)
    }
}

impl Packet for LoginPluginResponse {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_var_i32(self.message_id)?;
        writer.write_bool(self.successful)?;
        writer.write_all(&self.data)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let message_id = reader.read_var_i32()?;
        let successful = reader.read_bool()?;

        let mut data = Vec::new();
        reader.read_to_end(data.as_mut())?;

        Ok(LoginPluginResponse {
            message_id,
            successful,
            data,
        })
    }
}
