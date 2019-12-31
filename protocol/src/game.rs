use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use num_derive::{FromPrimitive, ToPrimitive};

use crate::chat::Message;
use crate::{DecodeError, EncodeError, PacketParser, PacketRead, PacketWrite};
use mc_varint::{VarIntRead, VarIntWrite};
use nbt::CompoundTag;

const SERVER_BOUND_CHAT_MESSAGE_MAX_LENGTH: u32 = 256;
const LEVEL_TYPE_MAX_LENGTH: u32 = 16;

pub enum GameServerBoundPacket {
    ServerBoundChatMessage(ServerBoundChatMessage),
    ServerBoundKeepAlive(ServerBoundKeepAlive),
}

pub enum GameClientBoundPacket {
    ClientBoundChatMessage(ClientBoundChatMessage),
    JoinGame(JoinGame),
    ClientBoundKeepAlive(ClientBoundKeepAlive),
    ChunkData(ChunkData),
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

pub struct ServerBoundChatMessage {
    pub message: String,
}

impl ServerBoundChatMessage {
    pub fn new(message: String) -> GameServerBoundPacket {
        let chat_message = ServerBoundChatMessage { message };

        GameServerBoundPacket::ServerBoundChatMessage(chat_message)
    }
}

impl PacketParser for ServerBoundChatMessage {
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

impl PacketParser for ClientBoundChatMessage {
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

impl PacketParser for JoinGame {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_u32::<BigEndian>(self.entity_id)?;
        writer.write_enum(&self.game_mode)?;
        writer.write_i32::<BigEndian>(self.dimension)?;
        writer.write_u8(self.max_players)?;
        writer.write_string(&self.level_type, LEVEL_TYPE_MAX_LENGTH)?;
        writer.write_var_i32(self.view_distance as i32)?;
        writer.write_bool(self.reduced_debug_info)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let entity_id = reader.read_u32::<BigEndian>()?;
        let game_mode = reader.read_enum()?;
        let dimension = reader.read_i32::<BigEndian>()?;
        let max_players = reader.read_u8()?;
        let level_type = reader.read_string(LEVEL_TYPE_MAX_LENGTH)?;
        let view_distance = reader.read_var_i32()? as u8;
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

pub struct ServerBoundKeepAlive {
    pub id: u64,
}

impl ServerBoundKeepAlive {
    pub fn new(id: u64) -> GameServerBoundPacket {
        let keep_alive = ServerBoundKeepAlive { id };

        GameServerBoundPacket::ServerBoundKeepAlive(keep_alive)
    }
}

impl PacketParser for ServerBoundKeepAlive {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_u64::<BigEndian>(self.id)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let id = reader.read_u64::<BigEndian>()?;

        Ok(ServerBoundKeepAlive { id })
    }
}

pub struct ClientBoundKeepAlive {
    pub id: u64,
}

impl ClientBoundKeepAlive {
    pub fn new(id: u64) -> GameClientBoundPacket {
        let keep_alive = ClientBoundKeepAlive { id };

        GameClientBoundPacket::ClientBoundKeepAlive(keep_alive)
    }
}

impl PacketParser for ClientBoundKeepAlive {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_u64::<BigEndian>(self.id)?;

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let id = reader.read_u64::<BigEndian>()?;

        Ok(ClientBoundKeepAlive { id })
    }
}

pub struct ChunkData {
    pub x: i32,
    pub z: i32,
    pub full: bool,
    pub primary_mask: i32,
    pub heights: CompoundTag,
    pub data: Vec<u8>,
    pub tiles: Vec<CompoundTag>,
}

impl ChunkData {
    pub fn new(
        x: i32,
        z: i32,
        full: bool,
        primary_mask: i32,
        heights: CompoundTag,
        data: Vec<u8>,
        tiles: Vec<CompoundTag>,
    ) -> GameClientBoundPacket {
        let chunk_data = ChunkData {
            x,
            z,
            full,
            primary_mask,
            heights,
            data,
            tiles,
        };

        GameClientBoundPacket::ChunkData(chunk_data)
    }
}

impl PacketParser for ChunkData {
    type Output = Self;

    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_i32::<BigEndian>(self.x)?;
        writer.write_i32::<BigEndian>(self.z)?;
        writer.write_bool(self.full)?;
        writer.write_var_i32(self.primary_mask)?;
        writer.write_compound_tag(&self.heights)?;
        writer.write_byte_array(&self.data)?;
        writer.write_var_i32(self.tiles.len() as i32)?;

        for tile_compound_tag in self.tiles.iter() {
            writer.write_compound_tag(&tile_compound_tag)?;
        }

        Ok(())
    }

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let x = reader.read_i32::<BigEndian>()?;
        let z = reader.read_i32::<BigEndian>()?;
        let full = reader.read_bool()?;
        let primary_mask = reader.read_var_i32()?;
        let heights = reader.read_compound_tag()?;
        let data = reader.read_byte_array()?;

        let tiles_length = reader.read_var_i32()?;
        let mut tiles = Vec::new();

        for _ in 0..tiles_length {
            let tile_compound_tag = reader.read_compound_tag()?;
            tiles.push(tile_compound_tag);
        }

