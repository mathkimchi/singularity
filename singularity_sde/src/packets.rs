//! Just the packets that this `sde` supports.

use singularity_macros::{Packet, PacketUnion, Request};
use singularity_sap::{
    datable::{ToData, TryFromData},
    packet::{PacketId, PacketTrait, RequestPacketTrait},
    standard_packets::display_packets::{
        DisplayEvent, RequestChangeName, RequestSpawnChildTab, RequestUpdateWindow,
    },
};

pub type SDEEvent = DisplayEvent;

#[derive(Debug, PacketUnion, Packet, Request)]
pub enum SDERequest {
    RequestChangeName(RequestChangeName),
    RequestUpdateWindow(RequestUpdateWindow),
    RequestSpawnChildTab(RequestSpawnChildTab),
}
