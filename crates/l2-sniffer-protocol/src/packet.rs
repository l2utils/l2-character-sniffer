//! Lineage 2 Protocol Packet Payloads and Structs.

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

/// Parsed Lineage 2 packet enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum L2Packet {
    AuthLogin(AuthLoginPacket),
    CharSelectInfo(CharSelectInfoPacket),
    UserInfo(UserInfoPacket),
    StatusUpdate(StatusUpdatePacket),
    ItemList(ItemListPacket),
    InventoryUpdate(InventoryUpdatePacket),
    WarehouseList(WarehouseListPacket),
    SkillList(SkillListPacket),
    AbnormalStatusUpdate(AbnormalStatusUpdatePacket),
    MagicEffectIcons(MagicEffectIconsPacket),
    PrivateStore(PrivateStorePacket),
    CommissionList(CommissionListPacket),
    WorldExchangeList(WorldExchangeListPacket),
    EinhasadStore(EinhasadStorePacket),
    MoveToLocation(MoveToLocationPacket),
    SystemMessage(SystemMessagePacket),
    Raw {
        opcode: u8,
        #[serde(skip_serializing)]
        payload: Vec<u8>,
    },
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AuthLoginPacket {
    pub account_name: String,
    pub session_key1: u32,
    pub session_key2: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CharSelectInfoPacket {
    pub account_name: String,
    pub character_slots: Vec<CharSelectSlot>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct UserInfoPacket {
    pub object_id: u32,
    pub session_id: u32,
    pub name: String,
    pub title: String,
    pub class_id: u32,
    pub level: u32,
    pub exp: u64,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub heading: i32,
    pub cur_hp: u32,
    pub max_hp: u32,
    pub cur_mp: u32,
    pub max_mp: u32,
    pub cur_cp: u32,
    pub max_cp: u32,
    pub sp: u32,
    pub cur_load: u32,
    pub max_load: u32,
    pub p_atk: u32,
    pub p_def: u32,
    pub m_atk: u32,
    pub m_def: u32,
    pub p_atk_spd: u32,
    pub m_atk_spd: u32,
    pub karma: u32,
    pub pk_kills: u32,
    pub pvp_kills: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusUpdateAttribute {
    pub attr_id: u32,
    pub value: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StatusUpdatePacket {
    pub object_id: u32,
    pub attributes: Vec<StatusUpdateAttribute>,
}

pub fn item_type_to_name(t: u16) -> &'static str {
    match t {
        0 => "Weapon",
        1 => "Armor",
        2 => "Accessory",
        3 => "Quest",
        4 => "Currency",
        5 => "EtcItem",
        _ => "Item",
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ItemInfo {
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemListPacket {
    pub show_window: bool,
    pub items: Vec<ItemInfo>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InventoryUpdatePacket {
    pub items: Vec<ItemInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WarehouseListPacket {
    pub wh_type: WarehouseType,
    pub player_adena: u64,
    pub items: Vec<ItemInfo>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillEntry {
    pub skill_id: u32,
    pub level: u32,
    pub sub_level: u32,
    pub is_passive: bool,
    pub is_disabled: bool,
    pub enchant_type: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillListPacket {
    pub skills: Vec<SkillEntry>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuffEffect {
    pub skill_id: u32,
    pub level: u32,
    pub sub_level: u32,
    pub duration_secs: u32,
    pub abnormal_type: u32,
    pub is_debuff: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbnormalStatusUpdatePacket {
    pub buffs: Vec<BuffEffect>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MagicEffectIconsPacket {
    pub buffs: Vec<BuffEffect>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateStoreItem {
    pub item_object_id: u32,
    pub item_id: u32,
    pub count: u64,
    pub price: u64,
    pub enchant_level: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrivateStorePacket {
    pub seller_object_id: u32,
    pub store_type: PrivateStoreType,
    pub store_title: String,
    pub items: Vec<PrivateStoreItem>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommissionListPacket {
    pub items: Vec<CommissionItem>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorldExchangeListPacket {
    pub items: Vec<WorldExchangeItem>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EinhasadProduct {
    pub product_id: u32,
    pub item_id: u32,
    pub item_count: u32,
    pub price_gold_coins: u32,
    pub daily_limit: u32,
    pub buy_count: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EinhasadStorePacket {
    pub products: Vec<EinhasadProduct>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MoveToLocationPacket {
    pub object_id: u32,
    pub target_x: i32,
    pub target_y: i32,
    pub target_z: i32,
    pub origin_x: i32,
    pub origin_y: i32,
    pub origin_z: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemMessagePacket {
    pub message_id: u32,
}

impl L2Packet {
    /// Parses a raw server-to-client or client-to-server decrypted payload.
    pub fn parse(opcode: u8, payload: &[u8]) -> Self {
        let mut cursor = Cursor::new(payload);
        match opcode {
            // UserInfo / CharSelected (Opcode 0x04 / 0x0B)
            0x04 | 0x0b => Self::parse_char_selected_retail(&mut cursor)
                .map(L2Packet::UserInfo)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            // CharSelectInfo / Roster (Opcode 0x09 / 0x1F / 0x13 / 0x71)
            0x09 | 0x1f | 0x13 | 0x71 => Self::parse_char_select_info_retail(&mut cursor, payload)
                .map(L2Packet::CharSelectInfo)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            // StatusUpdate (Opcode 0x0E / 0x18)
            0x0e | 0x18 => Self::parse_status_update(&mut cursor)
                .map(L2Packet::StatusUpdate)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            // ItemList (Opcode 0x11 / 0x1B)
            0x11 | 0x1b => Self::parse_item_list(&mut cursor)
                .map(L2Packet::ItemList)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            // InventoryUpdate (Opcode 0x21 / 0x27)
            0x21 | 0x27 => Self::parse_inventory_update(&mut cursor)
                .map(L2Packet::InventoryUpdate)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            // WarehouseDepositList (0x41) / WarehouseWithdrawList (0x42)
            0x41 | 0x42 => Self::parse_warehouse_list(&mut cursor, opcode)
                .map(L2Packet::WarehouseList)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            // SkillList (Opcode 0x58 / 0x5F)
            0x58 | 0x5f => Self::parse_skill_list(&mut cursor)
                .map(L2Packet::SkillList)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            // MagicEffectIcons (Opcode 0x7F)
            0x7f => Self::parse_magic_effect_icons(&mut cursor)
                .map(L2Packet::MagicEffectIcons)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            // AbnormalStatusUpdate (Opcode 0x85)
            0x85 => Self::parse_abnormal_status_update(&mut cursor)
                .map(L2Packet::AbnormalStatusUpdate)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            // PrivateStoreListSell (Opcode 0x9B / 0xA1) & PrivateStoreListBuy (0xB8 / 0xBE)
            0x9b | 0xa1 => Self::parse_private_store(&mut cursor, PrivateStoreType::Sell)
                .map(L2Packet::PrivateStore)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            0xb8 | 0xbe => Self::parse_private_store(&mut cursor, PrivateStoreType::Buy)
                .map(L2Packet::PrivateStore)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            // Extended Server Packets (Opcode 0xFE)
            0xfe => Self::parse_extended(&mut cursor, payload),
            // MoveToLocation (Opcode 0x01 / 0x72)
            0x01 | 0x72 => Self::parse_move_to_location(&mut cursor)
                .map(L2Packet::MoveToLocation)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            _ => L2Packet::Raw {
                opcode,
                payload: payload.to_vec(),
            },
        }
    }

    /// Parses modern retail character selection roster (Opcode 0x09)
    fn parse_char_select_info_retail(
        r: &mut Cursor<&[u8]>,
        raw: &[u8],
    ) -> Result<CharSelectInfoPacket, std::io::Error> {
        if raw.len() < 16 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Packet too short",
            ));
        }
        let count = r.read_u32::<LittleEndian>()?;
        if count == 0 || count > 50 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid character count",
            ));
        }
        let _max_slots = r.read_u32::<LittleEndian>()?;
        let _active = r.read_u8()?;
        let _sub_count = r.read_u32::<LittleEndian>()?;
        let _pad = r.read_u16::<LittleEndian>()?;
        let _pad1 = r.read_u8()?;

        let mut character_slots = Vec::with_capacity(count as usize);
        let mut account_name = String::new();

        for i in 0..count {
            if let Ok(name) = read_l2_string(r) {
                if name.is_empty() {
                    break;
                }
                let char_id = r.read_u32::<LittleEndian>().unwrap_or_default();
                let title = read_l2_string(r).unwrap_or_default();
                let session_id = r.read_u32::<LittleEndian>().unwrap_or_default();
                if session_id > 0 && account_name.is_empty() {
                    account_name = format!("#{session_id}");
                }
                let _clan_id = r.read_u32::<LittleEndian>().unwrap_or_default();
                let _builder = r.read_u32::<LittleEndian>().unwrap_or_default();
                let _sex = r.read_u32::<LittleEndian>().unwrap_or_default();
                let _race = r.read_u32::<LittleEndian>().unwrap_or_default();
                let class_id = r.read_u32::<LittleEndian>().unwrap_or_default();
                let _active_flag = r.read_u32::<LittleEndian>().unwrap_or_default();
                let _x = r.read_i32::<LittleEndian>().unwrap_or_default();
                let _y = r.read_i32::<LittleEndian>().unwrap_or_default();
                let _z = r.read_i32::<LittleEndian>().unwrap_or_default();
                let cur_hp = r.read_f64::<LittleEndian>().unwrap_or_default();
                let cur_mp = r.read_f64::<LittleEndian>().unwrap_or_default();
                let sp = r.read_u64::<LittleEndian>().unwrap_or_default();
                let exp = r.read_u64::<LittleEndian>().unwrap_or_default();
                let exp_pct_raw = r.read_f64::<LittleEndian>().unwrap_or_default();
                let exp_percent = if exp_pct_raw <= 1.0 {
                    exp_pct_raw * 100.0
                } else {
                    exp_pct_raw
                };
                let level = r.read_u32::<LittleEndian>().unwrap_or_default();
                let reputation = r.read_i32::<LittleEndian>().unwrap_or_default();
                let pk_kills = r.read_u32::<LittleEndian>().unwrap_or_default();
                let pvp_kills = r.read_u32::<LittleEndian>().unwrap_or_default();

                let mut last_access = 0;
                let cur_pos = r.position() as usize;
                if cur_pos < raw.len() {
                    let end_pos = (cur_pos + 600).min(raw.len());
                    for chunk in raw[cur_pos..end_pos].windows(4) {
                        let ts = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
                        if ts >= 1_500_000_000 && ts <= 2_100_000_000 {
                            last_access = ts;
                            break;
                        }
                    }
                }

                character_slots.push(CharSelectSlot {
                    name,
                    title,
                    char_id,
                    level,
                    class_id,
                    cur_hp,
                    max_hp: cur_hp,
                    cur_mp,
                    max_mp: cur_mp,
                    sp,
                    exp,
                    exp_percent,
                    reputation,
                    pk_kills,
                    pvp_kills,
                    vitality: 100,
                    last_access,
                });

                // Find next character slot dynamically by finding next UTF-16 string candidate
                if i + 1 < count {
                    let pos = r.position() as usize;
                    let mut found = false;
                    for offset in 100..raw.len().saturating_sub(pos + 30) {
                        let candidate_pos = pos + offset;
                        // Check if bytes at candidate_pos form a valid UTF-16 name followed by u32 and account name
                        if candidate_pos + 4 < raw.len()
                            && raw[candidate_pos + 1] == 0
                            && raw[candidate_pos].is_ascii_alphanumeric()
                        {
                            let mut test_r = Cursor::new(&raw[candidate_pos..]);
                            if let Ok(test_name) = read_l2_string(&mut test_r) {
                                if test_name.len() >= 2
                                    && test_name
                                        .chars()
                                        .all(|c| c.is_ascii_alphanumeric() || c == '_')
                                {
                                    let _test_id =
                                        test_r.read_u32::<LittleEndian>().unwrap_or_default();
                                    if let Ok(test_acc) = read_l2_string(&mut test_r) {
                                        if test_acc == account_name
                                            || (test_acc.len() >= 2
                                                && test_acc
                                                    .chars()
                                                    .all(|c| c.is_ascii_alphanumeric()))
                                        {
                                            r.set_position(candidate_pos as u64);
                                            found = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !found {
                        break;
                    }
                }
            }
        }

        Ok(CharSelectInfoPacket {
            account_name,
            character_slots,
        })
    }

    /// Parses client-to-server packets (e.g. AuthLogin).
    pub fn parse_client(opcode: u8, payload: &[u8]) -> Self {
        let mut cursor = Cursor::new(payload);
        match opcode {
            0x08 | 0x2b | 0x00 => {
                if let Ok(auth) = Self::parse_auth_login(&mut cursor) {
                    return L2Packet::AuthLogin(auth);
                }
            }
            0xd0 => {
                // Client Extended Opcode 0xD0
                if payload.len() >= 2 {
                    let sub_op = u16::from_le_bytes([payload[0], payload[1]]);
                    if sub_op == 0x0240 || sub_op == 0x0008 || sub_op == 0x002b || sub_op == 0x0001
                    {
                        let mut sub_cursor = Cursor::new(&payload[2..]);
                        if let Ok(auth) = Self::parse_auth_login(&mut sub_cursor) {
                            return L2Packet::AuthLogin(auth);
                        }
                    }
                }
            }
            _ => {}
        }
        L2Packet::Raw {
            opcode,
            payload: payload.to_vec(),
        }
    }

    /// Parses modern retail CharSelected (Opcode 0x0B)
    fn parse_char_selected_retail(r: &mut Cursor<&[u8]>) -> Result<UserInfoPacket, std::io::Error> {
        let name = read_l2_string(r)?;
        if name.len() < 2
            || name.len() > 32
            || !name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-' || c == ' ')
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid character name",
            ));
        }
        let object_id = r.read_u32::<LittleEndian>()?;
        if object_id == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid character object_id",
            ));
        }
        let _padding = r.read_u16::<LittleEndian>().unwrap_or_default();
        let session_id = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _clan_id = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _builder = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _sex = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _race = r.read_u32::<LittleEndian>().unwrap_or_default();
        let class_id = r.read_u32::<LittleEndian>().unwrap_or_default();
        if class_id > 300 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid class_id",
            ));
        }
        let _active = r.read_u32::<LittleEndian>().unwrap_or_default();
        let x = r.read_i32::<LittleEndian>().unwrap_or_default();
        let y = r.read_i32::<LittleEndian>().unwrap_or_default();
        let z = r.read_i32::<LittleEndian>().unwrap_or_default();
        let cur_hp = r
            .read_f64::<LittleEndian>()
            .map(|v| v as u32)
            .unwrap_or_default();
        let cur_mp = r
            .read_f64::<LittleEndian>()
            .map(|v| v as u32)
            .unwrap_or_default();
        let sp = r
            .read_u64::<LittleEndian>()
            .map(|v| v as u32)
            .unwrap_or_default();
        let exp = r.read_u64::<LittleEndian>().unwrap_or_default();
        let level = r.read_u32::<LittleEndian>().unwrap_or_default();
        if level == 0 || level > 130 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid character level",
            ));
        }

        Ok(UserInfoPacket {
            object_id,
            session_id,
            name,
            title: String::new(),
            class_id,
            level,
            exp,
            x,
            y,
            z,
            heading: 0,
            cur_hp,
            max_hp: cur_hp,
            cur_mp,
            max_mp: cur_mp,
            sp,
            ..Default::default()
        })
    }

    fn parse_auth_login(r: &mut Cursor<&[u8]>) -> Result<AuthLoginPacket, std::io::Error> {
        let raw_buf = *r.get_ref();
        let start_pos = r.position() as usize;
        let slice = if start_pos < raw_buf.len() {
            &raw_buf[start_pos..]
        } else {
            raw_buf
        };

        // 1. Try reading standard L2 format (Account name string first: UTF-16 / ASCII)
        let mut test_r = Cursor::new(slice);
        if let Ok(name) = read_l2_string(&mut test_r).or_else(|_| read_ascii_string(&mut test_r)) {
            if name.len() >= 2
                && name.len() <= 32
                && name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
            {
                let session_key1 = test_r.read_u32::<LittleEndian>().unwrap_or_default();
                let session_key2 = test_r.read_u32::<LittleEndian>().unwrap_or_default();
                return Ok(AuthLoginPacket {
                    account_name: name,
                    session_key1,
                    session_key2,
                });
            }
        }

        // 2. Try reading retail format (session_key1: u32, token_len: u16, raw_name, session_key2: u32)
        let mut retail_r = Cursor::new(slice);
        let session_key1 = retail_r.read_u32::<LittleEndian>().unwrap_or_default();
        let token_len = retail_r.read_u16::<LittleEndian>().unwrap_or_default();
        let raw_name = if let Ok(s) = read_ascii_string(&mut retail_r) {
            s
        } else {
            read_l2_string(&mut retail_r).unwrap_or_default()
        };
        let session_key2 = retail_r.read_u32::<LittleEndian>().unwrap_or_default();

        if !raw_name.is_empty()
            && raw_name.len() <= 32
            && raw_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        {
            return Ok(AuthLoginPacket {
                account_name: raw_name,
                session_key1,
                session_key2,
            });
        }

        if session_key1 > 0 && token_len > 0 && token_len <= 1024 {
            return Ok(AuthLoginPacket {
                account_name: format!("#{session_key1}"),
                session_key1,
                session_key2,
            });
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Invalid auth login payload",
        ))
    }

    fn parse_status_update(r: &mut Cursor<&[u8]>) -> Result<StatusUpdatePacket, std::io::Error> {
        let object_id = r.read_u32::<LittleEndian>()?;
        let count = r.read_u32::<LittleEndian>()?;
        if count > 200 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Too many status attributes",
            ));
        }
        let mut attributes = Vec::with_capacity(count as usize);

        for _ in 0..count {
            let attr_id = r.read_u32::<LittleEndian>()?;
            let value = r.read_u32::<LittleEndian>()?;
            attributes.push(StatusUpdateAttribute { attr_id, value });
        }

        Ok(StatusUpdatePacket {
            object_id,
            attributes,
        })
    }

    fn parse_item_list(r: &mut Cursor<&[u8]>) -> Result<ItemListPacket, std::io::Error> {
        let raw_buf = *r.get_ref();
        let start_pos = r.position() as usize;
        let slice = if start_pos < raw_buf.len() {
            &raw_buf[start_pos..]
        } else {
            raw_buf
        };

        if slice.len() < 3 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "ItemList payload too short",
            ));
        }

        // Try candidate header layouts:
        // Layout A: show_window (u8), count (u16) -> offset 3
        // Layout B: show_window (u8), sub_flag (u8), count (u16) -> offset 4
        // Layout C: show_window (u16), count (u16) -> offset 4
        // Layout D: show_window (u8), count (u32) -> offset 5
        let mut best_show_window = true;
        let mut best_count = 0usize;
        let mut best_offset = 3usize;

        // Try Layout B / C (offset 4)
        if slice.len() >= 4 {
            let cnt_b = u16::from_le_bytes([slice[2], slice[3]]) as usize;
            if cnt_b > 0 && cnt_b <= 500 {
                let remaining = slice.len() - 4;
                if remaining >= cnt_b
                    && (remaining % cnt_b == 0
                        || (remaining / cnt_b >= 32 && remaining / cnt_b <= 96))
                {
                    best_show_window = slice[0] != 0;
                    best_count = cnt_b;
                    best_offset = 4;
                }
            }
        }

        // Try Layout A (offset 3) if Layout B didn't match cleanly
        if best_count == 0 && slice.len() >= 3 {
            let cnt_a = u16::from_le_bytes([slice[1], slice[2]]) as usize;
            if cnt_a > 0 && cnt_a <= 500 {
                let remaining = slice.len() - 3;
                if remaining >= cnt_a
                    && (remaining % cnt_a == 0
                        || (remaining / cnt_a >= 32 && remaining / cnt_a <= 96))
                {
                    best_show_window = slice[0] != 0;
                    best_count = cnt_a;
                    best_offset = 3;
                }
            }
        }

        if best_count == 0 {
            // Fallback: read u8 show_window, u16 count
            best_show_window = slice[0] != 0;
            best_count = u16::from_le_bytes([slice[1], slice[2]]) as usize;
            best_offset = 3;
        }

        if best_count > 500 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Too many items in ItemList",
            ));
        }

        let item_payload = &slice[best_offset..];
        let items = Self::parse_item_array(item_payload, best_count);

        Ok(ItemListPacket {
            show_window: best_show_window,
            items,
        })
    }

    fn parse_inventory_update(
        r: &mut Cursor<&[u8]>,
    ) -> Result<InventoryUpdatePacket, std::io::Error> {
        let raw_buf = *r.get_ref();
        let start_pos = r.position() as usize;
        let slice = if start_pos < raw_buf.len() {
            &raw_buf[start_pos..]
        } else {
            raw_buf
        };

        if slice.len() < 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "InventoryUpdate payload too short",
            ));
        }

        let count = u16::from_le_bytes([slice[0], slice[1]]) as usize;
        if count > 500 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Too many items in InventoryUpdate",
            ));
        }

        let item_payload = &slice[2..];
        let items = Self::parse_item_array(item_payload, count);

        Ok(InventoryUpdatePacket { items })
    }

    fn parse_warehouse_list(
        r: &mut Cursor<&[u8]>,
        opcode: u8,
    ) -> Result<WarehouseListPacket, std::io::Error> {
        let raw_type = r.read_u16::<LittleEndian>().unwrap_or(0);
        let wh_type = match raw_type {
            1 => WarehouseType::Private,
            2 => WarehouseType::Clan,
            3 => WarehouseType::Castle,
            4 => WarehouseType::Freight,
            5 => WarehouseType::Package,
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid warehouse type",
                ))
            }
        };
        let player_adena = if opcode == 0x42 {
            r.read_u64::<LittleEndian>()
                .or_else(|_| r.read_u32::<LittleEndian>().map(|v| v as u64))
                .unwrap_or_default()
        } else {
            0
        };
        if player_adena > 100_000_000_000 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid warehouse adena",
            ));
        }
        let count = r
            .read_u16::<LittleEndian>()
            .map(|c| c as usize)
            .unwrap_or_default();
        if count > 500 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Too many warehouse items",
            ));
        }

        let raw_buf = *r.get_ref();
        let pos = r.position() as usize;
        let item_payload = if pos < raw_buf.len() {
            &raw_buf[pos..]
        } else {
            &[]
        };
        let items = Self::parse_item_array(item_payload, count);

        Ok(WarehouseListPacket {
            wh_type,
            player_adena,
            items,
        })
    }

    /// Parses a contiguous slice of N items using calculated stride and chronicle detection.
    fn parse_item_array(payload: &[u8], count: usize) -> Vec<ItemInfo> {
        if count == 0 || payload.is_empty() {
            return Vec::new();
        }

        let calculated_stride = payload.len() / count;
        // Standard item strides in L2: 22 (legacy minimal), 36 (C4), 42 (Interlude), 56 (Gracia), 64 (HF), 68/70/72 (Essence/Modern)
        let stride = if calculated_stride >= 18 && calculated_stride <= 128 {
            calculated_stride
        } else {
            56
        };

        let mut items = Vec::with_capacity(count);

        for i in 0..count {
            let start = i * stride;
            let end = (start + stride).min(payload.len());
            if start >= payload.len() || end <= start {
                break;
            }
            let item_buf = &payload[start..end];
            if let Some(item) = Self::parse_single_item(item_buf) {
                items.push(item);
            }
        }

        items
    }

    /// Parses an individual item binary buffer into ItemInfo.
    fn parse_single_item(buf: &[u8]) -> Option<ItemInfo> {
        if buf.len() < 16 {
            return None;
        }

        // 1. Check Legacy format: item_type1 (u16 @ 0), object_id (u32 @ 2), item_id (u32 @ 6), slot (u32 @ 10), count (u32/u64 @ 14)
        if buf.len() >= 22 {
            let l_type = u16::from_le_bytes([buf[0], buf[1]]);
            let l_obj = u32::from_le_bytes([buf[2], buf[3], buf[4], buf[5]]);
            let l_item = u32::from_le_bytes([buf[6], buf[7], buf[8], buf[9]]);
            let l_slot = u32::from_le_bytes([buf[10], buf[11], buf[12], buf[13]]);
            let l_count = if buf.len() >= 26 {
                u64::from_le_bytes([
                    buf[14], buf[15], buf[16], buf[17], buf[18], buf[19], buf[20], buf[21],
                ])
            } else {
                u32::from_le_bytes([buf[14], buf[15], buf[16], buf[17]]) as u64
            };

            // If legacy item_type1 is 0..3, and l_item is a valid item ID, and m_item has 0 in lower 16 bits (due to shift)
            let m_item_raw = if buf.len() >= 8 {
                u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]])
            } else {
                0
            };
            let is_legacy = l_type <= 3
                && l_item > 0
                && l_item < 5_000_000
                && l_count > 0
                && l_count < 100_000_000_000
                && (m_item_raw & 0xFFFF == 0 || (m_item_raw > 1_000_000 && l_item < 100_000));

            if is_legacy {
                let custom_type1 = if buf.len() >= 20 {
                    u16::from_le_bytes([buf[18], buf[19]])
                } else {
                    0
                };
                let equipped = if buf.len() >= 22 {
                    u16::from_le_bytes([buf[20], buf[21]]) != 0
                } else {
                    false
                };
                let enchant_level = if buf.len() >= 28 {
                    u16::from_le_bytes([buf[26], buf[27]])
                } else {
                    0
                };
                let is_augmented = if buf.len() >= 34 {
                    u32::from_le_bytes([buf[30], buf[31], buf[32], buf[33]]) != 0
                } else {
                    false
                };
                let mana = if buf.len() >= 38 {
                    i32::from_le_bytes([buf[34], buf[35], buf[36], buf[37]])
                } else {
                    -1
                };
                let durability = if buf.len() >= 42 {
                    i32::from_le_bytes([buf[38], buf[39], buf[40], buf[41]])
                } else {
                    -1
                };

                // Resolve item_type: 0=Weapon, 1=Armor, 2=Accessory, 3=Quest, 4=Currency, 5=EtcItem
                let item_type = if l_type > 0 {
                    l_type
                } else if l_item == 57 || l_item == 5575 || l_item == 5560 {
                    4 // Currency
                } else if (l_slot & 0x4080) != 0 {
                    0 // Weapon
                } else if (l_slot & (0x0001 | 0x0002 | 0x0004 | 0x0010 | 0x0020)) != 0 {
                    2 // Accessory
                } else if (l_slot & (0x0100 | 0x0200 | 0x0400 | 0x0800 | 0x1000 | 0x2000)) != 0 {
                    1 // Armor
                } else {
                    5 // EtcItem
                };

                return Some(ItemInfo {
                    object_id: l_obj,
                    item_id: l_item,
                    count: l_count,
                    item_type,
                    item_type_name: item_type_to_name(item_type).to_string(),
                    equipped,
                    slot: l_slot,
                    enchant_level,
                    custom_type1,
                    is_augmented,
                    mana,
                    durability,
                });
            }
        }

        // 2. Modern format: object_id (u32 @ 0), item_id (u32 @ 4), slot (u32 @ 8), count (u64 @ 12)
        let m_obj = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
        let m_item = u32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
        let m_slot = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);
        let m_count = if buf.len() >= 20 {
            u64::from_le_bytes([
                buf[12], buf[13], buf[14], buf[15], buf[16], buf[17], buf[18], buf[19],
            ])
        } else {
            u32::from_le_bytes([buf[12], buf[13], buf[14], buf[15]]) as u64
        };

        let modern_valid =
            m_item > 0 && m_item < 5_000_000 && m_count > 0 && m_count < 100_000_000_000;

        if modern_valid {
            let item_type_raw = if buf.len() >= 22 {
                u16::from_le_bytes([buf[20], buf[21]])
            } else {
                0
            };
            let custom_type1 = if buf.len() >= 24 {
                u16::from_le_bytes([buf[22], buf[23]])
            } else {
                0
            };
            let equipped = if buf.len() >= 26 {
                u16::from_le_bytes([buf[24], buf[25]]) != 0
            } else {
                false
            };
            let body_part = if buf.len() >= 30 {
                u32::from_le_bytes([buf[26], buf[27], buf[28], buf[29]])
            } else {
                0
            };
            let enchant_level = if buf.len() >= 32 {
                u16::from_le_bytes([buf[30], buf[31]])
            } else {
                0
            };
            let custom_type2 = if buf.len() >= 34 {
                u16::from_le_bytes([buf[32], buf[33]])
            } else {
                0
            };
            let is_augmented = if buf.len() >= 38 {
                u32::from_le_bytes([buf[34], buf[35], buf[36], buf[37]]) != 0
            } else {
                false
            };
            let mana = if buf.len() >= 42 {
                i32::from_le_bytes([buf[38], buf[39], buf[40], buf[41]])
            } else {
                -1
            };
            let durability = if buf.len() >= 46 {
                i32::from_le_bytes([buf[42], buf[43], buf[44], buf[45]])
            } else {
                -1
            };

            // Resolve item_type: 0=Weapon, 1=Shield/Armor, 2=Accessory/Jewelry, 3=Quest, 4=Currency, 5=EtcItem/Consumable
            let item_type = if item_type_raw > 0 {
                item_type_raw
            } else if m_item == 57
                || m_item == 5575
                || m_item == 5560
                || m_item == 37000
                || m_item == 40000
            {
                4 // Currency (Adena / L-Coin)
            } else if body_part != 0 {
                if (body_part & 0x4080) != 0 {
                    0 // Weapon (R_HAND, LR_HAND)
                } else if (body_part
                    & (0x0001 | 0x0002 | 0x0004 | 0x0010 | 0x0020 | 0x100000 | 0x200000 | 0x400000))
                    != 0
                {
                    2 // Jewelry / Accessory (EAR, NECK, FINGER, HAIR)
                } else if (body_part
                    & (0x0100
                        | 0x0200
                        | 0x0400
                        | 0x0800
                        | 0x1000
                        | 0x2000
                        | 0x8000
                        | 0x10000
                        | 0x20000))
                    != 0
                {
                    1 // Shield / Armor (CHEST, LEGS, GLOVES, FEET, HELM, CLOAK, BELT)
                } else {
                    0
                }
            } else if m_slot > 0 && equipped {
                if (m_slot & 0x4080) != 0 {
                    0 // Weapon
                } else if (m_slot & (0x0001 | 0x0002 | 0x0004 | 0x0010 | 0x0020)) != 0 {
                    2 // Jewelry
                } else {
                    1 // Armor
                }
            } else {
                5 // EtcItem / Consumable
            };

            return Some(ItemInfo {
                object_id: m_obj,
                item_id: m_item,
                count: m_count,
                item_type,
                item_type_name: item_type_to_name(item_type).to_string(),
                equipped,
                slot: m_slot,
                enchant_level,
                custom_type1: custom_type1 | custom_type2,
                is_augmented,
                mana,
                durability,
            });
        }

        // Fallback: modern
        Some(ItemInfo {
            object_id: m_obj,
            item_id: m_item,
            count: if m_count > 0 && m_count < 100_000_000_000 {
                m_count
            } else {
                1
            },
            item_type: 0,
            item_type_name: item_type_to_name(0).to_string(),
            equipped: false,
            slot: m_slot,
            enchant_level: 0,
            custom_type1: 0,
            is_augmented: false,
            mana: -1,
            durability: -1,
        })
    }

    fn parse_skill_list(r: &mut Cursor<&[u8]>) -> Result<SkillListPacket, std::io::Error> {
        let count = r.read_u32::<LittleEndian>().map(|c| c as usize)?;
        if count > 2000 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Too many skills",
            ));
        }
        let mut skills = Vec::with_capacity(count);
        for _ in 0..count {
            let is_passive = r
                .read_u32::<LittleEndian>()
                .map(|v| v != 0)
                .unwrap_or(false);
            let level = r.read_u32::<LittleEndian>().unwrap_or(1);
            let skill_id = r.read_u32::<LittleEndian>().unwrap_or(0);
            let is_disabled = r.read_u8().map(|v| v != 0).unwrap_or(false);
            let enchant_type = r.read_u32::<LittleEndian>().unwrap_or(0);

            if skill_id > 0 {
                skills.push(SkillEntry {
                    skill_id,
                    level,
                    sub_level: 0,
                    is_passive,
                    is_disabled,
                    enchant_type,
                });
            }
        }
        Ok(SkillListPacket { skills })
    }

    fn parse_magic_effect_icons(
        r: &mut Cursor<&[u8]>,
    ) -> Result<MagicEffectIconsPacket, std::io::Error> {
        let count = r
            .read_u16::<LittleEndian>()
            .map(|c| c as usize)
            .or_else(|_| r.read_u32::<LittleEndian>().map(|c| c as usize))?;
        if count > 200 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Too many effects",
            ));
        }
        let mut buffs = Vec::with_capacity(count);
        for _ in 0..count {
            let skill_id = r.read_u32::<LittleEndian>()?;
            let level = r
                .read_u16::<LittleEndian>()
                .map(|v| v as u32)
                .or_else(|_| r.read_u32::<LittleEndian>())
                .unwrap_or(1);
            let duration_secs = r.read_u32::<LittleEndian>().unwrap_or(0);

            buffs.push(BuffEffect {
                skill_id,
                level,
                sub_level: 0,
                duration_secs,
                abnormal_type: 0,
                is_debuff: false,
            });
        }
        Ok(MagicEffectIconsPacket { buffs })
    }

    fn parse_abnormal_status_update(
        r: &mut Cursor<&[u8]>,
    ) -> Result<AbnormalStatusUpdatePacket, std::io::Error> {
        let count = r
            .read_u16::<LittleEndian>()
            .map(|c| c as usize)
            .or_else(|_| r.read_u32::<LittleEndian>().map(|c| c as usize))?;
        if count > 200 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Too many abnormal effects",
            ));
        }
        let mut buffs = Vec::with_capacity(count);
        for _ in 0..count {
            let skill_id = r.read_u32::<LittleEndian>()?;
            let level = r.read_u16::<LittleEndian>().map(|v| v as u32).unwrap_or(1);
            let sub_level = r.read_u32::<LittleEndian>().unwrap_or(0);
            let abnormal_type = r.read_u32::<LittleEndian>().unwrap_or(0);
            let duration_secs = r.read_u32::<LittleEndian>().unwrap_or(0);

            buffs.push(BuffEffect {
                skill_id,
                level,
                sub_level,
                duration_secs,
                abnormal_type,
                is_debuff: abnormal_type >= 100,
            });
        }
        Ok(AbnormalStatusUpdatePacket { buffs })
    }

    fn parse_private_store(
        r: &mut Cursor<&[u8]>,
        default_type: PrivateStoreType,
    ) -> Result<PrivateStorePacket, std::io::Error> {
        let seller_object_id = r.read_u32::<LittleEndian>()?;
        let raw_type = r.read_u32::<LittleEndian>().unwrap_or(0);
        let store_type = match raw_type {
            1 => PrivateStoreType::Sell,
            3 => PrivateStoreType::Buy,
            8 => PrivateStoreType::PackageSell,
            _ => default_type,
        };
        let _player_adena = r
            .read_u64::<LittleEndian>()
            .or_else(|_| r.read_u32::<LittleEndian>().map(|v| v as u64))
            .unwrap_or_default();
        let store_title = read_l2_string(r).unwrap_or_default();
        let count = r
            .read_u32::<LittleEndian>()
            .map(|c| c as usize)
            .or_else(|_| r.read_u16::<LittleEndian>().map(|c| c as usize))?;
        if count > 500 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Too many store items",
            ));
        }
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            let item_object_id = r.read_u32::<LittleEndian>().unwrap_or(0);
            let item_id = r.read_u32::<LittleEndian>().unwrap_or(0);
            let count = r
                .read_u64::<LittleEndian>()
                .or_else(|_| r.read_u32::<LittleEndian>().map(|v| v as u64))
                .unwrap_or(1);
            let price = r
                .read_u64::<LittleEndian>()
                .or_else(|_| r.read_u32::<LittleEndian>().map(|v| v as u64))
                .unwrap_or(0);
            let enchant_level = r.read_u16::<LittleEndian>().unwrap_or(0);

            items.push(PrivateStoreItem {
                item_object_id,
                item_id,
                count,
                price,
                enchant_level,
            });
        }
        Ok(PrivateStorePacket {
            seller_object_id,
            store_type,
            store_title,
            items,
        })
    }

    fn parse_extended(r: &mut Cursor<&[u8]>, payload: &[u8]) -> L2Packet {
        let sub_op = match r.read_u16::<LittleEndian>() {
            Ok(op) => op,
            Err(_) => {
                return L2Packet::Raw {
                    opcode: 0xfe,
                    payload: payload.to_vec(),
                }
            }
        };

        match sub_op {
            // ExCommissionItemList / ExCommissionInfo (0x018F / 0x00F4)
            0x018f | 0x00f4 => Self::parse_commission_list(r)
                .map(L2Packet::CommissionList)
                .unwrap_or(L2Packet::Raw {
                    opcode: 0xfe,
                    payload: payload.to_vec(),
                }),
            // ExWorldExchangeItemList (0x021B)
            0x021b => Self::parse_world_exchange_list(r)
                .map(L2Packet::WorldExchangeList)
                .unwrap_or(L2Packet::Raw {
                    opcode: 0xfe,
                    payload: payload.to_vec(),
                }),
            // ExEinhasadStoreList (0x0230)
            0x0230 => Self::parse_einhasad_store(r)
                .map(L2Packet::EinhasadStore)
                .unwrap_or(L2Packet::Raw {
                    opcode: 0xfe,
                    payload: payload.to_vec(),
                }),
            _ => L2Packet::Raw {
                opcode: 0xfe,
                payload: payload.to_vec(),
            },
        }
    }

    fn parse_commission_list(
        r: &mut Cursor<&[u8]>,
    ) -> Result<CommissionListPacket, std::io::Error> {
        let _list_type = r.read_u32::<LittleEndian>().unwrap_or(0);
        let count = r.read_u32::<LittleEndian>().map(|c| c as usize)?;
        if count > 500 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Too many commission items",
            ));
        }
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            let commission_id = r
                .read_u64::<LittleEndian>()
                .or_else(|_| r.read_u32::<LittleEndian>().map(|v| v as u64))
                .unwrap_or(0);
            let item_object_id = r.read_u32::<LittleEndian>().unwrap_or(0);
            let item_id = r.read_u32::<LittleEndian>().unwrap_or(0);
            let count = r
                .read_u64::<LittleEndian>()
                .or_else(|_| r.read_u32::<LittleEndian>().map(|v| v as u64))
                .unwrap_or(1);
            let price_per_unit = r
                .read_u64::<LittleEndian>()
                .or_else(|_| r.read_u32::<LittleEndian>().map(|v| v as u64))
                .unwrap_or(0);
            let total_price = price_per_unit.saturating_mul(count);
            let duration_days = r.read_u32::<LittleEndian>().unwrap_or(0);
            let end_time_epoch_sec = r.read_u32::<LittleEndian>().map(|v| v as u64).unwrap_or(0);
            let enchant_level = r.read_u16::<LittleEndian>().unwrap_or(0);
            let seller_name = read_l2_string(r).unwrap_or_default();

            items.push(CommissionItem {
                commission_id,
                item_object_id,
                item_id,
                count,
                price_per_unit,
                total_price,
                enchant_level,
                seller_name,
                duration_days,
                end_time_epoch_sec,
            });
        }
        Ok(CommissionListPacket { items })
    }

    fn parse_world_exchange_list(
        r: &mut Cursor<&[u8]>,
    ) -> Result<WorldExchangeListPacket, std::io::Error> {
        let count = r.read_u32::<LittleEndian>().map(|c| c as usize)?;
        if count > 500 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Too many world exchange items",
            ));
        }
        let mut items = Vec::with_capacity(count);
        for _ in 0..count {
            let listing_id = r
                .read_u64::<LittleEndian>()
                .or_else(|_| r.read_u32::<LittleEndian>().map(|v| v as u64))
                .unwrap_or(0);
            let item_id = r.read_u32::<LittleEndian>().unwrap_or(0);
            let count = r
                .read_u64::<LittleEndian>()
                .or_else(|_| r.read_u32::<LittleEndian>().map(|v| v as u64))
                .unwrap_or(1);
            let price_adena = r.read_u64::<LittleEndian>().unwrap_or(0);
            let price_lcoin = r.read_u64::<LittleEndian>().unwrap_or(0);
            let enchant_level = r.read_u16::<LittleEndian>().unwrap_or(0);
            let end_time_epoch_sec = r
                .read_u64::<LittleEndian>()
                .or_else(|_| r.read_u32::<LittleEndian>().map(|v| v as u64))
                .unwrap_or(0);
            let seller_name = read_l2_string(r).unwrap_or_default();

            items.push(WorldExchangeItem {
                listing_id,
                item_id,
                count,
                price_adena,
                price_lcoin,
                enchant_level,
                seller_name,
                end_time_epoch_sec,
            });
        }
        Ok(WorldExchangeListPacket { items })
    }

    fn parse_einhasad_store(r: &mut Cursor<&[u8]>) -> Result<EinhasadStorePacket, std::io::Error> {
        let count = r.read_u32::<LittleEndian>().map(|c| c as usize)?;
        if count > 500 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Too many store products",
            ));
        }
        let mut products = Vec::with_capacity(count);
        for _ in 0..count {
            let product_id = r.read_u32::<LittleEndian>().unwrap_or(0);
            let item_id = r.read_u32::<LittleEndian>().unwrap_or(0);
            let item_count = r.read_u32::<LittleEndian>().unwrap_or(1);
            let price_gold_coins = r.read_u32::<LittleEndian>().unwrap_or(0);
            let daily_limit = r.read_u32::<LittleEndian>().unwrap_or(0);
            let buy_count = r.read_u32::<LittleEndian>().unwrap_or(0);

            products.push(EinhasadProduct {
                product_id,
                item_id,
                item_count,
                price_gold_coins,
                daily_limit,
                buy_count,
            });
        }
        Ok(EinhasadStorePacket { products })
    }

    fn parse_move_to_location(
        r: &mut Cursor<&[u8]>,
    ) -> Result<MoveToLocationPacket, std::io::Error> {
        let object_id = r.read_u32::<LittleEndian>()?;
        let target_x = r.read_i32::<LittleEndian>()?;
        let target_y = r.read_i32::<LittleEndian>()?;
        let target_z = r.read_i32::<LittleEndian>()?;
        let origin_x = r.read_i32::<LittleEndian>().unwrap_or_default();
        let origin_y = r.read_i32::<LittleEndian>().unwrap_or_default();
        let origin_z = r.read_i32::<LittleEndian>().unwrap_or_default();

        Ok(MoveToLocationPacket {
            object_id,
            target_x,
            target_y,
            target_z,
            origin_x,
            origin_y,
            origin_z,
        })
    }
}

