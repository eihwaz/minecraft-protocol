use crate::impl_enum_encoder_decoder;
use nbt::CompoundTag;
use num_derive::{FromPrimitive, ToPrimitive};
use std::io::{Read, Write};

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

#[derive(Debug, Eq, PartialEq)]
pub struct Position {
    pub x: i32,
    pub y: i16,
    pub z: i32,
}

#[derive(Debug)]
pub struct Slot {
    pub id: i32,
    pub amount: u8,
    pub compound_tag: CompoundTag,
}

#[derive(Debug)]
pub struct Metadata {}

#[derive(Debug)]
pub struct TagsMap {}
