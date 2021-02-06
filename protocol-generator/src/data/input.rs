use linked_hash_map::LinkedHashMap;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Protocol {
    pub handshaking: ProtocolState,
    pub status: ProtocolState,
    pub login: ProtocolState,
    #[serde(rename = "play")]
    pub game: ProtocolState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolState {
    pub to_client: ProtocolData,
    pub to_server: ProtocolData,
}

#[derive(Debug, Deserialize)]
pub struct ProtocolData {
    pub types: LinkedHashMap<String, Vec<Data>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Data {
    Type(String),
    Container(Vec<Container>),
    Mapper {
        #[serde(rename = "type")]
        mappings_type: String,
        mappings: LinkedHashMap<String, String>,
    },
    Switch(Switch),
    List(Box<List>),
    Bitfield(Vec<BitField>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Container {
    Value {
        name: String,
        #[serde(rename = "type")]
        data: Data,
    },
    List {
        name: Option<String>,
        #[serde(rename = "type")]
        data: Vec<Data>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Switch {
    Empty {
        #[serde(rename = "compareTo")]
        compare_to: String,
    },
    Value {
        #[serde(rename = "compareTo")]
        compare_to: String,
        fields: LinkedHashMap<String, Data>,
    },
    List {
        #[serde(rename = "compareTo")]
        compare_to: String,
        fields: LinkedHashMap<String, Vec<Data>>,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum List {
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
    List {
        #[serde(rename = "countType")]
        count_type: String,
        #[serde(rename = "type")]
        list_type: Vec<Data>,
    },
}

#[derive(Debug, Deserialize)]
pub struct BitField {
    name: String,
    size: usize,
    signed: bool,
}
