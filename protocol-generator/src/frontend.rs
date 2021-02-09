use crate::frontend;
use handlebars::{Handlebars, TemplateRenderError};
use serde::Serialize;
use serde_json::json;
use std::collections::HashSet;
use std::fmt;
use std::fmt::Display;
use std::io::Write;

#[derive(Debug, Copy, Clone)]
pub enum State {
    Handshake,
    Status,
    Login,
    Game,
}

impl State {
    pub fn data_import(&self) -> &str {
        match self {
            State::Handshake => "crate::data::handshake::*",
            State::Status => "crate::data::status::*",
            State::Login => "crate::data::login::*",
            State::Game => "crate::data::game::*",
        }
    }
}

impl Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            State::Handshake => "Handshake",
            State::Status => "Status",
            State::Login => "Login",
            State::Game => "Game",
        };

        write!(f, "{}", name)
    }
}

pub enum Bound {
    Server,
    Client,
}

impl Display for Bound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Bound::Server => "Server",
            Bound::Client => "Client",
        };

        write!(f, "{}", name)
    }
}

#[derive(Serialize, Debug)]
pub struct Packet {
    pub id: u8,
    pub name: String,
    pub fields: Vec<Field>,
}

impl Packet {
    pub fn new(id: u8, name: impl ToString, fields: Vec<Field>) -> Packet {
        Packet {
            id,
            name: name.to_string(),
            fields,
        }
    }
}

#[derive(Serialize, Debug)]
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

    pub fn change_type(&self, data_type: DataType) -> Field {
        Field::new(&self.name, data_type)
    }
}

#[derive(Serialize, Eq, PartialEq, Debug)]
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
    #[serde(rename(serialize = "i64"))]
    Long {
        var_long: bool,
    },
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
    #[serde(rename(serialize = "Message"))]
    Chat,
}

impl DataType {
    pub fn import<'a>(&self, state: &'a State) -> Option<&'a str> {
        match self {
            DataType::Uuid { .. } => Some("uuid::Uuid"),
            DataType::CompoundTag => Some("nbt::CompoundTag"),
            DataType::RefType { .. } => Some(state.data_import()),
            DataType::Chat => Some("crate::data::chat::Message"),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Protocol {
    pub state: State,
    pub server_bound_packets: Vec<Packet>,
    pub client_bound_packets: Vec<Packet>,
}

impl Protocol {
    pub fn new(
        state: State,
        server_bound_packets: Vec<Packet>,
        client_bound_packets: Vec<Packet>,
    ) -> Protocol {
        Protocol {
            state,
            server_bound_packets,
            client_bound_packets,
        }
    }

    pub fn data_type_imports(&self) -> HashSet<&str> {
        self.server_bound_packets
            .iter()
            .chain(self.client_bound_packets.iter())
            .flat_map(|p| p.fields.iter())
            .filter_map(|f| f.data_type.import(&self.state))
            .collect()
    }
}

#[derive(Serialize)]
struct GenerateContext<'a> {
    packet_enum_name: String,
    packets: &'a Vec<frontend::Packet>,
}

pub fn generate_rust_file<W: Write>(
    protocol: &frontend::Protocol,
    template_engine: &Handlebars,
    mut writer: W,
) -> Result<(), TemplateRenderError> {
    let server_bound_ctx = GenerateContext {
        packet_enum_name: format!("{}{}BoundPacket", &protocol.state, frontend::Bound::Server),
        packets: &protocol.server_bound_packets,
    };

    let client_bound_ctx = GenerateContext {
        packet_enum_name: format!("{}{}BoundPacket", &protocol.state, frontend::Bound::Client),
        packets: &protocol.client_bound_packets,
    };

    let mut imports = vec![
        "crate::DecodeError",
        "crate::Decoder",
        "std::io::Read",
        "minecraft_protocol_derive::Packet",
    ];

    imports.extend(protocol.data_type_imports().iter());

    template_engine.render_to_write(
        "packet_imports",
        &json!({ "imports": imports }),
        &mut writer,
    )?;

    template_engine.render_to_write("packet_enum", &server_bound_ctx, &mut writer)?;
    template_engine.render_to_write("packet_enum", &client_bound_ctx, &mut writer)?;

    template_engine.render_to_write("packet_structs", &server_bound_ctx, &mut writer)?;
    template_engine.render_to_write("packet_structs", &client_bound_ctx, &mut writer)?;

    Ok(())
}
