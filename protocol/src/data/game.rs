use crate::error::{DecodeError, EncodeError};
use crate::{impl_enum_encoder_decoder, Decoder, DecoderReadExt, Encoder, EncoderWriteExt};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use nbt::CompoundTag;
use num_derive::{FromPrimitive, ToPrimitive};
use std::io::{Read, Write};

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum MessagePosition {
    Chat,
    System,
    HotBar,
}

impl_enum_encoder_decoder!(MessagePosition);

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
    Hardcore = 8,
}

impl_enum_encoder_decoder!(GameMode);

#[derive(Debug, Eq, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i16,
    pub z: i32,
}

impl Encoder for Position {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        let encoded_x = (self.x & 0x3FFFFFF) as i64;
        let encoded_y = (self.y & 0xFFF) as i64;
        let encoded_z = (self.z & 0x3FFFFFF) as i64;

        writer.write_i64::<BigEndian>((encoded_x << 38) | (encoded_z << 12) | encoded_y)?;
        Ok(())
    }
}

impl Decoder for Position {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let encoded = reader.read_i64::<BigEndian>()?;

        let x = (encoded >> 38) as i32;
        let y = (encoded & 0xFFF) as i16;
        let z = (encoded << 26 >> 38) as i32;

        Ok(Position { x, y, z })
    }
}

#[derive(Debug)]
pub struct Slot {
    pub id: i32,
    pub amount: u8,
    pub compound_tag: CompoundTag,
}

impl Encoder for Option<Slot> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        match self {
            Some(slot) => {
                writer.write_bool(true)?;
                slot.encode(writer)
            }
            None => writer.write_bool(false),
        }
    }
}

impl Decoder for Option<Slot> {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        if reader.read_bool()? {
            Ok(Some(Slot::decode(reader)?))
        } else {
            Ok(None)
        }
    }
}

impl Encoder for Slot {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_var_i32(self.id)?;
        writer.write_u8(self.amount)?;
        writer.write_compound_tag(&self.compound_tag)?;

        Ok(())
    }
}

impl Decoder for Slot {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let id = reader.read_var_i32()?;
        let amount = reader.read_u8()?;
        let compound_tag = reader.read_compound_tag()?;

        Ok(Slot {
            id,
            amount,
            compound_tag,
        })
    }
}

#[derive(Debug)]
pub struct Metadata {}

impl Encoder for Metadata {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        unimplemented!()
    }
}

impl Decoder for Metadata {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        unimplemented!()
    }
}

#[derive(Debug)]
pub struct TagsMap {}

impl Encoder for TagsMap {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        unimplemented!()
    }
}

impl Decoder for TagsMap {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        unimplemented!()
    }
}
