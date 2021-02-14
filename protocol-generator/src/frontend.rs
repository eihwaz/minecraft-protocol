use crate::error::FrontendError;
use handlebars::Handlebars;
use serde::Serialize;
use std::io::Write;

#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum DataType {
    #[serde(rename(serialize = "bool"))]
    Boolean,
    #[serde(rename(serialize = "i8"))]
    Byte,
    #[serde(rename(serialize = "u8"))]
    UnsignedByte,
    #[serde(rename(serialize = "i16"))]
    Short,
    #[serde(rename(serialize = "u16"))]
    UnsignedShort,
    #[serde(rename(serialize = "i32"))]
    Int {
        var_int: bool,
    },
    #[serde(rename(serialize = "u32"))]
    UnsignedInt,
    #[serde(rename(serialize = "i64"))]
    Long {
        var_long: bool,
    },
    #[serde(rename(serialize = "u64"))]
    UnsignedLong,
    #[serde(rename(serialize = "f32"))]
    Float,
    #[serde(rename(serialize = "f64"))]
    Double,
    String {
        max_length: u16,
    },
    Uuid {
        hyphenated: bool,
    },
    #[serde(rename(serialize = "Vec<u8>"))]
    ByteArray {
        rest: bool,
    },
    CompoundTag,
    RefType {
        ref_name: String,
    },
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct PacketStruct {
    pub name: String,
    pub fields: Vec<Field>,
}

impl PacketStruct {
    pub fn new(name: impl ToString, fields: Vec<Field>) -> PacketStruct {
        PacketStruct {
            name: name.to_string(),
            fields,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct Field {
    pub name: String,
    #[serde(flatten)]
    pub data_type: DataType,
}

impl Field {
    pub fn new(name: impl ToString, data_type: DataType) -> Field {
        Field {
            name: name.to_string(),
            data_type,
        }
    }
}

fn write_packet_struct<W: Write>(
    template_engine: &Handlebars,
    packet_struct: PacketStruct,
    write: &mut W,
) -> Result<(), FrontendError> {
    template_engine.render_to_write("packet_struct", &packet_struct, write)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::frontend::{write_packet_struct, DataType, Field, PacketStruct};
    use crate::templates;

    #[test]
    fn test_write_packet_struct() {
        let template_engine = templates::create_template_engine("templates");

        let fields = vec![
            Field::new("boolean", DataType::Boolean),
            Field::new("byte", DataType::Byte),
            Field::new("unsigned_byte", DataType::UnsignedByte),
            Field::new("short", DataType::Short),
            Field::new("unsigned_short", DataType::UnsignedShort),
            Field::new("int", DataType::Int { var_int: false }),
            Field::new("varint", DataType::Int { var_int: true }),
            Field::new("unsigned_int", DataType::UnsignedInt),
            Field::new("long", DataType::Long { var_long: false }),
            Field::new("varlong", DataType::Long { var_long: true }),
            Field::new("unsigned_long", DataType::UnsignedLong),
            Field::new("float", DataType::Float),
            Field::new("double", DataType::Double),
            Field::new("string", DataType::String { max_length: 20 }),
            Field::new("uuid", DataType::Uuid { hyphenated: false }),
            Field::new("hyphenated", DataType::Uuid { hyphenated: true }),
            Field::new("byte_array", DataType::ByteArray { rest: false }),
            Field::new("rest", DataType::ByteArray { rest: true }),
            Field::new("compound_tag", DataType::CompoundTag),
            Field::new(
                "ref",
                DataType::RefType {
                    ref_name: "Chat".to_string(),
                },
            ),
        ];
        let packet_struct = PacketStruct::new("TestPacket", fields);
        let mut vec = vec![];

        write_packet_struct(&template_engine, packet_struct, &mut vec)
            .expect("Failed to write packet struct");

        let result = String::from_utf8(vec).expect("Failed to convert vec to string");

        assert_eq!(result, include_str!("../test/packet_struct.txt"));
    }
}
