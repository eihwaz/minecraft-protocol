use crate::data::chat::Message;
use crate::data::game::{GameMode, MessagePosition};
use crate::DecodeError;
use crate::Decoder;
use minecraft_protocol_derive::Packet;
use nbt::CompoundTag;
use std::io::Read;

pub enum GameServerBoundPacket {
    ServerBoundChatMessage(ServerBoundChatMessage),
    ServerBoundKeepAlive(ServerBoundKeepAlive),
}

pub enum GameClientBoundPacket {
    ClientBoundChatMessage(ClientBoundChatMessage),
    JoinGame(JoinGame),
    ClientBoundKeepAlive(ClientBoundKeepAlive),
    ChunkData(ChunkData),
    GameDisconnect(GameDisconnect),
}

impl GameServerBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            GameServerBoundPacket::ServerBoundChatMessage(_) => 0x03,
            GameServerBoundPacket::ServerBoundKeepAlive(_) => 0x0F,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x03 => {
                let chat_message = ServerBoundChatMessage::decode(reader)?;

                Ok(GameServerBoundPacket::ServerBoundChatMessage(chat_message))
            }
            0x0F => {
                let keep_alive = ServerBoundKeepAlive::decode(reader)?;

                Ok(GameServerBoundPacket::ServerBoundKeepAlive(keep_alive))
            }
            _ => Err(DecodeError::UnknownPacketType { type_id }),
        }
    }
}

impl GameClientBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            GameClientBoundPacket::ClientBoundChatMessage(_) => 0x0E,
            GameClientBoundPacket::GameDisconnect(_) => 0x1A,
            GameClientBoundPacket::ClientBoundKeepAlive(_) => 0x20,
            GameClientBoundPacket::ChunkData(_) => 0x21,
            GameClientBoundPacket::JoinGame(_) => 0x25,
        }
    }

    pub fn decode<R: Read>(type_id: u8, reader: &mut R) -> Result<Self, DecodeError> {
        match type_id {
            0x0E => {
                let chat_message = ClientBoundChatMessage::decode(reader)?;

                Ok(GameClientBoundPacket::ClientBoundChatMessage(chat_message))
            }
            0x1A => {
                let game_disconnect = GameDisconnect::decode(reader)?;

                Ok(GameClientBoundPacket::GameDisconnect(game_disconnect))
            }
            0x20 => {
                let keep_alive = ClientBoundKeepAlive::decode(reader)?;

                Ok(GameClientBoundPacket::ClientBoundKeepAlive(keep_alive))
            }
            0x21 => {
                let chunk_data = ChunkData::decode(reader)?;

                Ok(GameClientBoundPacket::ChunkData(chunk_data))
            }
            0x25 => {
                let join_game = JoinGame::decode(reader)?;

                Ok(GameClientBoundPacket::JoinGame(join_game))
            }
            _ => Err(DecodeError::UnknownPacketType { type_id }),
        }
    }
}

#[derive(Packet, Debug)]
pub struct ServerBoundChatMessage {
    #[packet(max_length = 256)]
    pub message: String,
}

#[derive(Packet, Debug)]
pub struct ClientBoundChatMessage {
    pub message: Message,
    pub position: MessagePosition,
}

#[derive(Packet, Debug)]
pub struct JoinGame {
    pub entity_id: u32,
    pub game_mode: GameMode,
    pub dimension: i32,
    pub max_players: u8,
    #[packet(max_length = 16)]
    pub level_type: String,
    #[packet(with = "var_int")]
    pub view_distance: i32,
    pub reduced_debug_info: bool,
}

#[derive(Packet)]
pub struct ServerBoundKeepAlive {
    pub id: u64,
}

#[derive(Packet)]
pub struct ClientBoundKeepAlive {
    pub id: u64,
}

#[derive(Packet, Debug)]
pub struct ChunkData {
    pub x: i32,
    pub z: i32,
    pub full: bool,
    #[packet(with = "var_int")]
    pub primary_mask: i32,
    pub heights: CompoundTag,
    pub data: Vec<u8>,
    pub tiles: Vec<CompoundTag>,
}

#[derive(Packet, Debug)]
pub struct GameDisconnect {
    pub reason: Message,
}
