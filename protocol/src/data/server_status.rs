use crate::data::chat::Message;
use crate::impl_json_encoder_decoder;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerStatus {
    pub version: ServerVersion,
    pub players: OnlinePlayers,
    pub description: Message,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ServerVersion {
    pub name: String,
    pub protocol: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OnlinePlayers {
    pub max: u32,
    pub online: u32,
    #[serde(default)]
    pub sample: Vec<OnlinePlayer>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct OnlinePlayer {
    pub name: String,
    pub id: Uuid,
}

impl_json_encoder_decoder!(ServerStatus);
