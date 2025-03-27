//! Just the packets that this `sde` supports.

use singularity_macros::RequestPacketUnion;
use singularity_sap::{
    datable::{ToData, TryFromData},
    packet::{PacketTrait, PacketTypeId, PacketUnion, RequestPacketUnion},
    standard_packets::display_packets::{
        DisplayEvent, RequestChangeName, RequestSpawnChildTab, RequestUpdateWindow,
    },
};

pub type SDEEvent = DisplayEvent;

#[derive(Debug, RequestPacketUnion)]
pub enum SDERequest {
    RequestChangeName(RequestChangeName),
    RequestUpdateWindow(RequestUpdateWindow),
    RequestSpawnChildTab(RequestSpawnChildTab),
}
