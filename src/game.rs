use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};

use crate::chat::Message;
use crate::{DecodeError, EncodeError, Packet, PacketRead, PacketWrite};
use mc_varint::{VarIntRead, VarIntWrite};

const SERVER_BOUND_CHAT_MESSAGE_MAX_LENGTH: u32 = 256;
const LEVEL_TYPE_MAX_LENGTH: u32 = 16;

pub enum GameServerBoundPacket {
    ServerBoundChatMessage(ServerBoundChatMessage),
}

pub enum GameClientBoundPacket {
    ClientBoundChatMessage(ClientBoundChatMessage),
    JoinGame(JoinGame),
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
            GameClientBoundPacket::JoinGame(_) => 0x25,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x0E => {
                let chat_message = ClientBoundChatMessage::decode(reader)?;

                Ok(GameClientBoundPacket::ClientBoundChatMessage(chat_message))
            }
            0x25 => {
                let join_game = JoinGame::decode(reader)?;

                Ok(GameClientBoundPacket::JoinGame(join_game))
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
        writer.write_enum(&self.position)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let message = reader.read_chat_message()?;
        let position = reader.read_enum()?;

        let chat_message = ClientBoundChatMessage { message, position };

        Ok(chat_message)
    }
}

#[derive(Debug)]
pub struct JoinGame {
    pub entity_id: u32,
    pub game_mode: GameMode,
    pub dimension: i32,
    pub max_players: u8,
    pub level_type: String,
    pub view_distance: u8,
    pub reduced_debug_info: bool,
}

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
    Hardcore = 8,
}

impl JoinGame {
    pub fn new(
        entity_id: u32,
        game_mode: GameMode,
        dimension: i32,
        max_players: u8,
        level_type: String,
        view_distance: u8,
        reduced_debug_info: bool,
    ) -> GameClientBoundPacket {
        let join_game = JoinGame {
            entity_id,
            game_mode,
            dimension,
            max_players,
            level_type,
            view_distance,
            reduced_debug_info,
        };

        GameClientBoundPacket::JoinGame(join_game)
    }
}

impl Packet for JoinGame {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_u32::<BigEndian>(self.entity_id)?;
        writer.write_enum(&self.game_mode)?;
        writer.write_i32::<BigEndian>(self.dimension)?;
        writer.write_u8(self.max_players)?;
        writer.write_string(&self.level_type, LEVEL_TYPE_MAX_LENGTH)?;
        writer.write_var_u32(self.view_distance as u32)?;
        writer.write_bool(self.reduced_debug_info)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let entity_id = reader.read_u32::<BigEndian>()?;
        let game_mode = reader.read_enum()?;
        let dimension = reader.read_i32::<BigEndian>()?;
        let max_players = reader.read_u8()?;
        let level_type = reader.read_string(LEVEL_TYPE_MAX_LENGTH)?;
        let view_distance = reader.read_var_u32()? as u8;
        let reduced_debug_info = reader.read_bool()?;

        Ok(JoinGame {
            entity_id,
            game_mode,
            dimension,
            max_players,
            level_type,
            view_distance,
            reduced_debug_info,
        })
    }
}
