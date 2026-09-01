//! Lineage 2 Protocol Packet Payloads and Structs.

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::opcode::ClientOpcode;

/// Parsed Lineage 2 packet enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum L2Packet {
    AuthLogin(AuthLoginPacket),
    CharSelectInfo(CharSelectInfoPacket),
    UserInfo(UserInfoPacket),
    StatusUpdate(StatusUpdatePacket),
    ItemList(ItemListPacket),
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemInfo {
    pub object_id: u32,
    pub item_id: u32,
    pub count: u64,
    pub item_type: u16,
    pub equipped: bool,
    pub enchant_level: u16,
    pub custom_type1: u16,
    pub is_augmented: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ItemListPacket {
    pub show_window: bool,
    pub items: Vec<ItemInfo>,
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
            // Modern Retail CharSelected / User Entrance (Opcode 0x0B)
            0x0b => Self::parse_char_selected_retail(&mut cursor)
                .map(L2Packet::UserInfo)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            // Modern Retail CharSelectInfo / Roster (Opcode 0x09)
            0x09 => Self::parse_char_select_info_retail(&mut cursor, payload)
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
    fn parse_char_select_info_retail(r: &mut Cursor<&[u8]>, raw: &[u8]) -> Result<CharSelectInfoPacket, std::io::Error> {
        if raw.len() < 16 {
            return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Packet too short"));
        }
        let count = r.read_u32::<LittleEndian>()?;
        if count == 0 || count > 50 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid character count"));
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
                if name.is_empty() { break; }
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
                let exp_percent = if exp_pct_raw <= 1.0 { exp_pct_raw * 100.0 } else { exp_pct_raw };
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
                        if candidate_pos + 4 < raw.len() && raw[candidate_pos + 1] == 0 && raw[candidate_pos].is_ascii_alphanumeric() {
                            let mut test_r = Cursor::new(&raw[candidate_pos..]);
                            if let Ok(test_name) = read_l2_string(&mut test_r) {
                                if test_name.len() >= 2 && test_name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                                    let _test_id = test_r.read_u32::<LittleEndian>().unwrap_or_default();
                                    if let Ok(test_acc) = read_l2_string(&mut test_r) {
                                        if test_acc == account_name || (test_acc.len() >= 2 && test_acc.chars().all(|c| c.is_ascii_alphanumeric())) {
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
        match ClientOpcode::from(opcode) {
            ClientOpcode::AuthLogin => Self::parse_auth_login(&mut cursor)
                .map(L2Packet::AuthLogin)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            _ => {
                // Check for Client Extended Opcode 0xD0
                if opcode == 0xd0 && payload.len() >= 2 {
                    let sub_op = u16::from_le_bytes([payload[0], payload[1]]);
                    let mut sub_cursor = Cursor::new(&payload[2..]);
                    if sub_op == 0x0240 || sub_op == 0x0008 || sub_op == 0x002b {
                        if let Ok(auth) = Self::parse_auth_login(&mut sub_cursor) {
                            return L2Packet::AuthLogin(auth);
                        }
                    }
                }
                L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }
            }
        }
    }

    /// Parses modern retail CharSelected (Opcode 0x0B)
    fn parse_char_selected_retail(r: &mut Cursor<&[u8]>) -> Result<UserInfoPacket, std::io::Error> {
        let name = read_l2_string(r)?;
        let object_id = r.read_u32::<LittleEndian>()?;
        let _padding = r.read_u16::<LittleEndian>().unwrap_or_default();
        let session_id = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _clan_id = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _builder = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _sex = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _race = r.read_u32::<LittleEndian>().unwrap_or_default();
        let class_id = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _active = r.read_u32::<LittleEndian>().unwrap_or_default();
        let x = r.read_i32::<LittleEndian>().unwrap_or_default();
        let y = r.read_i32::<LittleEndian>().unwrap_or_default();
        let z = r.read_i32::<LittleEndian>().unwrap_or_default();
        let cur_hp = r.read_f64::<LittleEndian>().map(|v| v as u32).unwrap_or_default();
        let cur_mp = r.read_f64::<LittleEndian>().map(|v| v as u32).unwrap_or_default();
        let sp = r.read_u64::<LittleEndian>().map(|v| v as u32).unwrap_or_default();
        let exp = r.read_u64::<LittleEndian>().unwrap_or_default();
        let level = r.read_u32::<LittleEndian>().unwrap_or_default();

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
        let session_key1 = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _token_len = r.read_u16::<LittleEndian>().unwrap_or_default();
        let raw_name = if let Ok(s) = read_ascii_string(r) {
            s
        } else {
            read_l2_string(r).unwrap_or_default()
        };
        let session_key2 = r.read_u32::<LittleEndian>().unwrap_or_default();

        let account_name = if session_key1 > 0 && (raw_name.is_empty() || raw_name.len() > 25) {
            format!("#{}", session_key1)
        } else if !raw_name.is_empty() {
            raw_name
        } else {
            format!("#{}", session_key1)
        };

        Ok(AuthLoginPacket {
            account_name,
            session_key1,
            session_key2,
        })
    }

    fn parse_status_update(r: &mut Cursor<&[u8]>) -> Result<StatusUpdatePacket, std::io::Error> {
        let object_id = r.read_u32::<LittleEndian>()?;
        let count = r.read_u32::<LittleEndian>()?;
        if count > 200 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Too many status attributes"));
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

    fn parse_move_to_location(r: &mut Cursor<&[u8]>) -> Result<MoveToLocationPacket, std::io::Error> {
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
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "String too long"));
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
    String::from_utf16(&u16_chars).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Reads an ASCII null-terminated string with length safeguard.
pub fn read_ascii_string(r: &mut Cursor<&[u8]>) -> Result<String, std::io::Error> {
    let mut bytes = Vec::new();
    let mut count = 0;
    loop {
        if count > 1024 {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "String too long"));
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
