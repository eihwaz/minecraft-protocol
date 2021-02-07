mod data;

use crate::data::input;
use handlebars::*;
use heck::{CamelCase, SnakeCase};

use crate::data::input::{Container, Data, ProtocolData, ProtocolState};
use crate::data::output;
use crate::data::output::Field;
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "protocol-generator")]
struct Opt {
    #[structopt(short, long, default_value = "1.14.4")]
    protocol_version: String,
}

pub fn main() {
    let opt: Opt = Opt::from_args();
    let template_engine = create_template_engine();

    let protocol_data_file_name = format!(
        "protocol-generator/minecraft-data/data/pc/{}/protocol.json",
        opt.protocol_version
    );

    let protocol_data_file =
        File::open(protocol_data_file_name).expect("Failed to open protocol data file");

    let protocol_input: input::Protocol =
        serde_json::from_reader(protocol_data_file).expect("Failed to parse protocol data");

    let protocols = vec![
        (
            transform_protocol_state(output::State::Handshake, &protocol_input.handshaking),
            output::State::Handshake,
        ),
        (
            transform_protocol_state(output::State::Status, &protocol_input.status),
            output::State::Status,
        ),
        (
            transform_protocol_state(output::State::Login, &protocol_input.login),
            output::State::Login,
        ),
        // (
        //     transform_protocol_state(State::Game, &protocol_input.game),
        //     State::Game,
        // ),
    ];

    for (protocol, state) in protocols {
        let file_name = format!(
            "protocol/src/packet/{}.rs",
            state.to_string().to_lowercase()
        );
        let file = File::create(file_name).expect("Failed to create file");

        generate_rust_file(&protocol, &template_engine, &file)
            .expect("Failed to generate rust file");
    }
}

fn create_template_engine() -> Handlebars<'static> {
    let mut template_engine = Handlebars::new();

    template_engine.register_helper("snake_case", Box::new(format_snake_case));
    template_engine.register_helper("packet_id", Box::new(format_packet_id));
    template_engine.register_escape_fn(|s| s.to_owned());

    template_engine
        .register_template_file(
            "packet_imports",
            "protocol-generator/templates/packet_imports.hbs",
        )
        .expect("Failed to register template");

    template_engine
        .register_template_file(
            "packet_enum",
            "protocol-generator/templates/packet_enum.hbs",
        )
        .expect("Failed to register template");

    template_engine
        .register_template_file(
            "packet_structs",
            "protocol-generator/templates/packet_structs.hbs",
        )
        .expect("Failed to register template");

    template_engine
}

fn transform_protocol_state(
    state: output::State,
    protocol_state: &ProtocolState,
) -> output::Protocol {
    let server_bound_packets =
        transform_protocol_data(&protocol_state.to_server, output::Bound::Server);
    let client_bound_packets =
        transform_protocol_data(&protocol_state.to_client, output::Bound::Client);

    output::Protocol {
        state,
        server_bound_packets,
        client_bound_packets,
    }
}

