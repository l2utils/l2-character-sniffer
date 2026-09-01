//! Character state update events.

use serde::{Deserialize, Serialize};
use crate::model::{Character, Location, Vitals};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
pub enum SnifferEvent {
    CharacterLoaded(Character),
    VitalsChanged {
        object_id: u32,
        vitals: Vitals,
    },
    LocationChanged {
        object_id: u32,
        location: Location,
    },
    ExpGained {
        object_id: u32,
        exp: u64,
        sp: u32,
    },
    ItemUpdated {
        object_id: u32,
        item_id: u32,
        count: u64,
    },
    RawPacketReceived {
        opcode: u8,
        length: usize,
    },
}
