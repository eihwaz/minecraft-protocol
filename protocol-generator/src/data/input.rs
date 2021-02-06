use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Protocol {
    handshaking: ProtocolState,
    status: ProtocolState,
    login: ProtocolState,
    #[serde(rename = "play")]
    game: ProtocolState,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolState {
    to_client: ProtocolTypes,
    to_server: ProtocolTypes,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProtocolTypes {
    types: HashMap<String, Vec<Container>>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum Container {
    Type(String),
    Data(Vec<ContainerData>),
}

#[derive(Debug, Serialize, Deserialize)]
struct ContainerData {
    name: String,
    #[serde(rename = "type")]
    container_type: ContainerDataType,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum ContainerDataType {
    Type(String),
    Mapper(String, Mappings),
    Switch(String, Switch),
}

#[derive(Debug, Serialize, Deserialize)]
struct Mappings {
    #[serde(rename = "type")]
    mappings_type: String,
    mappings: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Switch {
    #[serde(rename = "compareTo")]
    compare_to: String,
    fields: HashMap<String, String>,
}

#[cfg(test)]
mod tests {
    use crate::data::input::*;
    use std::collections::HashMap;

    #[test]
    fn test_to_json() {
        let mut types = HashMap::new();

        let data = vec![
            ContainerData {
                name: "protocolVersion".to_string(),
                container_type: ContainerDataType::Type("varint".to_string()),
            },
            ContainerData {
                name: "serverHost".to_string(),
                container_type: ContainerDataType::Type("string".to_string()),
            },
            ContainerData {
                name: "serverPort".to_string(),
                container_type: ContainerDataType::Type("u16".to_string()),
            },
            ContainerData {
                name: "nextState".to_string(),
                container_type: ContainerDataType::Type("varint".to_string()),
            },
        ];

        types.insert(
            "packet_set_protocol".to_owned(),
            vec![
                Container::Type("container".to_owned()),
                Container::Data(data),
            ],
        );

        let mut mappings = HashMap::new();
        mappings.insert("0x00".to_owned(), "set_protocol".to_owned());
        mappings.insert("0xfe".to_owned(), "legacy_server_list_ping".to_owned());

        let data2 = vec![ContainerData {
            name: "name".to_string(),
            container_type: ContainerDataType::Mapper(
                "mapper".to_owned(),
                Mappings {
                    mappings_type: "varint".to_string(),
                    mappings,
                },
            ),
        }];

        //             _type: ContainerDataType::Mapper(vec![
        //                 Mapper::Type("mapper".to_owned()),
        //                 Mapper::Data("varint".to_owned(), mappings),
        //             ]),

        types.insert(
            "packet".to_owned(),
            vec![
                Container::Type("container".to_owned()),
                Container::Data(data2),
            ],
        );

        // let protocol = Protocol {
        //     handshaking: ProtocolState {
        //         to_server: ProtocolTypes { types },
        //     },
        // };
        //
        // let j = serde_json::to_string(&protocol).unwrap();
        //
        // println!("{}", j);
        //
        // assert_eq!("", j);
    }
}
