//! Character and Account domain models and telemetry data structures.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::Enum))]
pub enum WarehouseType {
    Private,
    Clan,
    Castle,
    Freight,
    Package,
}

impl Default for WarehouseType {
    fn default() -> Self {
        WarehouseType::Private
    }
}

impl From<l2_sniffer_protocol::WarehouseType> for WarehouseType {
    fn from(w: l2_sniffer_protocol::WarehouseType) -> Self {
        match w {
            l2_sniffer_protocol::WarehouseType::Private => WarehouseType::Private,
            l2_sniffer_protocol::WarehouseType::Clan => WarehouseType::Clan,
            l2_sniffer_protocol::WarehouseType::Castle => WarehouseType::Castle,
            l2_sniffer_protocol::WarehouseType::Freight => WarehouseType::Freight,
            l2_sniffer_protocol::WarehouseType::Package => WarehouseType::Package,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::Enum))]
pub enum PrivateStoreType {
    Sell,
    Buy,
    PackageSell,
    Manufacture,
}

impl Default for PrivateStoreType {
    fn default() -> Self {
        PrivateStoreType::Sell
    }
}

impl From<l2_sniffer_protocol::PrivateStoreType> for PrivateStoreType {
    fn from(p: l2_sniffer_protocol::PrivateStoreType) -> Self {
        match p {
            l2_sniffer_protocol::PrivateStoreType::Sell => PrivateStoreType::Sell,
            l2_sniffer_protocol::PrivateStoreType::Buy => PrivateStoreType::Buy,
            l2_sniffer_protocol::PrivateStoreType::PackageSell => PrivateStoreType::PackageSell,
            l2_sniffer_protocol::PrivateStoreType::Manufacture => PrivateStoreType::Manufacture,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct CharSelectSlot {
    pub name: String,
    pub title: String,
    pub char_id: u32,
    pub level: u32,
    pub class_id: u32,
    pub cur_hp: f64,
    pub max_hp: f64,
    pub cur_mp: f64,
    pub max_mp: f64,
    pub sp: u64,
    pub exp: u64,
    pub exp_percent: f64,
    pub reputation: i32,
    pub pk_kills: u32,
    pub pvp_kills: u32,
    pub vitality: u32,
    pub last_access: u32,
}

impl From<l2_sniffer_protocol::CharSelectSlot> for CharSelectSlot {
    fn from(s: l2_sniffer_protocol::CharSelectSlot) -> Self {
        Self {
            name: s.name,
            title: s.title,
            char_id: s.char_id,
            level: s.level,
            class_id: s.class_id,
            cur_hp: s.cur_hp,
            max_hp: s.max_hp,
            cur_mp: s.cur_mp,
            max_mp: s.max_mp,
            sp: s.sp,
            exp: s.exp,
            exp_percent: s.exp_percent,
            reputation: s.reputation,
            pk_kills: s.pk_kills,
            pvp_kills: s.pvp_kills,
            vitality: s.vitality,
            last_access: s.last_access,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct InventoryItem {
    pub object_id: u32,
    pub item_id: u32,
    pub count: u64,
    pub item_type: u16,
    pub item_type_name: String,
    pub equipped: bool,
    pub slot: u32,
    pub enchant_level: u16,
    pub custom_type1: u16,
    pub is_augmented: bool,
    pub mana: i32,
    pub durability: i32,
}

impl From<l2_sniffer_protocol::ItemInfo> for InventoryItem {
    fn from(i: l2_sniffer_protocol::ItemInfo) -> Self {
        Self {
            object_id: i.object_id,
            item_id: i.item_id,
            count: i.count,
            item_type: i.item_type,
            item_type_name: i.item_type_name,
            equipped: i.equipped,
            slot: i.slot,
            enchant_level: i.enchant_level,
            custom_type1: i.custom_type1,
            is_augmented: i.is_augmented,
            mana: i.mana,
            durability: i.durability,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct SkillEntry {
    pub skill_id: u32,
    pub level: u32,
    pub sub_level: u32,
    pub is_passive: bool,
    pub is_disabled: bool,
    pub enchant_type: u32,
}

impl From<l2_sniffer_protocol::SkillEntry> for SkillEntry {
    fn from(s: l2_sniffer_protocol::SkillEntry) -> Self {
        Self {
            skill_id: s.skill_id,
            level: s.level,
            sub_level: s.sub_level,
            is_passive: s.is_passive,
            is_disabled: s.is_disabled,
            enchant_type: s.enchant_type,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct BuffEffect {
    pub skill_id: u32,
    pub level: u32,
    pub sub_level: u32,
    pub duration_secs: u32,
    pub abnormal_type: u32,
    pub is_debuff: bool,
}

impl From<l2_sniffer_protocol::BuffEffect> for BuffEffect {
    fn from(b: l2_sniffer_protocol::BuffEffect) -> Self {
        Self {
            skill_id: b.skill_id,
            level: b.level,
            sub_level: b.sub_level,
            duration_secs: b.duration_secs,
            abnormal_type: b.abnormal_type,
            is_debuff: b.is_debuff,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct PrivateStoreItem {
    pub item_object_id: u32,
    pub item_id: u32,
    pub count: u64,
    pub price: u64,
    pub enchant_level: u16,
}

impl From<l2_sniffer_protocol::PrivateStoreItem> for PrivateStoreItem {
    fn from(p: l2_sniffer_protocol::PrivateStoreItem) -> Self {
        Self {
            item_object_id: p.item_object_id,
            item_id: p.item_id,
            count: p.count,
            price: p.price,
            enchant_level: p.enchant_level,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct CommissionItem {
    pub commission_id: u64,
    pub item_object_id: u32,
    pub item_id: u32,
    pub count: u64,
    pub price_per_unit: u64,
    pub total_price: u64,
    pub enchant_level: u16,
    pub seller_name: String,
    pub duration_days: u32,
    pub end_time_epoch_sec: u64,
}

impl From<l2_sniffer_protocol::CommissionItem> for CommissionItem {
    fn from(c: l2_sniffer_protocol::CommissionItem) -> Self {
        Self {
            commission_id: c.commission_id,
            item_object_id: c.item_object_id,
            item_id: c.item_id,
            count: c.count,
            price_per_unit: c.price_per_unit,
            total_price: c.total_price,
            enchant_level: c.enchant_level,
            seller_name: c.seller_name,
            duration_days: c.duration_days,
            end_time_epoch_sec: c.end_time_epoch_sec,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct WorldExchangeItem {
    pub listing_id: u64,
    pub item_id: u32,
    pub count: u64,
    pub price_adena: u64,
    pub price_lcoin: u64,
    pub enchant_level: u16,
    pub seller_name: String,
    pub end_time_epoch_sec: u64,
}

impl From<l2_sniffer_protocol::WorldExchangeItem> for WorldExchangeItem {
    fn from(w: l2_sniffer_protocol::WorldExchangeItem) -> Self {
        Self {
            listing_id: w.listing_id,
            item_id: w.item_id,
            count: w.count,
            price_adena: w.price_adena,
            price_lcoin: w.price_lcoin,
            enchant_level: w.enchant_level,
            seller_name: w.seller_name,
            end_time_epoch_sec: w.end_time_epoch_sec,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct EinhasadProduct {
    pub product_id: u32,
    pub item_id: u32,
    pub item_count: u32,
    pub price_gold_coins: u32,
    pub daily_limit: u32,
    pub buy_count: u32,
}

impl From<l2_sniffer_protocol::EinhasadProduct> for EinhasadProduct {
    fn from(e: l2_sniffer_protocol::EinhasadProduct) -> Self {
        Self {
            product_id: e.product_id,
            item_id: e.item_id,
            item_count: e.item_count,
            price_gold_coins: e.price_gold_coins,
            daily_limit: e.daily_limit,
            buy_count: e.buy_count,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct Location {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct Vitals {
    pub cur_hp: u32,
    pub max_hp: u32,
    pub cur_mp: u32,
    pub max_mp: u32,
    pub cur_cp: u32,
    pub max_cp: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct AccountSession {
    pub account_name: String,
    pub client_addr: String,
    pub character_roster: Vec<CharSelectSlot>,
    pub active_character: Option<String>,
    pub last_seen_epoch_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct PrivateStoreSession {
    pub seller_object_id: u32,
    pub seller_name: Option<String>,
    pub store_type: PrivateStoreType,
    pub store_title: String,
    pub items: Vec<PrivateStoreItem>,
    pub last_seen_epoch_ms: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "server", derive(async_graphql::SimpleObject))]
pub struct MarketState {
    pub private_stores: Vec<PrivateStoreSession>,
    pub commission_items: Vec<CommissionItem>,
    pub world_exchange_items: Vec<WorldExchangeItem>,
    pub einhasad_products: Vec<EinhasadProduct>,
}
