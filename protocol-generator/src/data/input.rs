use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Protocol {
    handshaking: ProtocolState,
    status: ProtocolState,
    login: ProtocolState,
    #[serde(rename = "play")]
    game: ProtocolState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolState {
    to_client: ProtocolData,
    to_server: ProtocolData,
}

#[derive(Debug, Deserialize)]
struct ProtocolData {
    types: HashMap<String, Vec<Data>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Data {
    Type(String),
    Container(Vec<Container>),
    Mapper {
        #[serde(rename = "type")]
        mappings_type: String,
        mappings: HashMap<String, String>,
    },
    Switch(Switch),
    List(Box<List>),
    Bitfield(Vec<BitField>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Container {
    Value {
        name: String,
        #[serde(rename = "type")]
        data: Data,
    },
    Array {
        name: Option<String>,
        #[serde(rename = "type")]
        data: Vec<Data>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Switch {
    Empty {
        #[serde(rename = "compareTo")]
        compare_to: String,
    },
    Value {
        #[serde(rename = "compareTo")]
        compare_to: String,
        fields: HashMap<String, Data>,
    },
    List {
        #[serde(rename = "compareTo")]
        compare_to: String,
        fields: HashMap<String, Vec<Data>>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum List {
    Empty {
        #[serde(rename = "countType")]
        count_type: String,
    },
    Value {
        #[serde(rename = "countType")]
        count_type: String,
        #[serde(rename = "type")]
        list_type: Data,
    },
    Array {
        #[serde(rename = "countType")]
        count_type: String,
        #[serde(rename = "type")]
        list_type: Vec<Data>,
    },
}

#[derive(Debug, Deserialize)]
struct BitField {
    name: String,
    size: usize,
    signed: bool,
}
