use crate::error::DecodeError;
use byteorder::{BigEndian, ReadBytesExt};
use nbt::CompoundTag;
use num_traits::FromPrimitive;
use std::io::Read;
use uuid::Uuid;

pub trait Decoder {
    type Output;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError>;
}

/// Trait adds additional helper methods for `Read` to read protocol data.
trait DecoderReadExt {
    fn read_bool(&mut self) -> Result<bool, DecodeError>;

    fn read_string(&mut self, max_length: u16) -> Result<String, DecodeError>;

    fn read_byte_array(&mut self) -> Result<Vec<u8>, DecodeError>;

    fn read_enum<T: FromPrimitive>(&mut self) -> Result<T, DecodeError>;

    fn read_compound_tag(&mut self) -> Result<CompoundTag, DecodeError>;

    fn read_var_i32(&mut self) -> Result<i32, DecodeError>;

    fn read_var_i64(&mut self) -> Result<i64, DecodeError>;
}

macro_rules! read_signed_var_int (
    ($type: ident, $name: ident, $max_bytes: expr) => (
        fn $name(&mut self) -> Result<$type, DecodeError> {
            let mut bytes = 0;
            let mut output = 0;

            loop {
                let byte = self.read_u8()?;
                let value = (byte & 0b01111111) as $type;

                output |= value << 7 * bytes;
                bytes += 1;

                if bytes > $max_bytes {
                    return Err(DecodeError::VarIntTooLong { max_bytes: $max_bytes })
                }

                if (byte & 0b10000000) == 0 {
                    break;
                }
            }

            Ok(output)
        }
   );
);

impl<R: Read> DecoderReadExt for R {
    fn read_bool(&mut self) -> Result<bool, DecodeError> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(DecodeError::NonBoolValue),
        }
    }

    fn read_string(&mut self, max_length: u16) -> Result<String, DecodeError> {
        let length = self.read_var_i32()? as usize;

        if length as u16 > max_length {
            return Err(DecodeError::StringTooLong { length, max_length });
        }

        let mut buf = vec![0; length as usize];
        self.read_exact(&mut buf)?;

        Ok(String::from_utf8(buf)?)
    }

    fn read_byte_array(&mut self) -> Result<Vec<u8>, DecodeError> {
        let length = self.read_var_i32()?;

        let mut buf = vec![0; length as usize];
        self.read_exact(&mut buf)?;

        Ok(buf)
    }

    fn read_enum<T: FromPrimitive>(&mut self) -> Result<T, DecodeError> {
        let type_id = self.read_u8()?;
        let result = FromPrimitive::from_u8(type_id);

        result.ok_or_else(|| DecodeError::UnknownEnumType { type_id })
    }

    fn read_compound_tag(&mut self) -> Result<CompoundTag, DecodeError> {
        Ok(nbt::decode::read_compound_tag(self)?)
    }

    read_signed_var_int!(i32, read_var_i32, 5);
    read_signed_var_int!(i64, read_var_i64, 10);
}

impl Decoder for u8 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_u8()?)
    }
}

impl Decoder for i16 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_i16::<BigEndian>()?)
    }
}

impl Decoder for i32 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_i32::<BigEndian>()?)
    }
}

impl Decoder for u16 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_u16::<BigEndian>()?)
    }
}

impl Decoder for u32 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_u32::<BigEndian>()?)
    }
}

impl Decoder for i64 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_i64::<BigEndian>()?)
    }
}

impl Decoder for u64 {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_u64::<BigEndian>()?)
    }
}

impl Decoder for String {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_string(32_768)?)
    }
}

impl Decoder for bool {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_bool()?)
    }
}

impl Decoder for Vec<u8> {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_byte_array()?)
    }
}

impl Decoder for Uuid {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let mut buf = [0; 16];
        reader.read_exact(&mut buf)?;

        Ok(Uuid::from_bytes(buf))
    }
}

impl Decoder for CompoundTag {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        Ok(reader.read_compound_tag()?)
    }
}

impl Decoder for Vec<CompoundTag> {
    type Output = Self;

    fn decode<R: Read>(reader: &mut R) -> Result<Self::Output, DecodeError> {
        let length = reader.read_var_i32()? as usize;
        let mut vec = Vec::with_capacity(length);

        for _ in 0..length {
            let compound_tag = reader.read_compound_tag()?;
            vec.push(compound_tag);
        }

        Ok(vec)
    }
}

mod var_int {
    use crate::decoder::DecoderReadExt;
    use crate::error::DecodeError;
    use std::io::Read;

    pub fn decode<R: Read>(reader: &mut R) -> Result<i32, DecodeError> {
        Ok(reader.read_var_i32()?)
    }
}

mod var_long {
    use crate::decoder::DecoderReadExt;
    use crate::error::DecodeError;
    use std::io::Read;

    pub fn decode<R: Read>(reader: &mut R) -> Result<i64, DecodeError> {
        Ok(reader.read_var_i64()?)
    }
}

mod rest {
    use crate::error::DecodeError;
    use std::io::Read;

    pub fn decode<R: Read>(reader: &mut R) -> Result<Vec<u8>, DecodeError> {
        let mut data = Vec::new();
        reader.read_to_end(data.as_mut())?;

        Ok(data)
    }
}

mod uuid_hyp_str {
    use crate::decoder::DecoderReadExt;
    use crate::error::DecodeError;
    use std::io::Read;
    use uuid::Uuid;

    pub fn decode<R: Read>(reader: &mut R) -> Result<Uuid, DecodeError> {
        let uuid_hyphenated_string = reader.read_string(36)?;
        let uuid = Uuid::parse_str(&uuid_hyphenated_string)?;

        Ok(uuid)
    }
}

#[cfg(test)]
mod tests {
    use crate::decoder::DecoderReadExt;
    use std::io::Cursor;

    #[test]
    fn test_read_variable_i32_2_bytes_value() {
        let mut cursor = Cursor::new(vec![0b10101100, 0b00000010]);
        let value = cursor.read_var_i32().unwrap();

        assert_eq!(value, 300);
    }

    #[test]
    fn test_read_variable_i32_5_bytes_value() {
        let mut cursor = Cursor::new(vec![0xff, 0xff, 0xff, 0xff, 0x07]);
        let value = cursor.read_var_i32().unwrap();

        assert_eq!(value, 2147483647);
    }
}
