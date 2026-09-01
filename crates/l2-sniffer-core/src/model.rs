//! Character and Account domain models and telemetry data structures.

use serde::{Deserialize, Serialize};
pub use l2_sniffer_protocol::CharSelectSlot;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Vitals {
    pub cur_hp: u32,
    pub max_hp: u32,
    pub cur_mp: u32,
    pub max_mp: u32,
    pub cur_cp: u32,
    pub max_cp: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Stats {
    pub p_atk: u32,
    pub p_def: u32,
    pub m_atk: u32,
    pub m_def: u32,
    pub p_atk_spd: u32,
    pub m_atk_spd: u32,
    pub run_spd: u32,
    pub walk_spd: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct InventoryItem {
    pub object_id: u32,
    pub item_id: u32,
    pub count: u64,
    pub item_type: u16,
    pub equipped: bool,
    pub enchant_level: u16,
    pub is_augmented: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Character {
    pub object_id: u32,
    pub account_name: Option<String>,
    pub name: String,
    pub title: String,
    pub class_id: u32,
    pub level: u32,
    pub exp: u64,
    pub sp: u32,
    pub karma: u32,
    pub pk_kills: u32,
    pub pvp_kills: u32,
    pub location: Location,
    pub vitals: Vitals,
    pub stats: Stats,
    pub inventory: Vec<InventoryItem>,
    pub client_addr: Option<String>,
    pub last_updated_epoch_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AccountSession {
    pub account_name: String,
    pub client_addr: String,
    pub character_roster: Vec<CharSelectSlot>,
    pub active_character: Option<String>,
    pub last_seen_epoch_ms: u64,
}
