//! Just the packets that this `sde` supports.

use singularity_macros::{Packet, PacketUnion};
use singularity_sap::{
    datable::{ToData, TryFromData},
    packet::{IdType, PacketTrait},
    standard_packets::display_packets::{DisplayEvent, RequestChangeName, RequestUpdateWindow},
};

pub type SDEEvent = DisplayEvent;

#[derive(Debug, PacketUnion, Packet)]
pub enum SDERequest {
    RequestChangeName(RequestChangeName),
    RequestUpdateWindow(RequestUpdateWindow),
}
