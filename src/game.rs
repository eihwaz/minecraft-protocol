use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};

use crate::chat::Message;
use crate::{DecodeError, EncodeError, Packet, PacketRead, PacketWrite};

const SERVER_BOUND_CHAT_MESSAGE_MAX_LENGTH: u32 = 256;

pub enum GameServerBoundPacket {
    ServerBoundChatMessage(ServerBoundChatMessage),
}

pub enum GameClientBoundPacket {
    ClientBoundChatMessage(ClientBoundChatMessage),
}

impl GameServerBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            GameServerBoundPacket::ServerBoundChatMessage(_) => 0x03,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x03 => {
                let chat_message = ServerBoundChatMessage::decode(reader)?;

                Ok(GameServerBoundPacket::ServerBoundChatMessage(chat_message))
            }
            _ => Err(DecodeError::UnknownPacketType { type_id }),
        }
    }
}

impl GameClientBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            GameClientBoundPacket::ClientBoundChatMessage(_) => 0x0E,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x0E => {
                let chat_message = ClientBoundChatMessage::decode(reader)?;

                Ok(GameClientBoundPacket::ClientBoundChatMessage(chat_message))
            }
            _ => Err(DecodeError::UnknownPacketType { type_id }),
        }
    }
}

pub struct ServerBoundChatMessage {
    pub message: String,
}

impl ServerBoundChatMessage {
    pub fn new(message: String) -> GameServerBoundPacket {
        let chat_message = ServerBoundChatMessage { message };

        GameServerBoundPacket::ServerBoundChatMessage(chat_message)
    }
}

impl Packet for ServerBoundChatMessage {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_string(&self.message, SERVER_BOUND_CHAT_MESSAGE_MAX_LENGTH)
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let message = reader.read_string(SERVER_BOUND_CHAT_MESSAGE_MAX_LENGTH)?;

        Ok(ServerBoundChatMessage { message })
    }
}

pub struct ClientBoundChatMessage {
    pub message: Message,
    pub position: MessagePosition,
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum MessagePosition {
    Chat,
    System,
    HotBar,
}

impl ClientBoundChatMessage {
    pub fn new(message: Message, position: MessagePosition) -> GameClientBoundPacket {
        let chat_message = ClientBoundChatMessage { message, position };

        GameClientBoundPacket::ClientBoundChatMessage(chat_message)
    }
}

impl Packet for ClientBoundChatMessage {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_chat_message(&self.message)?;
        writer.write_u8(ToPrimitive::to_u8(&self.position).unwrap())?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let message = reader.read_chat_message()?;
        let position_type_id = reader.read_u8()?;

        let position = FromPrimitive::from_u8(position_type_id).ok_or_else(|| {
            DecodeError::UnknownEnumType {
                type_id: position_type_id,
            }
        })?;

        let chat_message = ClientBoundChatMessage { message, position };

        Ok(chat_message)
    }
}
