use handlebars::{Handlebars, TemplateRenderError};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufWriter, Read};

enum State {
    Handshake,
    Status,
    Login,
    Game,
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

enum Bound {
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

#[derive(Serialize)]
struct Packet {
    name: String,
    fields: Vec<Field>,
}

impl Packet {
    pub fn new(name: impl ToString, fields: Vec<Field>) -> Packet {
        Packet {
            name: name.to_string(),
            fields,
        }
    }
}

#[derive(Serialize)]
struct Field {
    name: String,
    #[serde(rename(serialize = "type"))]
    data_type: DataType,
}

impl Field {
    pub fn new(name: impl ToString, data_type: DataType) -> Field {
        Field {
            name: name.to_string(),
            data_type,
        }
    }
}

#[derive(Serialize)]
enum DataType {
    Boolean,
    Byte,
    UnsignedByte,
    Short,
    UnsignedShort,
    Int,
    Long,
    Float,
    Double,
    String,
    Chat,
    VarInt,
    VarLong,
    ByteArray,
}

struct Protocol {
    state: State,
    server_bound_packets: Vec<Packet>,
    client_bound_packets: Vec<Packet>,
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

    pub fn generate_rust_file(
        &self,
        template_engine: &Handlebars,
        writer: &mut BufWriter<&File>,
    ) -> Result<(), TemplateRenderError> {
        write_protocol_enum(
            writer,
            &template_engine,
            &self.server_bound_packets,
            &Bound::Server,
            &self.state,
        )?;

        write_protocol_enum(
            writer,
            &template_engine,
            &self.client_bound_packets,
            &Bound::Client,
            &self.state,
        )?;

        Ok(())
    }
}

fn write_protocol_enum(
    writer: &mut BufWriter<&File>,
    template_engine: &Handlebars,
    packets: &Vec<Packet>,
    bound: &Bound,
    state: &State,
) -> Result<(), TemplateRenderError> {
    if !packets.is_empty() {
        let enum_name = format!("{}{}BoundPacket", state, bound);

        let data = json!({
          "protocol_state_name": enum_name,
          "packets": &packets
        });

        template_engine.render_to_write("protocol_state_enum", &data, writer)?;
    }

    Ok(())
}

pub fn main() {
    let mut template_engine = Handlebars::new();

    template_engine
        .register_template_file(
            "protocol_state_enum",
            "protocol-generator/templates/protocol_state_enum.hbs",
        )
        .expect("Failed to register template");

    let protocol = Protocol::new(
        State::Login,
        vec![
            Packet::new("LoginStart", vec![Field::new("name", DataType::String)]),
            Packet::new(
                "EncryptionResponse",
                vec![
                    Field::new("shared_secret", DataType::ByteArray),
                    Field::new("verify_token", DataType::ByteArray),
                ],
            ),
            Packet::new("LoginPluginResponse", vec![]),
        ],
        vec![
            Packet::new("LoginDisconnect", vec![]),
            Packet::new("EncryptionRequest", vec![]),
            Packet::new("LoginSuccess", vec![]),
            Packet::new("SetCompression", vec![]),
            Packet::new("LoginPluginRequest", vec![]),
        ],
    );

    let file = File::create("login.rs").expect("Failed to create file");
    let mut writer = BufWriter::new(&file);

    protocol
        .generate_rust_file(&template_engine, &mut writer)
        .expect("Failed to generate rust file");
}
