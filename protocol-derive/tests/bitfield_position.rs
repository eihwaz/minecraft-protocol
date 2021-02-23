#[macro_use]
extern crate minecraft_protocol_derive;

use minecraft_protocol::decoder::Decoder;
use minecraft_protocol::encoder::Encoder;
use minecraft_protocol::error::{DecodeError, EncodeError};

#[derive(Packet)]
pub struct Position {
    #[packet(bitfield(size = 26))]
    pub x: i32,
    #[packet(bitfield(size = 26))]
    pub z: i32,
    #[packet(bitfield(size = 12))]
    pub y: u16,
}

#[cfg(test)]
mod tests {
    use crate::Position;
    use minecraft_protocol::decoder::Decoder;
    use minecraft_protocol::encoder::Encoder;
    use minecraft_protocol::error::{DecodeError, EncodeError};
    use std::io::Cursor;

    #[test]
    fn test_encode_position_i64_bitfield() {
        let position = Position {
            x: 1000,
            y: 64,
            z: -1000,
        };

        let mut vec = Vec::new();

        position
            .encode(&mut vec)
            .expect("Failed to encode position");

        assert_eq!(vec, 275152780755008i64.to_be_bytes().to_vec());
    }

    #[test]
    fn test_decode_position_i64_bitfield() {
        let value = -137164079660992i64;
        let vec = value.to_be_bytes().to_vec();
        let mut cursor = Cursor::new(vec);
        let position = Position::decode(&mut cursor).expect("Failed to decode position");

        assert_eq!(position.x, -500);
        assert_eq!(position.y, 64);
        assert_eq!(position.z, -1000);
    }
}
