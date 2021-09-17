use crate::error::EncodeError;
use byteorder::{BigEndian, WriteBytesExt};
use nbt::CompoundTag;
use num_traits::ToPrimitive;
use std::io::Write;
use uuid::Uuid;

pub trait Encoder {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError>;
}

/// Trait adds additional helper methods for `Write` to write protocol data.
pub trait EncoderWriteExt {
    fn write_bool(&mut self, value: bool) -> Result<(), EncodeError>;

    fn write_string(&mut self, value: &str, max_length: u16) -> Result<(), EncodeError>;

    fn write_byte_array(&mut self, value: &[u8]) -> Result<(), EncodeError>;

    fn write_enum<T: ToPrimitive>(&mut self, value: &T) -> Result<(), EncodeError>;

    fn write_compound_tag(&mut self, value: &CompoundTag) -> Result<(), EncodeError>;

    fn write_var_i32(&mut self, value: i32) -> Result<(), EncodeError>;

    fn write_var_i64(&mut self, value: i64) -> Result<(), EncodeError>;
}

macro_rules! write_signed_var_int (
    ($type: ident, $name: ident) => (
        fn $name(&mut self, mut value: $type) -> Result<(), EncodeError> {
            loop {
                let mut byte = (value & 0b01111111) as u8;
                value = value >> 7;

                if value != 0 {
                    byte |= 0b10000000;
                }

                self.write_u8(byte)?;

                if value == 0 {
                   break;
                }
            }

            Ok(())
        }
    )
);

impl<W: Write> EncoderWriteExt for W {
    fn write_bool(&mut self, value: bool) -> Result<(), EncodeError> {
        if value {
            self.write_u8(1)?;
        } else {
            self.write_u8(0)?;
        }

        Ok(())
    }

    fn write_string(&mut self, value: &str, max_length: u16) -> Result<(), EncodeError> {
        let length = value.len();

        if length > max_length as usize {
            return Err(EncodeError::StringTooLong { length, max_length });
        }

        self.write_var_i32(value.len() as i32)?;
        self.write_all(value.as_bytes())?;

        Ok(())
    }

    fn write_byte_array(&mut self, value: &[u8]) -> Result<(), EncodeError> {
        self.write_var_i32(value.len() as i32)?;
        self.write_all(value)?;

        Ok(())
    }

    fn write_enum<T: ToPrimitive>(&mut self, value: &T) -> Result<(), EncodeError> {
        let type_value = ToPrimitive::to_u8(value).unwrap();
        self.write_u8(type_value)?;

        Ok(())
    }

    fn write_compound_tag(&mut self, value: &CompoundTag) -> Result<(), EncodeError> {
        nbt::encode::write_compound_tag(self, value.clone())?;

        Ok(())
    }

    write_signed_var_int!(i32, write_var_i32);
    write_signed_var_int!(i64, write_var_i64);
}

impl Encoder for u8 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_u8(*self)?)
    }
}

impl Encoder for i16 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_i16::<BigEndian>(*self)?)
    }
}

impl Encoder for i32 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_i32::<BigEndian>(*self)?)
    }
}

impl Encoder for u16 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_u16::<BigEndian>(*self)?)
    }
}

impl Encoder for u32 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_u32::<BigEndian>(*self)?)
    }
}

impl Encoder for i64 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_i64::<BigEndian>(*self)?)
    }
}

impl Encoder for u64 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_u64::<BigEndian>(*self)?)
    }
}

impl Encoder for f32 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_f32::<BigEndian>(*self)?)
    }
}

impl Encoder for f64 {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_f64::<BigEndian>(*self)?)
    }
}

impl Encoder for String {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_string(self, 32_768)?)
    }
}

impl Encoder for bool {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_bool(*self)?)
    }
}

impl Encoder for Vec<u8> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_byte_array(self)?)
    }
}

impl Encoder for Uuid {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_all(self.as_bytes())?)
    }
}

impl Encoder for CompoundTag {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        Ok(writer.write_compound_tag(self)?)
    }
}

impl Encoder for Vec<CompoundTag> {
    fn encode<W: Write>(&self, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_var_i32(self.len() as i32)?;

        for compound_tag in self {
            writer.write_compound_tag(&compound_tag)?;
        }

        Ok(())
    }
}

pub mod var_int {
    use crate::encoder::EncoderWriteExt;
    use crate::error::EncodeError;
    use std::io::Write;

    pub fn encode<W: Write>(value: &i32, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_var_i32(*value)?;

        Ok(())
    }
}

pub mod var_long {
    use crate::encoder::EncoderWriteExt;
    use crate::error::EncodeError;
    use std::io::Write;

    pub fn encode<W: Write>(value: &i64, writer: &mut W) -> Result<(), EncodeError> {
        writer.write_var_i64(*value)?;

        Ok(())
    }
}

pub mod rest {
    use crate::error::EncodeError;
    use std::io::Write;

    pub fn encode<W: Write>(value: &[u8], writer: &mut W) -> Result<(), EncodeError> {
        writer.write_all(value)?;

        Ok(())
    }
}

pub mod uuid_hyp_str {
    use crate::encoder::EncoderWriteExt;
    use crate::error::EncodeError;
    use std::io::Write;
    use uuid::Uuid;

    pub fn encode<W: Write>(value: &Uuid, writer: &mut W) -> Result<(), EncodeError> {
        let uuid_hyphenated_string = value.to_hyphenated().to_string();
        writer.write_string(&uuid_hyphenated_string, 36)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::encoder::EncoderWriteExt;
    use std::io::Cursor;

    #[test]
    fn test_write_variable_i32_2_bytes_value() {
        let mut cursor = Cursor::new(Vec::with_capacity(5));
        cursor.write_var_i32(300).unwrap();

        assert_eq!(cursor.into_inner(), vec![0b10101100, 0b00000010]);
    }

    #[test]
    fn test_write_variable_i32_5_bytes_value() {
        let mut cursor = Cursor::new(Vec::with_capacity(5));
        cursor.write_var_i32(2147483647).unwrap();

        assert_eq!(cursor.into_inner(), vec![0xff, 0xff, 0xff, 0xff, 0x07]);
    }
}
