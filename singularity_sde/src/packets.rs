//! Just the packets that this `sde` supports.

use singularity_macros::{Datable, Packet};
use singularity_sap::{
    byte_stream::{ToData, TryFromData},
    packet::{IdType, PacketTrait},
    standard_packets::display_packets::{DisplayEvent, RequestChangeName, RequestUpdateWindow},
};

pub type SDEEvent = DisplayEvent;

#[derive(Debug, Datable, Packet)]
pub enum SDERequest {
    RequestChangeName(RequestChangeName),
    RequestUpdateWindow(RequestUpdateWindow),
}
