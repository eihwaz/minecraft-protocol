use crate::mappings::Mappings;
use crate::{backend, frontend, transformers};
use handlebars::{Handlebars, TemplateRenderError};
use serde::Serialize;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::fmt::Display;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::Path;

#[derive(Debug)]
pub enum State {
    Handshake,
    Status,
    Login,
    Game,
}

impl State {
    pub fn data_import(&self) -> &'static str {
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
    #[serde(rename(serialize = "u32"))]
    UnsignedInt,
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
    pub fn import(&self, state: &State) -> Option<&'static str> {
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
    pub server_bound_packets: Vec<Packet>,
    pub client_bound_packets: Vec<Packet>,
}

impl Protocol {
    pub fn new(server_bound_packets: Vec<Packet>, client_bound_packets: Vec<Packet>) -> Protocol {
        Protocol {
            server_bound_packets,
            client_bound_packets,
        }
    }

    pub fn data_type_imports(&self, state: &State) -> HashSet<&'static str> {
        self.server_bound_packets
            .iter()
            .chain(self.client_bound_packets.iter())
            .flat_map(|p| p.fields.iter())
            .filter_map(|f| f.data_type.import(state))
            .collect()
    }
}

pub fn generate_rust_files<M: Mappings>(
    versions_data: HashMap<String, File>,
    template_engine: &Handlebars,
    mappings: &M,
) -> Result<(), TemplateRenderError> {
    generate_versions_module_file(template_engine, versions_data.keys().cloned().collect())?;

    for (version, data_file) in versions_data.iter() {
        println!("Generating protocol data for version {}", version);

        let protocol_handler: backend::ProtocolHandler =
            serde_json::from_reader(data_file).expect("Failed to parse protocol data");

        let frontend_protocols =
            transformers::transform_protocol_handler(mappings, &protocol_handler);

        let formatted_version = version.replace(".", "_").replace("-", "_");

        let folder_name = format!("protocol/src/version/v_{}", formatted_version);
        let folder_path = Path::new(&folder_name);

        generate_protocol_module_file(template_engine, &folder_path)?;

        for (protocol, state) in frontend_protocols {
            let file_name = format!("{}.rs", state.to_string().to_lowercase());

            let mut file = File::create(folder_path.join(file_name))
                .expect("Failed to create protocol enum file");

            generate_protocol_enum_header(template_engine, &protocol, &state, &mut file)?;

            generate_protocol_enum_content(
                template_engine,
                &protocol.server_bound_packets,
                &state,
                &Bound::Server,
                &mut file,
            )?;

            generate_protocol_enum_content(
                template_engine,
                &protocol.client_bound_packets,
                &state,
                &Bound::Client,
                &mut file,
            )?;

            generate_packets_structs(template_engine, &protocol.server_bound_packets, &mut file)?;

            generate_packets_structs(template_engine, &protocol.client_bound_packets, &mut file)?;
        }
    }

    Ok(())
}

fn generate_versions_module_file(
    template_engine: &Handlebars,
    versions: Vec<String>,
) -> Result<(), TemplateRenderError> {
    let mut file =
        File::create("protocol/src/version/mod.rs").expect("Failed to create versions module file");

    let ctx = json!({ "versions": versions });

    template_engine.render_to_write("protocol_versions_module", &ctx, &mut file)?;

    Ok(())
}

fn generate_protocol_module_file(
    template_engine: &Handlebars,
    folder_path: &Path,
) -> Result<(), TemplateRenderError> {
    generate_module_file(template_engine, folder_path, "protocol_module")
}

fn generate_module_file(
    template_engine: &Handlebars,
    folder_path: &Path,
    name: &str,
) -> Result<(), TemplateRenderError> {
    create_dir_all(folder_path).expect("Failed to create module folder");

    let mut file = File::create(folder_path.join("mod.rs")).expect("Failed to create module file");

    template_engine.render_to_write(name, &(), &mut file)?;

    Ok(())
}

fn generate_protocol_enum_header<W: Write>(
    template_engine: &Handlebars,
    protocol: &frontend::Protocol,
    state: &frontend::State,
    write: &mut W,
) -> Result<(), TemplateRenderError> {
    let imports = protocol.data_type_imports(state);
    let ctx = json!({ "imports": imports });

    template_engine.render_to_write("protocol_header", &ctx, write)?;

    Ok(())
}

fn generate_protocol_enum_content<W: Write>(
    template_engine: &Handlebars,
    packets: &Vec<Packet>,
    state: &frontend::State,
    bound: &frontend::Bound,
    write: &mut W,
) -> Result<(), TemplateRenderError> {
    let protocol_enum_name = format!("{}Bound{}Packet", bound, state);
    let ctx = json!({ "protocol_enum_name": protocol_enum_name, "packets": packets });

    template_engine.render_to_write("protocol_enum", &ctx, write)?;

    Ok(())
}

fn generate_packets_structs<W: Write>(
    template_engine: &Handlebars,
    packets: &Vec<Packet>,
    write: &mut W,
) -> Result<(), TemplateRenderError> {
    let ctx = json!({ "packets": packets });

    template_engine.render_to_write("packets_structs", &ctx, write)?;

    Ok(())
}
