mod data;

use crate::data::*;
use handlebars::*;
use heck::SnakeCase;
use serde::Serialize;
use serde_json::json;
use std::fs::File;
use std::io::Write;

pub fn main() {
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

    let protocol = Protocol::new(
        State::Login,
        vec![
            Packet::new(
                "LoginStart",
                vec![Field::new("name", DataType::String { max_length: 256 })],
            ),
            Packet::new(
                "EncryptionResponse",
                vec![
                    Field::new("shared_secret", DataType::ByteArray { rest: true }),
                    Field::new("verify_token", DataType::ByteArray { rest: true }),
                ],
            ),
            Packet::new(
                "LoginPluginResponse",
                vec![
                    Field::new("message_id", DataType::Int { var_int: true }),
                    Field::new("successful", DataType::Boolean),
                    Field::new("data", DataType::ByteArray { rest: true }),
                ],
            ),
        ],
        vec![
            Packet::new(
                "LoginDisconnect",
                vec![
                    Field::new("hyphenated", DataType::Uuid { hyphenated: true }),
                    Field::new("default", DataType::Uuid { hyphenated: false }),
                ],
            ),
            Packet::new(
                "EncryptionRequest",
                vec![Field::new(
                    "game_mode",
                    DataType::RefType {
                        ref_name: "GameMode".to_string(),
                    },
                )],
            ),
            Packet::new(
                "LoginSuccess",
                vec![Field::new(
                    "server_status",
                    DataType::RefType {
                        ref_name: "ServerStatus".to_string(),
                    },
                )],
            ),
            Packet::new("SetCompression", vec![]),
            Packet::new("LoginPluginRequest", vec![]),
        ],
    );

    let file = File::create("login.rs").expect("Failed to create file");

    generate_rust_file(&protocol, &template_engine, &file).expect("Failed to generate rust file");
}

#[derive(Serialize)]
struct GenerateContext<'a> {
    packet_enum_name: String,
    packets: &'a Vec<Packet>,
}

pub fn generate_rust_file<W: Write>(
    protocol: &Protocol,
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
