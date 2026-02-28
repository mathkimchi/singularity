//! Just the packets that this `sde` supports.

use singularity_common::sap::packets::{StandardEvent, StandardRequest};

pub type SDEEvent = StandardEvent;
pub type SDERequest = StandardRequest;
