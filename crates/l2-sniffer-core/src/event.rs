//! Character and Account state update events.

use std::net::SocketAddr;
use serde::{Deserialize, Serialize};
use crate::model::{CharSelectSlot, Character, Location, Vitals};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "payload")]
pub enum SnifferEvent {
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
    SkillsUpdated {
        client_addr: Option<SocketAddr>,
        object_id: u32,
        skills: Vec<crate::model::SkillEntry>,
    },
    BuffsUpdated {
        client_addr: Option<SocketAddr>,
        object_id: u32,
        buffs: Vec<crate::model::BuffEffect>,
    },
    InventoryLoaded {
        client_addr: Option<SocketAddr>,
        object_id: u32,
        items: Vec<crate::model::InventoryItem>,
    },
    WarehouseLoaded {
        client_addr: Option<SocketAddr>,
        object_id: u32,
        wh_type: crate::model::WarehouseType,
        player_adena: u64,
        items: Vec<crate::model::InventoryItem>,
    },
    PrivateStoreUpdated {
        client_addr: Option<SocketAddr>,
        store: crate::model::PrivateStoreSession,
    },
    CommissionMarketUpdated {
        items: Vec<crate::model::CommissionItem>,
    },
    WorldExchangeUpdated {
        items: Vec<crate::model::WorldExchangeItem>,
    },
    EinhasadStoreUpdated {
        products: Vec<crate::model::EinhasadProduct>,
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
