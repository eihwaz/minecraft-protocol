mod data;

use crate::data::input;
use handlebars::*;
use heck::{CamelCase, KebabCase, MixedCase, SnakeCase, TitleCase};

use crate::data::input::{Container, Data, ProtocolData, ProtocolState};
use crate::data::output;
use crate::data::output::{Bound, Packet, State};
use linked_hash_map::LinkedHashMap;
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
            transform_protocol_state(State::Handshake, &protocol_input.handshaking),
            State::Handshake,
        ),
        (
            transform_protocol_state(State::Status, &protocol_input.status),
            State::Status,
        ),
        (
            transform_protocol_state(State::Login, &protocol_input.login),
            State::Login,
        ),
        (
            transform_protocol_state(State::Game, &protocol_input.game),
            State::Game,
        ),
    ];

    for (protocol, state) in protocols {
        let file_name = format!("{}.rs", state.to_string().to_lowercase());
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

fn transform_protocol_state(state: State, protocol_state: &ProtocolState) -> output::Protocol {
    let server_bound_packets = transform_protocol_data(&protocol_state.to_server);
    let client_bound_packets = transform_protocol_data(&protocol_state.to_client);

    output::Protocol {
        state,
        server_bound_packets,
        client_bound_packets,
    }
}

fn transform_protocol_data(protocol_data: &ProtocolData) -> Vec<Packet> {
    let mut packets = vec![];

    let reversed_packet_ids = protocol_data
        .types
        .get("packet")
        .and_then(|d| d.get(1))
        .and_then(|d| match d {
            Data::Container(data) => data.get(0),
            _ => None,
        })
        .and_then(|c| match c {
            Container::List { data, .. } => data.get(1),
            _ => None,
        })
        .and_then(|d| match d {
            Data::Mapper { mappings, .. } => Some(mappings),
            _ => None,
        })
        .expect("Failed to get packet ids");

    let packet_ids: HashMap<String, u8> = reversed_packet_ids
        .into_iter()
        .map(|(k, v)| {
            (
                v.clone(),
                u8::from_str_radix(k.trim_start_matches("0x"), 16).expect("Invalid packet id"),
            )
        })
        .collect();

    for (unformatted_name, data) in protocol_data.types.iter() {
        if !unformatted_name.starts_with("packet_")
            || unformatted_name == "packet_legacy_server_list_ping"
        {
            continue;
        }

        let no_prefix_name = unformatted_name.trim_start_matches("packet_");

        let id = *packet_ids
            .get(no_prefix_name)
            .expect("Failed to get packet id");
        let name = no_prefix_name.to_camel_case();

        let packet = Packet {
            id,
            name,
            fields: vec![],
        };

        packets.push(packet);
    }

    packets
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
        packet_enum_name: format!("{}{}BoundPacket", &protocol.state, Bound::Server),
        packets: &protocol.server_bound_packets,
    };

    let client_bound_ctx = GenerateContext {
        packet_enum_name: format!("{}{}BoundPacket", &protocol.state, Bound::Client),
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
