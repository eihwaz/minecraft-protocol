use crate::backend::Data;
use crate::mappings::Mappings;
use crate::{backend, frontend};
use heck::{CamelCase, SnakeCase};
use std::collections::HashMap;

pub fn transform_protocol<M: Mappings>(
    mappings: &M,
    state: frontend::State,
    protocol: &backend::Protocol,
) -> frontend::Protocol {
    let server_bound_packets = transform_packets(
        mappings,
        protocol,
        &protocol.to_server,
        frontend::Bound::Server,
    );

    let client_bound_packets = transform_packets(
        mappings,
        protocol,
        &protocol.to_client,
        frontend::Bound::Client,
    );

    frontend::Protocol {
        state,
        server_bound_packets,
        client_bound_packets,
    }
}

fn get_packet_ids(packets: &backend::Packets) -> HashMap<String, u8> {
    let reversed_packet_ids = packets
        .types
        .get("packet")
        .and_then(|d| d.get(1))
        .and_then(|d| match d {
            backend::Data::Container(data) => data.get(0),
            _ => None,
        })
        .and_then(|c| match c {
            backend::Container::List { data_vec, .. } => data_vec.get(1),
            _ => None,
        })
        .and_then(|d| match d {
            backend::Data::Mapper { mappings, .. } => Some(mappings),
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

fn transform_packets<M: Mappings>(
    mappings: &M,
    protocol: &backend::Protocol,
    packets: &backend::Packets,
    bound: frontend::Bound,
) -> Vec<frontend::Packet> {
    let packet_ids = get_packet_ids(packets);
    let mut output_packets = vec![];

    for (unformatted_name, data_vec) in packets.types.iter() {
        if !unformatted_name.starts_with("packet_")
            || unformatted_name == "packet_legacy_server_list_ping"
        {
            continue;
        }

        let no_prefix_unformatted = unformatted_name.trim_start_matches("packet_");

        let id = *packet_ids
            .get(no_prefix_unformatted)
            .expect("Failed to get packet id");

        let packet_name = mappings.rename_packet(
            unformatted_name,
            &no_prefix_unformatted.to_camel_case(),
            &bound,
            protocol,
        );

        let mut fields = vec![];

        for data in data_vec {
            if let backend::Data::Container(container_vec) = data {
                for container in container_vec {
                    match container {
                        backend::Container::Value { name, data } => {
                            match transform_value_field(&name, &data) {
                                Some(field) => {
                                    fields.push(mappings.change_field_type(&packet_name, field))
                                }
                                None => println!(
                                    "[{}] Field \"{}\" are skipped ({:?}",
                                    packet_name, name, data
                                ),
                            }
                        }
                        backend::Container::List { name, data_vec } => {
                            if let Some(name) = name {
                                match transform_list_field(&name, data_vec) {
                                    Some(field) => {
                                        fields.push(mappings.change_field_type(&packet_name, field))
                                    }
                                    None => {}
                                }
                            }
                        }
                    }
                }
            }
        }

        let packet = frontend::Packet {
            id,
            name: packet_name,
            fields,
        };

        output_packets.push(packet);
    }

    output_packets
}

fn transform_value_field(
    unformatted_field_name: &str,
    data: &backend::Data,
) -> Option<frontend::Field> {
    match data {
        backend::Data::Type(name) => match transform_data_type(name) {
            Some(data_type) => Some(frontend::Field {
                name: format_field_name(unformatted_field_name),
                data_type,
            }),
            None => None,
        },
        _ => None,
    }
}

fn transform_list_field(
    unformatted_field_name: &str,
    data_vec: &Vec<backend::Data>,
) -> Option<frontend::Field> {
    match &data_vec[0] {
        backend::Data::Type(name) => match name.as_ref() {
            "buffer" => Some(frontend::Field {
                name: format_field_name(unformatted_field_name),
                data_type: frontend::DataType::ByteArray { rest: false },
            }),
            "array" => None,
            "switch" => None,
            "particleData" => Some(frontend::Field {
                name: format_field_name(unformatted_field_name),
                data_type: frontend::DataType::RefType {
                    ref_name: "ParticleData".to_string(),
                },
            }),
            "option" => transform_value_field(unformatted_field_name, &data_vec[1]),
            _ => None,
        },
        _ => None,
    }
}

fn transform_data_type(name: &str) -> Option<frontend::DataType> {
    match name {
        "bool" => Some(frontend::DataType::Boolean),
        "i8" => Some(frontend::DataType::Byte),
        "i16" => Some(frontend::DataType::Short),
        "i32" => Some(frontend::DataType::Int { var_int: false }),
        "i64" => Some(frontend::DataType::Long { var_long: false }),
        "u8" => Some(frontend::DataType::UnsignedByte),
        "u16" => Some(frontend::DataType::UnsignedShort),
        "f32" => Some(frontend::DataType::Float),
        "f64" => Some(frontend::DataType::Double),
        "varint" => Some(frontend::DataType::Int { var_int: true }),
        "varlong" => Some(frontend::DataType::Long { var_long: true }),
        "string" => Some(frontend::DataType::String { max_length: 0 }),
        "nbt" | "optionalNbt" => Some(frontend::DataType::CompoundTag),
        "UUID" => Some(frontend::DataType::Uuid { hyphenated: false }),
        "restBuffer" => Some(frontend::DataType::ByteArray { rest: true }),
        "position" => Some(frontend::DataType::RefType {
            ref_name: "Position".to_string(),
        }),
        "slot" => Some(frontend::DataType::RefType {
            ref_name: "Option<Slot>".to_string(),
        }),
        "entityMetadata" => Some(frontend::DataType::RefType {
            ref_name: "Metadata".to_string(),
        }),
        "tags" => Some(frontend::DataType::RefType {
            ref_name: "TagsMap".to_string(),
        }),
        _ => {
            println!("Unknown data type \"{}\"", name);
            None
        }
    }
}

fn format_field_name(unformatted_field_name: &str) -> String {
    if unformatted_field_name == "type" {
        String::from("type_")
    } else {
        unformatted_field_name.to_snake_case()
    }
}
