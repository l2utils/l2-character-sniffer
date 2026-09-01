//! Central state tracker for sniffing sessions.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use l2_sniffer_protocol::{L2Packet, UserInfoPacket};
use tokio::sync::{broadcast, RwLock};
use tracing::info;

use crate::event::SnifferEvent;
use crate::model::{Character, Location, Stats, Vitals};

/// Shared thread-safe tracker state.
#[derive(Clone)]
pub struct CharacterTracker {
    characters: Arc<RwLock<HashMap<u32, Character>>>,
    active_player_id: Arc<RwLock<Option<u32>>>,
    event_tx: broadcast::Sender<SnifferEvent>,
}

impl Default for CharacterTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl CharacterTracker {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(512);
        Self {
            characters: Arc::new(RwLock::new(HashMap::new())),
            active_player_id: Arc::new(RwLock::new(None)),
            event_tx,
        }
    }

    /// Subscribe to live state updates.
    pub fn subscribe(&self) -> broadcast::Receiver<SnifferEvent> {
        self.event_tx.subscribe()
    }

    /// Retrieve snapshot of currently tracked characters.
    pub async fn get_characters(&self) -> Vec<Character> {
        let chars = self.characters.read().await;
        chars.values().cloned().collect()
    }

    /// Retrieve active character if recognized.
    pub async fn get_active_character(&self) -> Option<Character> {
        let active_id = *self.active_player_id.read().await;
        if let Some(id) = active_id {
            let chars = self.characters.read().await;
            chars.get(&id).cloned()
        } else {
            None
        }
    }

    /// Ingest a parsed packet from the network capture stream.
    pub async fn handle_packet(&self, packet: L2Packet) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        match packet {
            L2Packet::UserInfo(info) => {
                self.handle_user_info(info, now).await;
            }
            L2Packet::StatusUpdate(update) => {
                self.handle_status_update(update.object_id, &update.attributes, now).await;
            }
            L2Packet::MoveToLocation(mv) => {
                self.handle_movement(mv.object_id, mv.target_x, mv.target_y, mv.target_z, now).await;
            }
            L2Packet::Raw { opcode, payload } => {
                let _ = self.event_tx.send(SnifferEvent::RawPacketReceived {
                    opcode,
                    length: payload.len(),
                });
            }
            _ => {}
        }
    }

    async fn handle_user_info(&self, info: UserInfoPacket, now: u64) {
        let mut chars = self.characters.write().await;
        let mut active = self.active_player_id.write().await;

        let char_entry = chars.entry(info.object_id).or_insert_with(Character::default);
        char_entry.object_id = info.object_id;
        char_entry.name = info.name.clone();
        char_entry.class_id = info.class_id;
        char_entry.level = info.level;
        char_entry.exp = info.exp;
        char_entry.sp = info.sp;
        char_entry.location = Location {
            x: info.x,
            y: info.y,
            z: info.z,
            heading: info.heading,
        };
        char_entry.vitals = Vitals {
            cur_hp: info.cur_hp,
            max_hp: info.max_hp,
            cur_mp: info.cur_mp,
            max_mp: info.max_mp,
            cur_cp: info.cur_cp,
            max_cp: info.max_cp,
        };
        char_entry.stats = Stats {
            p_atk: info.p_atk,
            p_def: info.p_def,
            m_atk: info.m_atk,
            m_def: info.m_def,
            p_atk_spd: info.p_atk_spd,
            m_atk_spd: info.m_atk_spd,
            run_spd: 0,
            walk_spd: 0,
        };
        char_entry.last_updated_epoch_ms = now;

        *active = Some(info.object_id);
        info!("Updated character info for: {} (ID: {})", info.name, info.object_id);

        let _ = self.event_tx.send(SnifferEvent::CharacterLoaded(char_entry.clone()));
    }

    async fn handle_status_update(
        &self,
        object_id: u32,
        attrs: &[l2_sniffer_protocol::StatusUpdateAttribute],
        now: u64,
    ) {
        let mut chars = self.characters.write().await;
        if let Some(char_entry) = chars.get_mut(&object_id) {
            for attr in attrs {
                match attr.attr_id {
                    1 => char_entry.level = attr.value,
                    2 => char_entry.exp = attr.value as u64,
                    3 => char_entry.stats.p_atk = attr.value,
                    9 => char_entry.vitals.cur_hp = attr.value,
                    10 => char_entry.vitals.max_hp = attr.value,
                    11 => char_entry.vitals.cur_mp = attr.value,
                    12 => char_entry.vitals.max_mp = attr.value,
                    13 => char_entry.sp = attr.value,
                    33 => char_entry.vitals.cur_cp = attr.value,
                    34 => char_entry.vitals.max_cp = attr.value,
                    _ => {}
                }
            }
            char_entry.last_updated_epoch_ms = now;

            let _ = self.event_tx.send(SnifferEvent::VitalsChanged {
                object_id,
                vitals: char_entry.vitals.clone(),
            });
        }
    }

    async fn handle_movement(&self, object_id: u32, x: i32, y: i32, z: i32, now: u64) {
        let mut chars = self.characters.write().await;
        if let Some(char_entry) = chars.get_mut(&object_id) {
            char_entry.location.x = x;
            char_entry.location.y = y;
            char_entry.location.z = z;
            char_entry.last_updated_epoch_ms = now;

            let _ = self.event_tx.send(SnifferEvent::LocationChanged {
                object_id,
                location: char_entry.location.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_sniffer_protocol::{StatusUpdateAttribute, StatusUpdatePacket, UserInfoPacket};

    #[tokio::test]
    async fn test_character_tracker_lifecycle() {
        let tracker = CharacterTracker::new();
        let mut rx = tracker.subscribe();

        // Ingest UserInfo
        let user_info = UserInfoPacket {
            object_id: 12345,
            name: "HeroKnight".to_string(),
            level: 76,
            cur_hp: 4500,
            max_hp: 4500,
            cur_mp: 1200,
            max_mp: 1200,
            x: 100,
            y: 200,
            z: -50,
            ..Default::default()
        };

        tracker.handle_packet(L2Packet::UserInfo(user_info)).await;

        let active = tracker.get_active_character().await;
        assert!(active.is_some());
        let hero = active.unwrap();
        assert_eq!(hero.name, "HeroKnight");
        assert_eq!(hero.level, 76);
        assert_eq!(hero.vitals.cur_hp, 4500);

        // Update status (e.g. HP dropped to 3000)
        let status = StatusUpdatePacket {
            object_id: 12345,
            attributes: vec![StatusUpdateAttribute { attr_id: 9, value: 3000 }],
        };
        tracker.handle_packet(L2Packet::StatusUpdate(status)).await;

        let updated = tracker.get_active_character().await.unwrap();
        assert_eq!(updated.vitals.cur_hp, 3000);

        // Verify event receiver received events
        let ev1 = rx.recv().await.unwrap();
        assert!(matches!(ev1, SnifferEvent::CharacterLoaded(_)));

        let ev2 = rx.recv().await.unwrap();
        assert!(matches!(ev2, SnifferEvent::VitalsChanged { object_id: 12345, .. }));
    }
}
