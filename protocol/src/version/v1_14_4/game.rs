use crate::data::chat::Message;
use crate::decoder::Decoder;
use crate::decoder::DecoderReadExt;
use crate::encoder::EncoderWriteExt;
use crate::error::DecodeError;
use byteorder::{ReadBytesExt, WriteBytesExt};
use minecraft_protocol_derive::{Decoder, Encoder};
use nbt::CompoundTag;
use std::io::Read;
use uuid::Uuid;

#[derive(Debug)]
pub enum GameServerBoundPacket {
    ServerBoundChatMessage(ServerBoundChatMessage),
    ServerBoundKeepAlive(ServerBoundKeepAlive),
    ServerBoundAbilities(ServerBoundAbilities),
}

#[derive(Debug)]
pub enum GameClientBoundPacket {
    ClientBoundChatMessage(ClientBoundChatMessage),
    JoinGame(JoinGame),
    ClientBoundKeepAlive(ClientBoundKeepAlive),
    ChunkData(ChunkData),
    GameDisconnect(GameDisconnect),
    BossBar(BossBar),
    EntityAction(EntityAction),
}

impl GameServerBoundPacket {
    pub fn get_type_id(&self) -> u8 {
        match self {
            GameServerBoundPacket::ServerBoundChatMessage(_) => 0x03,
            GameServerBoundPacket::ServerBoundKeepAlive(_) => 0x0F,
            GameServerBoundPacket::ServerBoundAbilities(_) => 0x19,
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
            GameClientBoundPacket::BossBar(_) => 0x0D,
            GameClientBoundPacket::EntityAction(_) => 0x1B,
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

#[derive(Encoder, Decoder, Debug)]
pub struct ServerBoundChatMessage {
    #[data_type(max_length = 256)]
    pub message: String,
}

impl ServerBoundChatMessage {
    pub fn new(message: String) -> GameServerBoundPacket {
        let chat_message = ServerBoundChatMessage { message };

        GameServerBoundPacket::ServerBoundChatMessage(chat_message)
    }
}

#[derive(Encoder, Decoder, Debug)]
pub struct ClientBoundChatMessage {
    pub message: Message,
    pub position: MessagePosition,
}

#[derive(Encoder, Decoder, Debug, Eq, PartialEq)]
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

#[derive(Encoder, Decoder, Debug)]
pub struct JoinGame {
    pub entity_id: u32,
    pub game_mode: GameMode,
    pub dimension: i32,
    pub max_players: u8,
    #[data_type(max_length = 16)]
    pub level_type: String,
    #[data_type(with = "var_int")]
    pub view_distance: i32,
    pub reduced_debug_info: bool,
}

#[derive(Encoder, Decoder, Debug, Eq, PartialEq)]
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
        view_distance: i32,
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

#[derive(Encoder, Decoder, Debug)]
pub struct ServerBoundKeepAlive {
    pub id: u64,
}

impl ServerBoundKeepAlive {
    pub fn new(id: u64) -> GameServerBoundPacket {
        let keep_alive = ServerBoundKeepAlive { id };

        GameServerBoundPacket::ServerBoundKeepAlive(keep_alive)
    }
}

#[derive(Encoder, Decoder, Debug)]
pub struct ClientBoundKeepAlive {
    pub id: u64,
}

impl ClientBoundKeepAlive {
    pub fn new(id: u64) -> GameClientBoundPacket {
        let keep_alive = ClientBoundKeepAlive { id };

        GameClientBoundPacket::ClientBoundKeepAlive(keep_alive)
    }
}

#[derive(Encoder, Decoder, Debug)]
pub struct ChunkData {
    pub x: i32,
    pub z: i32,
    pub full: bool,
    #[data_type(with = "var_int")]
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

#[derive(Encoder, Decoder, Debug)]
pub struct GameDisconnect {
    pub reason: Message,
}

impl GameDisconnect {
    pub fn new(reason: Message) -> GameClientBoundPacket {
        let game_disconnect = GameDisconnect { reason };

        GameClientBoundPacket::GameDisconnect(game_disconnect)
    }
}

#[derive(Encoder, Decoder, Debug, PartialEq)]
pub struct BossBar {
    pub id: Uuid,
    pub action: BossBarAction,
}

#[derive(Encoder, Decoder, Debug, PartialEq)]
pub enum BossBarAction {
    Add {
        title: Message,
        health: f32,
        color: BossBarColor,
        division: BossBarDivision,
        flags: u8,
    },
    Remove,
    UpdateHealth {
        health: f32,
    },
    UpdateTitle {
        title: Message,
    },
    UpdateStyle {
        color: BossBarColor,
        division: BossBarDivision,
    },
    UpdateFlags {
        flags: u8,
    },
}

#[derive(Encoder, Decoder, Debug, PartialEq)]
pub enum BossBarColor {
    Pink,
    Blue,
    Red,
    Green,
    Yellow,
    Purple,
    White,
}

#[derive(Encoder, Decoder, Debug, PartialEq)]
pub enum BossBarDivision {
    None,
    Notches6,
    Notches10,
    Notches12,
    Notches20,
}

impl BossBar {
    pub fn new(id: Uuid, action: BossBarAction) -> GameClientBoundPacket {
        let boss_bar = BossBar { id, action };

        GameClientBoundPacket::BossBar(boss_bar)
    }
}

#[derive(Encoder, Decoder, Debug, PartialEq)]
pub struct EntityAction {
    #[data_type(with = "var_int")]
    pub entity_id: i32,
    pub action_id: EntityActionId,
    #[data_type(with = "var_int")]
    pub jump_boost: i32,
}

#[derive(Encoder, Decoder, Debug, PartialEq)]
#[data_type(with = "var_int")]
pub enum EntityActionId {
    StartSneaking,
    StopSneaking,
    LeaveBad,
    StartSprinting,
    StopSprinting,
    StartJumpWithHorse,
    StopJumpWithHorse,
    OpenHorseInventory,
    StartFlyingWithElytra,
}

#[derive(Encoder, Decoder, Debug, PartialEq)]
pub struct ServerBoundAbilities {
    #[data_type(bitfield)]
    pub invulnerable: bool,
    #[data_type(bitfield)]
    pub allow_flying: bool,
    #[data_type(bitfield)]
    pub flying: bool,
    #[data_type(bitfield)]
    pub creative_mode: bool,
    pub fly_speed: f32,
    pub walk_speed: f32,
}

#[cfg(test)]
mod tests {
    use crate::data::chat::Payload;
    use crate::decoder::Decoder;
    use crate::encoder::Encoder;
    use crate::encoder::EncoderWriteExt;
    use crate::error::{DecodeError, EncodeError};
    use crate::version::v1_14_4::game::*;
    use crate::STRING_MAX_LENGTH;
    use nbt::CompoundTag;
    use std::io::Cursor;
    use std::str::FromStr;

    #[test]
    fn test_server_bound_chat_message_encode() {
        let chat_message = ServerBoundChatMessage {
            message: String::from("hello server!"),
        };

        let mut vec = Vec::new();
        chat_message.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../../../test/packet/game/server_bound_chat_message.dat").to_vec()
        );
    }

    #[test]
    fn test_server_bound_chat_message_decode() {
        let mut cursor = Cursor::new(
            include_bytes!("../../../test/packet/game/server_bound_chat_message.dat").to_vec(),
        );
        let chat_message = ServerBoundChatMessage::decode(&mut cursor).unwrap();

        assert_eq!(chat_message.message, "hello server!");
    }

    #[test]
    fn test_server_bound_chat_message_encode_invalid_length() {
        let chat_message = ServerBoundChatMessage {
            message: "abc".repeat(100),
        };

        let mut vec = Vec::new();

        let encode_error = chat_message
            .encode(&mut vec)
            .err()
            .expect("Expected error `StringTooLong` because message has invalid length");

        match encode_error {
            EncodeError::StringTooLong { length, max_length } => {
                assert_eq!(length, 300);
                assert_eq!(max_length, 256);
            }
            _ => panic!("Expected `StringTooLong` but got `{:?}`", encode_error),
        }
    }

    #[test]
    fn test_server_bound_chat_message_decode_invalid_length() {
        let message = "abc".repeat(100);

        let mut vec = Vec::new();
        vec.write_string(&message, STRING_MAX_LENGTH).unwrap();

        let mut cursor = Cursor::new(vec);

        let decode_error = ServerBoundChatMessage::decode(&mut cursor)
            .err()
            .expect("Expected error `StringTooLong` because message has invalid length");

        match decode_error {
            DecodeError::StringTooLong { length, max_length } => {
                assert_eq!(length, 300);
                assert_eq!(max_length, 256);
            }
            _ => panic!("Expected `StringTooLong` but got `{:?}`", decode_error),
        }
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
            include_bytes!("../../../test/packet/game/client_bound_chat_message.dat").to_vec()
        );
    }

    #[test]
    fn test_client_bound_chat_message_decode() {
        let mut cursor = Cursor::new(
            include_bytes!("../../../test/packet/game/client_bound_chat_message.dat").to_vec(),
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
            include_bytes!("../../../test/packet/game/server_bound_keep_alive.dat").to_vec()
        );
    }

    #[test]
    fn test_server_bound_keep_alive_decode() {
        let mut cursor = Cursor::new(
            include_bytes!("../../../test/packet/game/server_bound_keep_alive.dat").to_vec(),
        );
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
            include_bytes!("../../../test/packet/game/client_bound_keep_alive.dat").to_vec()
        );
    }

    #[test]
    fn test_client_bound_keep_alive_decode() {
        let mut cursor = Cursor::new(
            include_bytes!("../../../test/packet/game/client_bound_keep_alive.dat").to_vec(),
        );
        let keep_alive = ClientBoundKeepAlive::decode(&mut cursor).unwrap();

        assert_eq!(keep_alive.id, 240714);
    }

    #[test]
    fn test_join_game_encode() {
        let join_game = JoinGame {
            entity_id: 27,
            game_mode: GameMode::Hardcore,
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
            include_bytes!("../../../test/packet/game/join_game.dat").to_vec()
        );
    }

    #[test]
    fn test_join_game_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../../../test/packet/game/join_game.dat").to_vec());
        let join_game = JoinGame::decode(&mut cursor).unwrap();

        assert_eq!(join_game.entity_id, 27);
        assert_eq!(join_game.game_mode, GameMode::Hardcore);
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
            include_bytes!("../../../test/packet/game/chunk_data.dat").to_vec()
        );
    }

    #[test]
    fn test_chunk_data_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../../../test/packet/game/chunk_data.dat").to_vec());
        let chunk_data = ChunkData::decode(&mut cursor).unwrap();

        assert_eq!(chunk_data.x, -2);
        assert_eq!(chunk_data.z, 5);
        assert!(chunk_data.full);
        assert_eq!(chunk_data.heights.name, Some(String::from("HeightMaps")));
        assert_eq!(chunk_data.data, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(chunk_data.primary_mask, 65535);
        assert_eq!(chunk_data.tiles[0].name, Some(String::from("TileEntity")));
    }

    #[test]
    fn test_game_disconnect_encode() {
        let game_disconnect = GameDisconnect {
            reason: Message::new(Payload::text("Message")),
        };

        let mut vec = Vec::new();
        game_disconnect.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../../../test/packet/game/game_disconnect.dat").to_vec()
        );
    }

    #[test]
    fn test_game_disconnect_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../../../test/packet/game/game_disconnect.dat").to_vec());
        let game_disconnect = GameDisconnect::decode(&mut cursor).unwrap();

        assert_eq!(
            game_disconnect.reason,
            Message::new(Payload::text("Message"))
        );
    }

    #[test]
    fn test_boss_bar_add_encode() {
        let boss_bar_add = create_boss_bar_add_packet();

        let mut vec = Vec::new();
        boss_bar_add.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../../../test/packet/game/boss_bar_add.dat").to_vec()
        );
    }

    #[test]
    fn test_boss_bar_add_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../../../test/packet/game/boss_bar_add.dat").to_vec());
        let boss_bar_add = BossBar::decode(&mut cursor).unwrap();

        assert_eq!(boss_bar_add, create_boss_bar_add_packet());
    }

    fn create_boss_bar_add_packet() -> BossBar {
        BossBar {
            id: Uuid::from_str("afa32ac8-d3bf-47f3-99eb-294d60b3dca2").unwrap(),
            action: BossBarAction::Add {
                title: Message::from_str("Boss title"),
                health: 123.45,
                color: BossBarColor::Yellow,
                division: BossBarDivision::Notches10,
                flags: 7,
            },
        }
    }

    #[test]
    fn test_boss_bar_remove_encode() {
        let boss_bar_remove = create_boss_bar_remove_packet();

        let mut vec = Vec::new();
        boss_bar_remove.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../../../test/packet/game/boss_bar_remove.dat").to_vec()
        );
    }

    #[test]
    fn test_boss_bar_remove_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../../../test/packet/game/boss_bar_remove.dat").to_vec());
        let boss_bar_remove = BossBar::decode(&mut cursor).unwrap();

        assert_eq!(boss_bar_remove, create_boss_bar_remove_packet());
    }

    fn create_boss_bar_remove_packet() -> BossBar {
        BossBar {
            id: Uuid::from_str("afa32ac8-d3bf-47f3-99eb-294d60b3dca2").unwrap(),
            action: BossBarAction::Remove,
        }
    }

    #[test]
    fn test_entity_action_encode() {
        let entity_action = EntityAction {
            entity_id: 12345,
            action_id: EntityActionId::StartFlyingWithElytra,
            jump_boost: i32::MAX,
        };

        let mut vec = Vec::new();
        entity_action.encode(&mut vec).unwrap();

        assert_eq!(
            vec,
            include_bytes!("../../../test/packet/game/entity_action.dat").to_vec()
        );
    }

    #[test]
    fn test_entity_action_decode() {
        let mut cursor =
            Cursor::new(include_bytes!("../../../test/packet/game/entity_action.dat").to_vec());
        let entity_action = EntityAction::decode(&mut cursor).unwrap();

        assert_eq!(
            entity_action,
            EntityAction {
                entity_id: 12345,
                action_id: EntityActionId::StartFlyingWithElytra,
                jump_boost: i32::MAX,
            }
        );
    }

    #[test]
    fn test_serverbound_abilities_encode() {
        let abilities = ServerBoundAbilities {
            invulnerable: true,
            flying: true,
            allow_flying: false,
            creative_mode: true,
            fly_speed: 0.0,
            walk_speed: 0.0,
        };

        let mut vec = Vec::new();
        abilities.encode(&mut vec).unwrap();

        assert_eq!(vec, [13, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_serverbound_abilities_decode() {
        let vec = [13, 0, 0, 0, 0, 0, 0, 0, 0].to_vec();
        let mut cursor = Cursor::new(vec);

        let abilities = ServerBoundAbilities::decode(&mut cursor).unwrap();
        assert!(abilities.invulnerable);
        assert!(!abilities.allow_flying);
        assert!(abilities.flying);
        assert!(abilities.creative_mode);
    }
}
