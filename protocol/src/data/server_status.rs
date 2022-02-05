use crate::data::chat::Message;
use crate::impl_json_encoder_decoder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ServerStatus {
    pub version: ServerVersion,
    pub players: OnlinePlayers,
    pub description: Message,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub favicon: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ServerVersion {
    pub name: String,
    pub protocol: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OnlinePlayers {
    pub max: u32,
    pub online: u32,
    #[serde(default)]
    pub sample: Vec<OnlinePlayer>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct OnlinePlayer {
    pub name: String,
    pub id: Uuid,
}

impl_json_encoder_decoder!(ServerStatus);
