//! Central state tracker for multi-client and multi-account sniffing sessions.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use l2_sniffer_protocol::{AuthLoginPacket, CharSelectInfoPacket, L2Packet, UserInfoPacket};
use tokio::sync::{broadcast, RwLock};
use tracing::info;

use crate::event::SnifferEvent;
use crate::model::{AccountSession, Character, Location, Stats, Vitals};

/// Shared thread-safe tracker state across all client sessions and accounts.
#[derive(Clone)]
pub struct CharacterTracker {
    characters: Arc<RwLock<HashMap<u32, Character>>>,
    client_to_object: Arc<RwLock<HashMap<SocketAddr, u32>>>,
    client_to_account: Arc<RwLock<HashMap<SocketAddr, String>>>,
    accounts: Arc<RwLock<HashMap<String, AccountSession>>>,
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
            client_to_object: Arc::new(RwLock::new(HashMap::new())),
            client_to_account: Arc::new(RwLock::new(HashMap::new())),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            active_player_id: Arc::new(RwLock::new(None)),
            event_tx,
        }
    }

    /// Subscribe to live state updates.
    pub fn subscribe(&self) -> broadcast::Receiver<SnifferEvent> {
        self.event_tx.subscribe()
    }

    /// Retrieve snapshot of all currently tracked characters across all clients.
    pub async fn get_characters(&self) -> Vec<Character> {
        let chars = self.characters.read().await;
        chars.values().cloned().collect()
    }

    /// Retrieve snapshot of all detected account sessions.
    pub async fn get_accounts(&self) -> Vec<AccountSession> {
        let accs = self.accounts.read().await;
        accs.values().cloned().collect()
    }

    /// Retrieve character by client endpoint.
    pub async fn get_character_by_client(&self, client_addr: &SocketAddr) -> Option<Character> {
        let c_to_o = self.client_to_object.read().await;
        if let Some(obj_id) = c_to_o.get(client_addr) {
            let chars = self.characters.read().await;
            chars.get(obj_id).cloned()
        } else {
            None
        }
    }

    /// Registers a new game client connection and emits an event.
    pub async fn register_client_connection(&self, client_addr: SocketAddr, server_addr: SocketAddr) {
        info!("Client connected: {} -> {}", client_addr, server_addr);
        let _ = self.event_tx.send(SnifferEvent::ClientConnected {
            client_addr,
            server_addr,
        });
    }

    /// Unregisters a disconnected game client and emits a disconnect event.
    pub async fn unregister_client_connection(&self, client_addr: SocketAddr, reason: String) {
        info!("Client disconnected: {} ({})", client_addr, reason);
        let mut c_to_o = self.client_to_object.write().await;
        c_to_o.remove(&client_addr);
        let mut c_to_a = self.client_to_account.write().await;
        c_to_a.remove(&client_addr);

        let _ = self.event_tx.send(SnifferEvent::ClientDisconnected {
            client_addr,
            reason,
        });
    }

    /// Ingest a packet without client session context.
    pub async fn handle_packet(&self, packet: L2Packet) {
        self.handle_packet_with_client(None, packet).await;
    }

    /// Ingest a parsed packet from a specific client TCP stream.
    pub async fn handle_packet_with_client(&self, client_addr: Option<SocketAddr>, packet: L2Packet) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        match packet {
            L2Packet::AuthLogin(auth) => {
                self.handle_auth_login(client_addr, auth, now).await;
            }
            L2Packet::CharSelectInfo(info) => {
                self.handle_char_select_info(client_addr, info, now).await;
            }
            L2Packet::UserInfo(info) => {
                self.handle_user_info(client_addr, info, now).await;
            }
            L2Packet::StatusUpdate(update) => {
                self.handle_status_update(client_addr, update.object_id, &update.attributes, now).await;
            }
            L2Packet::MoveToLocation(mv) => {
                self.handle_movement(client_addr, mv.object_id, mv.target_x, mv.target_y, mv.target_z, now).await;
            }
            L2Packet::Raw { opcode, payload } => {
                let _ = self.event_tx.send(SnifferEvent::RawPacketReceived {
                    client_addr,
                    opcode,
                    length: payload.len(),
                });
            }
            _ => {}
        }
    }

    async fn handle_auth_login(&self, client_addr: Option<SocketAddr>, auth: AuthLoginPacket, now: u64) {
        if auth.account_name.is_empty() {
            return;
        }

        if let Some(addr) = client_addr {
            let mut c_to_a = self.client_to_account.write().await;
            c_to_a.insert(addr, auth.account_name.clone());

            let mut accs = self.accounts.write().await;
            let entry = accs.entry(auth.account_name.clone()).or_insert_with(|| AccountSession {
                account_name: auth.account_name.clone(),
                client_addr: addr.to_string(),
                character_roster: Vec::new(),
                active_character: None,
                last_seen_epoch_ms: now,
            });
            entry.client_addr = addr.to_string();
            entry.last_seen_epoch_ms = now;
        }

        info!("Account login detected: '{}' on {:?}", auth.account_name, client_addr);

        let _ = self.event_tx.send(SnifferEvent::AccountDetected {
            client_addr,
            account_name: auth.account_name,
        });
    }

    async fn handle_char_select_info(&self, client_addr: Option<SocketAddr>, info: CharSelectInfoPacket, now: u64) {
        let account_name = if !info.account_name.is_empty() {
            info.account_name.clone()
        } else if let Some(addr) = client_addr {
            let c_to_a = self.client_to_account.read().await;
            c_to_a.get(&addr).cloned().unwrap_or_default()
        } else {
            String::new()
        };

        if let Some(addr) = client_addr {
            if !account_name.is_empty() {
                let mut accs = self.accounts.write().await;
                let entry = accs.entry(account_name.clone()).or_insert_with(|| AccountSession {
                    account_name: account_name.clone(),
                    client_addr: addr.to_string(),
                    character_roster: Vec::new(),
                    active_character: None,
                    last_seen_epoch_ms: now,
                });
                entry.character_roster = info.character_slots.clone();
                entry.last_seen_epoch_ms = now;
            }
        }

        info!("Character selection screen for account '{}' with {} characters", account_name, info.character_slots.len());

        let _ = self.event_tx.send(SnifferEvent::AccountRosterLoaded {
            client_addr,
            account_name,
            characters: info.character_slots,
        });
    }

    async fn handle_user_info(&self, client_addr: Option<SocketAddr>, info: UserInfoPacket, now: u64) {
        let mut chars = self.characters.write().await;
        let mut active = self.active_player_id.write().await;

        let account_name = if let Some(addr) = client_addr {
            let mut c_to_o = self.client_to_object.write().await;
            c_to_o.insert(addr, info.object_id);

            let c_to_a = self.client_to_account.read().await;
            c_to_a.get(&addr).cloned()
        } else {
            None
        };

        let char_entry = chars.entry(info.object_id).or_insert_with(Character::default);
        char_entry.object_id = info.object_id;
        char_entry.account_name = account_name;
        char_entry.name = info.name.clone();
        char_entry.class_id = info.class_id;
        char_entry.level = info.level;
        char_entry.exp = info.exp;
        char_entry.sp = info.sp;
        char_entry.client_addr = client_addr.map(|a| a.to_string());
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
        info!("Updated character info for: {} (ID: {}, Client: {:?})", info.name, info.object_id, client_addr);

        let _ = self.event_tx.send(SnifferEvent::CharacterLoaded {
            client_addr,
            character: char_entry.clone(),
        });
    }

    async fn handle_status_update(
        &self,
        client_addr: Option<SocketAddr>,
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
                client_addr,
                object_id,
                vitals: char_entry.vitals.clone(),
            });
        }
    }

    async fn handle_movement(
        &self,
        client_addr: Option<SocketAddr>,
        object_id: u32,
        x: i32,
        y: i32,
        z: i32,
        now: u64,
    ) {
        let mut chars = self.characters.write().await;
        if let Some(char_entry) = chars.get_mut(&object_id) {
            char_entry.location.x = x;
            char_entry.location.y = y;
            char_entry.location.z = z;
            char_entry.last_updated_epoch_ms = now;

            let _ = self.event_tx.send(SnifferEvent::LocationChanged {
                client_addr,
                object_id,
                location: char_entry.location.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2_sniffer_protocol::{AuthLoginPacket, CharSelectSlot, UserInfoPacket};

    #[tokio::test]
    async fn test_multi_client_and_account_tracker() {
        let tracker = CharacterTracker::new();
        let mut rx = tracker.subscribe();

        let client1: SocketAddr = "192.168.1.50:60001".parse().unwrap();

        // 1. Account login
        let auth = AuthLoginPacket {
            account_name: "JasonAccount1".to_string(),
            session_key1: 12345,
            session_key2: 67890,
        };
        tracker.handle_packet_with_client(Some(client1), L2Packet::AuthLogin(auth)).await;

        let ev1 = rx.recv().await.unwrap();
        assert!(matches!(ev1, SnifferEvent::AccountDetected { .. }));

        // 2. Character Select Info
        let slot = CharSelectSlot {
            name: "HeroKnight".to_string(),
            char_id: 1001,
            level: 85,
            class_id: 90,
            cur_hp: 6000.0,
            max_hp: 6000.0,
            ..Default::default()
        };
        let select_info = CharSelectInfoPacket {
            account_name: "JasonAccount1".to_string(),
            character_slots: vec![slot],
        };
        tracker.handle_packet_with_client(Some(client1), L2Packet::CharSelectInfo(select_info)).await;

        let ev2 = rx.recv().await.unwrap();
        assert!(matches!(ev2, SnifferEvent::AccountRosterLoaded { .. }));

        // 3. Enter World UserInfo
        let char1 = UserInfoPacket {
            object_id: 1001,
            name: "HeroKnight".to_string(),
            level: 85,
            cur_hp: 6000,
            max_hp: 6000,
            ..Default::default()
        };
        tracker.handle_packet_with_client(Some(client1), L2Packet::UserInfo(char1)).await;

        let c = tracker.get_character_by_client(&client1).await.unwrap();
        assert_eq!(c.name, "HeroKnight");
        assert_eq!(c.account_name, Some("JasonAccount1".to_string()));
    }
}
