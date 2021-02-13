//! This crate implements Minecraft protocol.
//!
//! Information about protocol can be found at https://wiki.vg/Protocol.
#[cfg(feature = "data")]
pub mod data;
pub mod decoder;
pub mod encoder;
pub mod error;