fn transform_protocol_data(
    protocol_data: &ProtocolData,
    bound: output::Bound,
) -> Vec<output::Packet> {
    let packet_ids = get_packet_ids(protocol_data);
    let mut packets = vec![];

    for (unformatted_name, data_vec) in protocol_data.types.iter() {
        if !unformatted_name.starts_with("packet_")
            || unformatted_name == "packet_legacy_server_list_ping"
        {
            continue;
        }

        let no_prefix_name = unformatted_name.trim_start_matches("packet_");

        let id = *packet_ids
            .get(no_prefix_name)
            .expect("Failed to get packet id");
        let packet_name = rename_packet(&no_prefix_name.to_camel_case(), &bound);

        let mut fields = vec![];

        for data in data_vec {
            if let Data::Container(container_vec) = data {
                for container in container_vec {
                    match container {
                        Container::Value { name, data } => {
                            if let Some(field) = transform_field(name, data) {
                                fields.push(modify_field(&packet_name, field));
                            }
                        }
                        Container::List { name, data_vec } => {
                            if let Some(name) = name {
                                for data in data_vec {
                                    if let Some(field) = transform_field(name, data) {
                                        fields.push(modify_field(&packet_name, field));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let packet = output::Packet {
            id,
            name: packet_name,
            fields,
        };

        packets.push(packet);
    }

    packets
}

fn get_packet_ids(protocol_data: &ProtocolData) -> HashMap<String, u8> {
    let reversed_packet_ids = protocol_data
        .types
        .get("packet")
        .and_then(|d| d.get(1))
        .and_then(|d| match d {
            Data::Container(data) => data.get(0),
            _ => None,
        })
        .and_then(|c| match c {
            Container::List { data_vec, .. } => data_vec.get(1),
            _ => None,
        })
        .and_then(|d| match d {
            Data::Mapper { mappings, .. } => Some(mappings),
            _ => None,
        })
        .expect("Failed to get packet ids");

    reversed_packet_ids
        .into_iter()
        .map(|(k, v)| {
            (
                v.clone(),
                u8::from_str_radix(k.trim_start_matches("0x"), 16).expect("Invalid packet id"),
            )
        })
        .collect()
}

fn transform_field(unformatted_field_name: &str, data: &Data) -> Option<output::Field> {
    match data {
        Data::Type(str_type) => match transform_data_type(str_type) {
            Some(data_type) => Some(output::Field {
                name: format_field_name(unformatted_field_name),
                data_type,
            }),
            None => None,
        },
        _ => None,
    }
}

fn format_field_name(unformatted_field_name: &str) -> String {
    if unformatted_field_name == "Type" {
        String::from("type_")
    } else {
        unformatted_field_name.to_snake_case()
    }
}

fn transform_data_type(name: &str) -> Option<output::DataType> {
    match name {
        "bool" => Some(output::DataType::Boolean),
        "i8" => Some(output::DataType::Byte),
        "i16" => Some(output::DataType::Short),
        "i32" => Some(output::DataType::Int { var_int: false }),
        "i64" => Some(output::DataType::Long { var_long: false }),
        "u8" => Some(output::DataType::UnsignedByte),
        "u16" => Some(output::DataType::UnsignedShort),
        "f32" => Some(output::DataType::Float),
        "f64" => Some(output::DataType::Double),
        "varint" => Some(output::DataType::Int { var_int: true }),
        "varlong" => Some(output::DataType::Long { var_long: true }),
        "string" => Some(output::DataType::String { max_length: 0 }),
        "nbt" | "optionalNbt" => Some(output::DataType::CompoundTag),
        "UUID" => Some(output::DataType::Uuid { hyphenated: false }),
        "buffer" => Some(output::DataType::ByteArray { rest: false }),
        "restBuffer" => Some(output::DataType::ByteArray { rest: true }),
        _ => {
            println!("Unknown data type \"{}\"", name);
            None
        }
    }
}

fn modify_field(packet_name: &str, field: output::Field) -> output::Field {
    match (packet_name, field.name.as_str()) {
        ("StatusResponse", "response") => field.change_type(output::DataType::RefType {
            ref_name: "ServerStatus".to_owned(),
        }),
        _ => field,
    }
}

fn rename_packet(name: &str, bound: &output::Bound) -> String {
    match (name, bound) {
        ("EncryptionBegin", output::Bound::Server) => "EncryptionResponse",
        ("EncryptionBegin", output::Bound::Client) => "EncryptionRequest",
        ("PingStart", output::Bound::Server) => "StatusRequest",
        ("Ping", output::Bound::Server) => "PingRequest",
        ("ServerInfo", output::Bound::Client) => "StatusResponse",
        ("Ping", output::Bound::Client) => "PingResponse",
        _ => name,
    }
    .to_owned()
}

#[derive(Serialize)]
struct GenerateContext<'a> {
    packet_enum_name: String,
    packets: &'a Vec<output::Packet>,
}

fn generate_rust_file<W: Write>(
    protocol: &output::Protocol,
    template_engine: &Handlebars,
    mut writer: W,
) -> Result<(), TemplateRenderError> {
    let server_bound_ctx = GenerateContext {
        packet_enum_name: format!("{}{}BoundPacket", &protocol.state, output::Bound::Server),
        packets: &protocol.server_bound_packets,
    };

    let client_bound_ctx = GenerateContext {
        packet_enum_name: format!("{}{}BoundPacket", &protocol.state, output::Bound::Client),
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

fn format_snake_case(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let str = h
        .param(0)
        .and_then(|v| v.value().as_str())
        .ok_or(RenderError::new(
            "Param 0 with str type is required for snake case helper.",
        ))? as &str;

    let snake_case_str = str.to_snake_case();

    out.write(snake_case_str.as_ref())?;
    Ok(())
}

fn format_packet_id(
    h: &Helper,
    _: &Handlebars,
    _: &Context,
    _: &mut RenderContext,
    out: &mut dyn Output,
) -> Result<(), RenderError> {
    let id = h
        .param(0)
        .and_then(|v| v.value().as_u64())
        .ok_or(RenderError::new(
            "Param 0 with u64 type is required for packet id helper.",
        ))? as u64;

    let packet_id_str = format!("{:#04X}", id);

    out.write(packet_id_str.as_ref())?;
    Ok(())
}
