//! Character and Account state update events.

use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use crate::model::{CharSelectSlot, Character, Location, Vitals};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
pub enum CompanionEvent {
    ClientConnected {
        client_addr: SocketAddr,
        server_addr: SocketAddr,
    },
    ClientDisconnected {
        client_addr: SocketAddr,
        reason: String,
    },
    AccountDetected {
        client_addr: Option<SocketAddr>,
        account_name: String,
    },
    AccountRosterLoaded {
        client_addr: Option<SocketAddr>,
        account_name: String,
        characters: Vec<CharSelectSlot>,
    },
    CharacterLoaded {
        client_addr: Option<SocketAddr>,
        character: Character,
    },
    VitalsChanged {
        client_addr: Option<SocketAddr>,
        object_id: u32,
        vitals: Vitals,
    },
    LocationChanged {
        client_addr: Option<SocketAddr>,
        object_id: u32,
        location: Location,
    },
    ExpGained {
        client_addr: Option<SocketAddr>,
        object_id: u32,
        exp: u64,
        sp: u32,
    },
    ItemUpdated {
        client_addr: Option<SocketAddr>,
        object_id: u32,
        item_id: u32,
        count: u64,
    },
    RawPacketReceived {
        client_addr: Option<SocketAddr>,
        opcode: u8,
        length: usize,
    },
}
