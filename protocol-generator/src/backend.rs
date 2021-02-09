use linked_hash_map::LinkedHashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ProtocolHandler {
    pub handshaking: Protocol,
    pub status: Protocol,
    pub login: Protocol,
    #[serde(rename = "play")]
    pub game: Protocol,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Protocol {
    pub to_client: Packets,
    pub to_server: Packets,
}

#[derive(Debug, Deserialize)]
pub struct Packets {
    pub types: LinkedHashMap<String, Vec<Data>>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Data {
    Type(String),
    Containers(Vec<Container>),
    Container(Box<Container>),
    Mapper {
        #[serde(rename = "type")]
        mappings_type: String,
        mappings: LinkedHashMap<String, String>,
    },
    Switch(Switch),
    List(Box<List>),
    Bitfield(Vec<BitField>),
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
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
        data_vec: Vec<Data>,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Switch {
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
    Empty {
        #[serde(rename = "compareTo")]
        compare_to: String,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum List {
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
    Empty {
        #[serde(rename = "countType")]
        count_type: String,
    },
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct BitField {
    name: String,
    size: usize,
    signed: bool,
}
