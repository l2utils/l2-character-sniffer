//! Lineage 2 Protocol Packet Payloads and Structs.

use byteorder::{LittleEndian, ReadBytesExt};
use serde::{Deserialize, Serialize};
use std::io::Cursor;

use crate::opcode::{ClientOpcode, ServerOpcode};

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
    pub char_id: u32,
    pub level: u32,
    pub class_id: u32,
    pub cur_hp: f64,
    pub max_hp: f64,
    pub cur_mp: f64,
    pub max_mp: f64,
    pub sp: u32,
    pub exp: u64,
    pub karma: u32,
    pub pk_kills: u32,
    pub pvp_kills: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct CharSelectInfoPacket {
    pub account_name: String,
    pub character_slots: Vec<CharSelectSlot>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserInfoPacket {
    pub object_id: u32,
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
        match ServerOpcode::from(opcode) {
            ServerOpcode::UserInfo => Self::parse_user_info(&mut cursor)
                .map(L2Packet::UserInfo)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            ServerOpcode::CharSelectInfo => Self::parse_char_select_info(&mut cursor)
                .map(L2Packet::CharSelectInfo)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            ServerOpcode::StatusUpdate => Self::parse_status_update(&mut cursor)
                .map(L2Packet::StatusUpdate)
                .unwrap_or(L2Packet::Raw {
                    opcode,
                    payload: payload.to_vec(),
                }),
            ServerOpcode::MoveToLocation => Self::parse_move_to_location(&mut cursor)
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
            _ => L2Packet::Raw {
                opcode,
                payload: payload.to_vec(),
            },
        }
    }

    fn parse_auth_login(r: &mut Cursor<&[u8]>) -> Result<AuthLoginPacket, std::io::Error> {
        // Try reading null-terminated UTF-16 string or ASCII string
        let account_name = if let Ok(s) = read_l2_string(r) {
            if !s.is_empty() && s.chars().all(|c| c.is_ascii_graphic() || c.is_ascii_whitespace()) {
                s
            } else {
                read_ascii_string(r)?
            }
        } else {
            read_ascii_string(r)?
        };

        let session_key1 = r.read_u32::<LittleEndian>().unwrap_or_default();
        let session_key2 = r.read_u32::<LittleEndian>().unwrap_or_default();

        Ok(AuthLoginPacket {
            account_name,
            session_key1,
            session_key2,
        })
    }

    fn parse_char_select_info(r: &mut Cursor<&[u8]>) -> Result<CharSelectInfoPacket, std::io::Error> {
        let count = r.read_u32::<LittleEndian>()?;
        let _max_slots = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _active = r.read_u8().unwrap_or_default();

        let mut character_slots = Vec::with_capacity(count as usize);

        for _ in 0..count {
            if let Ok(name) = read_l2_string(r) {
                let char_id = r.read_u32::<LittleEndian>().unwrap_or_default();
                let _login_name = read_l2_string(r).unwrap_or_default();
                let _session_id = r.read_u32::<LittleEndian>().unwrap_or_default();
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
                let sp = r.read_u32::<LittleEndian>().unwrap_or_default();
                let exp = r.read_u64::<LittleEndian>().unwrap_or_default();
                let level = r.read_u32::<LittleEndian>().unwrap_or_default();
                let karma = r.read_u32::<LittleEndian>().unwrap_or_default();
                let pk_kills = r.read_u32::<LittleEndian>().unwrap_or_default();
                let pvp_kills = r.read_u32::<LittleEndian>().unwrap_or_default();

                character_slots.push(CharSelectSlot {
                    name,
                    char_id,
                    level,
                    class_id,
                    cur_hp,
                    max_hp: cur_hp,
                    cur_mp,
                    max_mp: cur_mp,
                    sp,
                    exp,
                    karma,
                    pk_kills,
                    pvp_kills,
                });
            }
        }

        Ok(CharSelectInfoPacket {
            account_name: String::new(),
            character_slots,
        })
    }

    fn parse_user_info(r: &mut Cursor<&[u8]>) -> Result<UserInfoPacket, std::io::Error> {
        let x = r.read_i32::<LittleEndian>()?;
        let y = r.read_i32::<LittleEndian>()?;
        let z = r.read_i32::<LittleEndian>()?;
        let heading = r.read_i32::<LittleEndian>()?;
        let object_id = r.read_u32::<LittleEndian>()?;
        let name = read_l2_string(r)?;
        let _race = r.read_u32::<LittleEndian>().unwrap_or_default();
        let _sex = r.read_u32::<LittleEndian>().unwrap_or_default();
        let class_id = r.read_u32::<LittleEndian>().unwrap_or_default();
        let level = r.read_u32::<LittleEndian>().unwrap_or_default();
        let exp = r.read_u64::<LittleEndian>().unwrap_or_default();

        Ok(UserInfoPacket {
            object_id,
            name,
            title: String::new(),
            class_id,
            level,
            exp,
            x,
            y,
            z,
            heading,
            ..Default::default()
        })
    }

    fn parse_status_update(r: &mut Cursor<&[u8]>) -> Result<StatusUpdatePacket, std::io::Error> {
        let object_id = r.read_u32::<LittleEndian>()?;
        let count = r.read_u32::<LittleEndian>()?;
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

/// Reads a UTF-16LE null-terminated string as standard in Lineage 2 protocol.
pub fn read_l2_string(r: &mut Cursor<&[u8]>) -> Result<String, std::io::Error> {
    let mut u16_chars = Vec::new();
    loop {
        match r.read_u16::<LittleEndian>() {
            Ok(0) => break,
            Ok(ch) => u16_chars.push(ch),
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

/// Reads an ASCII null-terminated string.
pub fn read_ascii_string(r: &mut Cursor<&[u8]>) -> Result<String, std::io::Error> {
    let mut bytes = Vec::new();
    loop {
        match r.read_u8() {
            Ok(0) => break,
            Ok(b) => bytes.push(b),
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
