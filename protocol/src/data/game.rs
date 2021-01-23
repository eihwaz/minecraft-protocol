use crate::impl_enum_encoder_decoder;
use num_derive::{FromPrimitive, ToPrimitive};

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum MessagePosition {
    Chat,
    System,
    HotBar,
}

impl_enum_encoder_decoder!(MessagePosition);

#[derive(Debug, Eq, PartialEq, FromPrimitive, ToPrimitive)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    Spectator = 3,
    Hardcore = 8,
}

impl_enum_encoder_decoder!(GameMode);
