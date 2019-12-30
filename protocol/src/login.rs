use std::io::{Read, Write};

use mc_varint::{VarIntRead, VarIntWrite};
use uuid::Uuid;

use crate::chat::Message;
use crate::{DecodeError, EncodeError, Packet, PacketRead, PacketWrite, STRING_MAX_LENGTH};

const LOGIN_MAX_LENGTH: u32 = 16;
const SERVER_ID_MAX_LENGTH: u32 = 20;
const HYPHENATED_UUID_LENGTH: u32 = 36;

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

pub struct LoginDisconnect {
    pub reason: Message,
}

impl LoginDisconnect {
    pub fn new(reason: Message) -> LoginClientBoundPacket {
        let login_disconnect = LoginDisconnect { reason };

        LoginClientBoundPacket::LoginDisconnect(login_disconnect)
    }
}

impl Packet for LoginDisconnect {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_chat_message(&self.reason)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let reason = reader.read_chat_message()?;

        Ok(LoginDisconnect { reason })
    }
}

pub struct EncryptionRequest {
    pub server_id: String,
    pub public_key: Vec<u8>,
    pub verify_token: Vec<u8>,
}

impl EncryptionRequest {
    pub fn new(
        server_id: String,
        public_key: Vec<u8>,
        verify_token: Vec<u8>,
    ) -> LoginClientBoundPacket {
        let encryption_request = EncryptionRequest {
            server_id,
            public_key,
            verify_token,
        };

        LoginClientBoundPacket::EncryptionRequest(encryption_request)
    }
}

impl Packet for EncryptionRequest {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_string(&self.server_id, SERVER_ID_MAX_LENGTH)?;
        writer.write_byte_array(&self.public_key)?;
        writer.write_byte_array(&self.verify_token)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let server_id = reader.read_string(SERVER_ID_MAX_LENGTH)?;
        let public_key = reader.read_byte_array()?;
        let verify_token = reader.read_byte_array()?;

        Ok(EncryptionRequest {
            server_id,
            public_key,
            verify_token,
        })
    }
}

pub struct LoginSuccess {
    pub uuid: Uuid,
    pub username: String,
}

impl LoginSuccess {
    pub fn new(uuid: Uuid, username: String) -> LoginClientBoundPacket {
        let login_success = LoginSuccess { uuid, username };

        LoginClientBoundPacket::LoginSuccess(login_success)
    }
}

impl Packet for LoginSuccess {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        let uuid_hyphenated_string = self.uuid.to_hyphenated().to_string();

        writer.write_string(&uuid_hyphenated_string, HYPHENATED_UUID_LENGTH)?;
        writer.write_string(&self.username, LOGIN_MAX_LENGTH)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let uuid_hyphenated_string = reader.read_string(HYPHENATED_UUID_LENGTH)?;

        let uuid = Uuid::parse_str(&uuid_hyphenated_string)?;
        let username = reader.read_string(LOGIN_MAX_LENGTH)?;

        Ok(LoginSuccess { uuid, username })
    }
}

pub struct SetCompression {
    pub threshold: i32,
}

impl SetCompression {
    pub fn new(threshold: i32) -> LoginClientBoundPacket {
        let set_compression = SetCompression { threshold };

        LoginClientBoundPacket::SetCompression(set_compression)
    }
}

impl Packet for SetCompression {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_var_i32(self.threshold)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let threshold = reader.read_var_i32()?;

        Ok(SetCompression { threshold })
    }
}

pub struct LoginPluginRequest {
    pub message_id: i32,
    pub channel: String,
    pub data: Vec<u8>,
}

impl LoginPluginRequest {
    pub fn new(message_id: i32, channel: String, data: Vec<u8>) -> LoginClientBoundPacket {
        let login_plugin_request = LoginPluginRequest {
            message_id,
            channel,
            data,
        };

        LoginClientBoundPacket::LoginPluginRequest(login_plugin_request)
    }
}

impl Packet for LoginPluginRequest {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_var_i32(self.message_id)?;
        writer.write_string(&self.channel, STRING_MAX_LENGTH)?;
        writer.write_all(&self.data)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let message_id = reader.read_var_i32()?;
        let channel = reader.read_string(STRING_MAX_LENGTH)?;
        let mut data = Vec::new();
        reader.read_to_end(data.as_mut())?;

        Ok(LoginPluginRequest {
            message_id,
            channel,
            data,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::login::LoginPluginResponse;
    use crate::login::LoginStart;
    use crate::Packet;
    use std::io::Cursor;

    #[test]
    fn test_login_start_packet_encode() {
        let login_start = LoginStart {
            name: String::from("Username"),
        };

        let mut vec = Vec::new();
        login_start.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../test/packet/login/login_start.dat").to_vec()
        );
    }

    #[test]
    fn test_login_start_packet_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../test/packet/login/login_start.dat").to_vec());
        let login_start = LoginStart::decode(&mut cursor).unwrap();

        assert_eq!(login_start.name, String::from("Username"));
    }

    #[test]
    fn test_login_plugin_response_encode() {
        let login_plugin_response = LoginPluginResponse {
            message_id: 55,
            successful: true,
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        };

        let mut vec = Vec::new();
        login_plugin_response.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../test/packet/login/login_plugin_response.dat").to_vec()
        );
    }

    #[test]
    fn test_login_plugin_response_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../test/packet/login/login_plugin_response.dat").to_vec());
        let login_plugin_response = LoginPluginResponse::decode(&mut cursor).unwrap();

        assert_eq!(login_plugin_response.message_id, 55);
        assert!(login_plugin_response.successful);
        assert_eq!(
            login_plugin_response.data,
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
        );
    }
}
