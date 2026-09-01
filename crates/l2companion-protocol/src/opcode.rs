//! Lineage 2 Protocol Packet Opcodes.

use serde::{Deserialize, Serialize};

/// Known Server-to-Client packet opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServerOpcode {
    Die,
    Revive,
    Attack,
    CharInfo,
    UserInfo,
    DeleteObject,
    ItemAction,
    GetItem,
    StatusUpdate,
    CharSelectInfo,
    NpcInfo,
    ItemList,
    TargetSelected,
    TargetUnselected,
    AutoAttackStart,
    AutoAttackStop,
    ChangeMoveType,
    ChangeWaitType,
    StopMove,
    MagicSkillUse,
    MagicSkillCanceled,
    CreatureSay,
    EquipUpdate,
    SkillList,
    SystemMessage,
    RestartResponse,
    MoveToLocation,
    ValidateLocation,
    PartySmallWindowAll,
    PartySmallWindowAdd,
    PartySmallWindowDelete,
    PartySmallWindowUpdate,
    KeyPacket,
    Unknown(u8),
}

/// Known Client-to-Server packet opcodes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClientOpcode {
    ProtocolVersion,
    AuthLogin,
    CharacterSelect,
    RequestEnterWorld,
    RequestRestart,
    Action,
    MoveBackwardToLocation,
    UseItem,
    Say2,
    Unknown(u8),
}

impl From<u8> for ServerOpcode {
    fn from(op: u8) -> Self {
        match op {
            0x00 => ServerOpcode::Die,
            0x01 => ServerOpcode::MoveToLocation,
            0x03 => ServerOpcode::CharInfo,
            0x04 => ServerOpcode::UserInfo,
            0x05 => ServerOpcode::Attack,
            0x08 => ServerOpcode::DeleteObject,
            0x09 => ServerOpcode::CharSelectInfo,
            0x0c => ServerOpcode::ItemAction,
            0x0d => ServerOpcode::GetItem,
            0x0e => ServerOpcode::StatusUpdate,
            0x16 => ServerOpcode::NpcInfo,
            0x1b => ServerOpcode::ItemList,
            0x1f => ServerOpcode::CharSelectInfo,
            0x23 => ServerOpcode::TargetSelected,
            0x24 => ServerOpcode::TargetUnselected,
            0x25 => ServerOpcode::AutoAttackStart,
            0x26 => ServerOpcode::AutoAttackStop,
            0x28 => ServerOpcode::ChangeMoveType,
            0x29 => ServerOpcode::ChangeWaitType,
            0x47 => ServerOpcode::StopMove,
            0x48 => ServerOpcode::MagicSkillUse,
            0x49 => ServerOpcode::MagicSkillCanceled,
            0x4a => ServerOpcode::CreatureSay,
            0x4b => ServerOpcode::EquipUpdate,
            0x4e => ServerOpcode::PartySmallWindowAll,
            0x4f => ServerOpcode::PartySmallWindowAdd,
            0x50 => ServerOpcode::PartySmallWindowDelete,
            0x51 => ServerOpcode::PartySmallWindowUpdate,
            0x58 => ServerOpcode::SkillList,
            0x61 => ServerOpcode::ValidateLocation,
            0x64 => ServerOpcode::SystemMessage,
            0x6f => ServerOpcode::RestartResponse,
            other => ServerOpcode::Unknown(other),
        }
    }
}

impl From<u8> for ClientOpcode {
    fn from(op: u8) -> Self {
        match op {
            0x00 => ClientOpcode::ProtocolVersion,
            0x08 => ClientOpcode::AuthLogin,
            0x0d => ClientOpcode::CharacterSelect,
            0x03 => ClientOpcode::RequestEnterWorld,
            0x2b => ClientOpcode::AuthLogin,
            0x04 => ClientOpcode::Action,
            0x0f => ClientOpcode::MoveBackwardToLocation,
            0x19 => ClientOpcode::UseItem,
            0x49 => ClientOpcode::Say2,
            other => ClientOpcode::Unknown(other),
        }
    }
}