/// Reads a UTF-16LE null-terminated string with length safeguard.
pub fn read_l2_string(r: &mut Cursor<&[u8]>) -> Result<String, std::io::Error> {
    let mut u16_chars = Vec::new();
    let mut count = 0;
    loop {
        if count > 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "String too long",
            ));
        }
        match r.read_u16::<LittleEndian>() {
            Ok(0) => break,
            Ok(ch) => {
                u16_chars.push(ch);
                count += 1;
            }
            Err(e) => {
                if u16_chars.is_empty() {
                    return Err(e);
                }
                break;
            }
        }
    }
    String::from_utf16(&u16_chars)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Reads an ASCII null-terminated string with length safeguard.
pub fn read_ascii_string(r: &mut Cursor<&[u8]>) -> Result<String, std::io::Error> {
    let mut bytes = Vec::new();
    let mut count = 0;
    loop {
        if count > 1024 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "String too long",
            ));
        }
        match r.read_u8() {
            Ok(0) => break,
            Ok(b) => {
                bytes.push(b);
                count += 1;
            }
            Err(e) => {
                if bytes.is_empty() {
                    return Err(e);
                }
                break;
            }
        }
    }
    String::from_utf8(bytes).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_skill_list() {
        let mut data = Vec::new();
        data.extend_from_slice(&(1u32).to_le_bytes()); // count = 1
        data.extend_from_slice(&(0u32).to_le_bytes()); // is_passive = 0
        data.extend_from_slice(&(15u32).to_le_bytes()); // level = 15
        data.extend_from_slice(&(1069u32).to_le_bytes()); // skill_id = 1069 (Power Strike)
        data.push(0); // is_disabled = false
        data.extend_from_slice(&(0u32).to_le_bytes()); // enchant_type = 0

        let packet = L2Packet::parse(0x58, &data);
        match packet {
            L2Packet::SkillList(sl) => {
                assert_eq!(sl.skills.len(), 1);
                assert_eq!(sl.skills[0].skill_id, 1069);
                assert_eq!(sl.skills[0].level, 15);
                assert!(!sl.skills[0].is_passive);
            }
            _ => panic!("Expected SkillList packet"),
        }
    }

    #[test]
    fn test_parse_abnormal_status_update() {
        let mut data = Vec::new();
        data.extend_from_slice(&(1u16).to_le_bytes()); // count = 1
        data.extend_from_slice(&(1068u32).to_le_bytes()); // skill_id = 1068 (Might)
        data.extend_from_slice(&(3u16).to_le_bytes()); // level = 3
        data.extend_from_slice(&(0u32).to_le_bytes()); // sub_level = 0
        data.extend_from_slice(&(0u32).to_le_bytes()); // abnormal_type = 0
        data.extend_from_slice(&(1200u32).to_le_bytes()); // duration = 1200s

        let packet = L2Packet::parse(0x85, &data);
        match packet {
            L2Packet::AbnormalStatusUpdate(ab) => {
                assert_eq!(ab.buffs.len(), 1);
                assert_eq!(ab.buffs[0].skill_id, 1068);
                assert_eq!(ab.buffs[0].level, 3);
                assert_eq!(ab.buffs[0].duration_secs, 1200);
            }
            _ => panic!("Expected AbnormalStatusUpdate packet"),
        }
    }

    #[test]
    fn test_parse_warehouse_list() {
        let mut data = Vec::new();
        data.extend_from_slice(&(1u16).to_le_bytes()); // wh_type = 1 (Private)
        data.extend_from_slice(&(1u16).to_le_bytes()); // count = 1
                                                       // read_item_entry:
        data.extend_from_slice(&(0u16).to_le_bytes()); // item_type
        data.extend_from_slice(&(1001u32).to_le_bytes()); // object_id
        data.extend_from_slice(&(57u32).to_le_bytes()); // item_id = 57 (Adena)
        data.extend_from_slice(&(0u32).to_le_bytes()); // slot
        data.extend_from_slice(&(50000u64).to_le_bytes()); // count = 50,000

        let packet = L2Packet::parse(0x41, &data);
        match packet {
            L2Packet::WarehouseList(wh) => {
                assert_eq!(wh.wh_type, WarehouseType::Private);
                assert_eq!(wh.items.len(), 1);
                assert_eq!(wh.items[0].item_id, 57);
                assert_eq!(wh.items[0].count, 50000);
            }
            _ => panic!("Expected WarehouseList packet"),
        }
    }

    #[test]
    fn test_parse_auth_login_standard() {
        let mut data = Vec::new();
        // UTF-16LE "my_account\0"
        for ch in "my_account".encode_utf16() {
            data.extend_from_slice(&ch.to_le_bytes());
        }
        data.extend_from_slice(&(0u16).to_le_bytes());
        data.extend_from_slice(&(12345u32).to_le_bytes());
        data.extend_from_slice(&(67890u32).to_le_bytes());

        let packet = L2Packet::parse_client(0x2b, &data);
        match packet {
            L2Packet::AuthLogin(auth) => {
                assert_eq!(auth.account_name, "my_account");
                assert_eq!(auth.session_key1, 12345);
                assert_eq!(auth.session_key2, 67890);
            }
            _ => panic!("Expected AuthLogin packet"),
        }
    }

    #[test]
    fn test_parse_auth_login_retail() {
        let mut data = Vec::new();
        data.extend_from_slice(&(99999u32).to_le_bytes());
        data.extend_from_slice(&(10u16).to_le_bytes());
        data.extend_from_slice(b"retail_acc\0");
        data.extend_from_slice(&(88888u32).to_le_bytes());

        let packet = L2Packet::parse_client(0x08, &data);
        match packet {
            L2Packet::AuthLogin(auth) => {
                assert_eq!(auth.account_name, "retail_acc");
                assert_eq!(auth.session_key1, 99999);
                assert_eq!(auth.session_key2, 88888);
            }
            _ => panic!("Expected AuthLogin packet"),
        }
    }

    #[test]
    fn test_parse_item_list_modern() {
        let mut data = Vec::new();
        data.push(1); // show_window = true
        data.push(0); // sub_flag / block_mode
        data.extend_from_slice(&(2u16).to_le_bytes()); // count = 2

        // Item 1: Adena (64 bytes stride)
        let mut item1 = vec![0u8; 64];
        item1[0..4].copy_from_slice(&(268435456u32).to_le_bytes()); // object_id
        item1[4..8].copy_from_slice(&(57u32).to_le_bytes()); // item_id = 57 (Adena)
        item1[8..12].copy_from_slice(&(0u32).to_le_bytes()); // slot
        item1[12..20].copy_from_slice(&(15000000u64).to_le_bytes()); // count = 15,000,000
        data.extend_from_slice(&item1);

        // Item 2: Weapon (e.g. Damascus Sword +4)
        let mut item2 = vec![0u8; 64];
        item2[0..4].copy_from_slice(&(268435457u32).to_le_bytes()); // object_id
        item2[4..8].copy_from_slice(&(72u32).to_le_bytes()); // item_id = 72
        item2[8..12].copy_from_slice(&(128u32).to_le_bytes()); // slot
        item2[12..20].copy_from_slice(&(1u64).to_le_bytes()); // count = 1
        item2[24..26].copy_from_slice(&(1u16).to_le_bytes()); // equipped = true
        item2[30..32].copy_from_slice(&(4u16).to_le_bytes()); // enchant = +4
        data.extend_from_slice(&item2);

        let packet = L2Packet::parse(0x11, &data);
        match packet {
            L2Packet::ItemList(il) => {
                assert_eq!(il.items.len(), 2);
                assert_eq!(il.items[0].object_id, 268435456);
                assert_eq!(il.items[0].item_id, 57);
                assert_eq!(il.items[0].count, 15000000);
                assert!(!il.items[0].equipped);

                assert_eq!(il.items[1].object_id, 268435457);
                assert_eq!(il.items[1].item_id, 72);
                assert_eq!(il.items[1].count, 1);
                assert_eq!(il.items[1].enchant_level, 4);
                assert!(il.items[1].equipped);
            }
            _ => panic!("Expected ItemList packet"),
        }
    }
}
