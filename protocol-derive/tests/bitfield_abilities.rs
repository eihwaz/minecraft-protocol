#[macro_use]
extern crate minecraft_protocol_derive;

use minecraft_protocol::decoder::Decoder;
use minecraft_protocol::encoder::Encoder;
use minecraft_protocol::error::{DecodeError, EncodeError};

#[derive(Packet)]
pub struct Abilities {
    #[packet(bitfield(size = 4))]
    pub _unused: u8,
    #[packet(bitfield(size = 1))]
    pub creative_mode: bool,
    #[packet(bitfield(size = 1))]
    pub allow_flying: bool,
    #[packet(bitfield(size = 1))]
    pub flying: bool,
    #[packet(bitfield(size = 1))]
    pub invulnerable: bool,
}

#[cfg(test)]
mod tests {
    use crate::Abilities;
    use minecraft_protocol::decoder::Decoder;
    use minecraft_protocol::encoder::Encoder;
    use minecraft_protocol::error::{DecodeError, EncodeError};
    use std::io::Cursor;

    #[test]
    fn test_encode_abilities_i8_bitfield() {
        let abilities = Abilities {
            _unused: 0,
            creative_mode: true,
            allow_flying: true,
            flying: true,
            invulnerable: true,
        };
        let mut vec = Vec::new();

        abilities
            .encode(&mut vec)
            .expect("Failed to encode abilities");
        assert_eq!(vec, [15]);
    }

    #[test]
    fn test_decode_abilities_i8_bitfield() {
        let value = 15i8;

        let vec = value.to_be_bytes().to_vec();
        let mut cursor = Cursor::new(vec);

        let abilities = Abilities::decode(&mut cursor).expect("Failed to decode abilities");
        assert!(abilities.invulnerable);
        assert!(abilities.flying);
        assert!(abilities.allow_flying);
        assert!(abilities.creative_mode);
    }
}
