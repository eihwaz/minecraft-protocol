#[macro_use]
extern crate minecraft_protocol_derive;

use minecraft_protocol::decoder::Decoder;
use minecraft_protocol::encoder::Encoder;
use minecraft_protocol::error::{DecodeError, EncodeError};

#[derive(Packet)]
pub struct HalfLong {
    #[packet(bitfield(size = 32))]
    pub _unused: u32,
    #[packet(bitfield(size = 32))]
    pub value: u32,
}

#[derive(Packet)]
pub struct HalfInt {
    #[packet(bitfield(size = 16))]
    pub _unused: u16,
    #[packet(bitfield(size = 16))]
    pub value: u16,
}

#[derive(Packet)]
pub struct HalfShort {
    #[packet(bitfield(size = 8))]
    pub _unused: u8,
    #[packet(bitfield(size = 8))]
    pub value: u8,
}

#[cfg(test)]
mod tests {
    use crate::{HalfInt, HalfLong, HalfShort};
    use minecraft_protocol::decoder::Decoder;
    use minecraft_protocol::encoder::Encoder;
    use minecraft_protocol::error::{DecodeError, EncodeError};
    use std::io::Cursor;

    #[test]
    fn test_encode_half_long_u64_bitfield() {
        let half = HalfLong {
            _unused: 0,
            value: u32::MAX,
        };
        let mut vec = Vec::new();

        half.encode(&mut vec).expect("Failed to encode half");
        assert_eq!(vec, u32::MAX.to_be_bytes().to_vec());
    }

    #[test]
    fn test_decode_half_long_u64_bitfield() {
        let value = u32::MAX as u64;

        let vec = value.to_be_bytes().to_vec();
        let mut cursor = Cursor::new(vec);

        let half = HalfLong::decode(&mut cursor).expect("Failed to decode half");
        assert_eq!(half.value, u32::MAX);
    }

    #[test]
    fn test_encode_half_int_u32_bitfield() {
        let half = HalfInt {
            _unused: 0,
            value: u16::MAX,
        };
        let mut vec = Vec::new();

        half.encode(&mut vec).expect("Failed to encode half");
        assert_eq!(vec, u16::MAX.to_be_bytes().to_vec());
    }

    #[test]
    fn test_decode_half_int_u32_bitfield() {
        let value = u16::MAX as u32;

        let vec = value.to_be_bytes().to_vec();
        let mut cursor = Cursor::new(vec);

        let half = HalfInt::decode(&mut cursor).expect("Failed to decode half");
        assert_eq!(half.value, u16::MAX);
    }

    #[test]
    fn test_encode_half_short_u8_bitfield() {
        let half = HalfShort {
            _unused: 0,
            value: u8::MAX,
        };
        let mut vec = Vec::new();

        half.encode(&mut vec).expect("Failed to encode half");
        assert_eq!(vec, u8::MAX.to_be_bytes().to_vec());
    }

    #[test]
    fn test_decode_half_short_u8_bitfield() {
        let value = u8::MAX as u16;
        let vec = value.to_be_bytes().to_vec();
        let mut cursor = Cursor::new(vec);

        let half = HalfShort::decode(&mut cursor).expect("Failed to decode half");
        assert_eq!(half.value, u8::MAX);
    }
}