        Ok(ChunkData {
            x,
            z,
            full,
            primary_mask,
            heights,
            data,
            tiles,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::chat::{Message, Payload};
    use crate::game::{
        ChunkData, ClientBoundChatMessage, ClientBoundKeepAlive, GameMode, JoinGame,
        MessagePosition, ServerBoundChatMessage, ServerBoundKeepAlive,
    };
    use crate::PacketParser;
    use nbt::CompoundTag;
    use std::io::Cursor;

    #[test]
    fn test_server_bound_chat_message_encode() {
        let chat_message = ServerBoundChatMessage {
            message: String::from("hello server!"),
        };

        let mut vec = Vec::new();
        chat_message.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../test/packet/game/server_bound_chat_message.dat").to_vec()
        );
    }

    #[test]
    fn test_server_bound_chat_message_decode() {
        let mut cursor = Cursor::new(
            include_bytes!("../test/packet/game/server_bound_chat_message.dat").to_vec(),
        );
        let chat_message = ServerBoundChatMessage::decode(&mut cursor).unwrap();

        assert_eq!(chat_message.message, "hello server!");
    }

    #[test]
    fn test_client_bound_chat_message_encode() {
        let chat_message = ClientBoundChatMessage {
            message: Message::new(Payload::text("hello client!")),
            position: MessagePosition::System,
        };

        let mut vec = Vec::new();
        chat_message.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../test/packet/game/client_bound_chat_message.dat").to_vec()
        );
    }

    #[test]
    fn test_client_bound_chat_message_decode() {
        let mut cursor = Cursor::new(
            include_bytes!("../test/packet/game/client_bound_chat_message.dat").to_vec(),
        );
        let chat_message = ClientBoundChatMessage::decode(&mut cursor).unwrap();

        assert_eq!(
            chat_message.message,
            Message::new(Payload::text("hello client!"))
        );

        assert_eq!(chat_message.position, MessagePosition::System);
    }

    #[test]
    fn test_server_bound_keep_alive_encode() {
        let keep_alive = ServerBoundKeepAlive { id: 31122019 };

        let mut vec = Vec::new();
        keep_alive.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../test/packet/game/server_bound_keep_alive.dat").to_vec()
        );
    }

    #[test]
    fn test_server_bound_keep_alive_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../test/packet/game/server_bound_keep_alive.dat").to_vec());
        let keep_alive = ServerBoundKeepAlive::decode(&mut cursor).unwrap();

        assert_eq!(keep_alive.id, 31122019);
    }

    #[test]
    fn test_client_bound_keep_alive_encode() {
        let keep_alive = ClientBoundKeepAlive { id: 240714 };

        let mut vec = Vec::new();
        keep_alive.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../test/packet/game/client_bound_keep_alive.dat").to_vec()
        );
    }

    #[test]
    fn test_client_bound_keep_alive_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../test/packet/game/client_bound_keep_alive.dat").to_vec());
        let keep_alive = ClientBoundKeepAlive::decode(&mut cursor).unwrap();

        assert_eq!(keep_alive.id, 240714);
    }

    #[test]
    fn test_join_game_encode() {
        let join_game = JoinGame {
            entity_id: 27,
            game_mode: GameMode::Spectator,
            dimension: 23,
            max_players: 100,
            level_type: String::from("default"),
            view_distance: 10,
            reduced_debug_info: true,
        };

        let mut vec = Vec::new();
        join_game.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../test/packet/game/join_game.dat").to_vec()
        );
    }

    #[test]
    fn test_join_game_decode() {
        let mut cursor = Cursor::new(include_bytes!("../test/packet/game/join_game.dat").to_vec());
        let join_game = JoinGame::decode(&mut cursor).unwrap();

        assert_eq!(join_game.entity_id, 27);
        assert_eq!(join_game.game_mode, GameMode::Spectator);
        assert_eq!(join_game.dimension, 23);
        assert_eq!(join_game.max_players, 100);
        assert_eq!(join_game.level_type, String::from("default"));
        assert_eq!(join_game.view_distance, 10);
        assert!(join_game.reduced_debug_info);
    }

    #[test]
    fn test_chunk_data_encode() {
        let chunk_data = ChunkData {
            x: -2,
            z: 5,
            full: true,
            primary_mask: 65535,
            heights: CompoundTag::named("HeightMaps"),
            data: vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
            tiles: vec![CompoundTag::named("TileEntity")],
        };

        let mut vec = Vec::new();
        chunk_data.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../test/packet/game/chunk_data.dat").to_vec()
        );
    }

    #[test]
    fn test_chunk_data_decode() {
        let mut cursor = Cursor::new(include_bytes!("../test/packet/game/chunk_data.dat").to_vec());
        let chunk_data = ChunkData::decode(&mut cursor).unwrap();

        assert_eq!(chunk_data.x, -2);
        assert_eq!(chunk_data.z, 5);
        assert!(chunk_data.full);
        assert_eq!(chunk_data.heights.name, Some(String::from("HeightMaps")));
        assert_eq!(chunk_data.data, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(chunk_data.primary_mask, 65535);
        assert_eq!(chunk_data.tiles[0].name, Some(String::from("TileEntity")));
    }
}
