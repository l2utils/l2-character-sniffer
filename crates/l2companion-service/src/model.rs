//! Character and Account domain models and telemetry data structures.

use async_graphql::SimpleObject;
use serde::{Deserialize, Serialize};
pub use l2companion_protocol::{
    BuffEffect, CharSelectSlot, CommissionItem, EinhasadProduct, ItemInfo as InventoryItem,
    PrivateStoreItem, PrivateStoreType, SkillEntry, WarehouseType, WorldExchangeItem,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, SimpleObject)]
pub struct Location {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, SimpleObject)]
pub struct Vitals {
    pub cur_hp: u32,
    pub max_hp: u32,
    pub cur_mp: u32,
    pub max_mp: u32,
    pub cur_cp: u32,
    pub max_cp: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, SimpleObject)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, SimpleObject)]
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
    pub skills: Vec<SkillEntry>,
    pub buffs: Vec<BuffEffect>,
    pub inventory: Vec<InventoryItem>,
    pub warehouse: Vec<InventoryItem>,
    pub client_addr: Option<String>,
    pub last_updated_epoch_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, SimpleObject)]
pub struct AccountSession {
    pub account_name: String,
    pub client_addr: String,
    pub character_roster: Vec<CharSelectSlot>,
    pub active_character: Option<String>,
    pub last_seen_epoch_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, SimpleObject)]
pub struct PrivateStoreSession {
    pub seller_object_id: u32,
    pub seller_name: Option<String>,
    pub store_type: PrivateStoreType,
    pub store_title: String,
    pub items: Vec<PrivateStoreItem>,
    pub last_seen_epoch_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, SimpleObject)]
pub struct MarketState {
    pub private_stores: Vec<PrivateStoreSession>,
    pub commission_items: Vec<CommissionItem>,
    pub world_exchange_items: Vec<WorldExchangeItem>,
    pub einhasad_products: Vec<EinhasadProduct>,
}
