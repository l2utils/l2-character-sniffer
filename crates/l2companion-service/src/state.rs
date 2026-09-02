//! Central state tracker for multi-client and multi-account sniffing sessions.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use l2companion_protocol::{
    AuthLoginPacket, CharSelectInfoPacket, CommissionListPacket, EinhasadStorePacket,
    InventoryUpdatePacket, ItemListPacket, L2Packet, PrivateStorePacket, SkillListPacket,
    UserInfoPacket, WarehouseListPacket, WorldExchangeListPacket,
};
use tokio::sync::{broadcast, RwLock};
use tracing::info;

use crate::event::CompanionEvent;
use crate::model::{
    AccountSession, BuffEffect, Character, CommissionItem, EinhasadProduct, InventoryItem,
    Location, MarketState, PrivateStoreSession, SkillEntry, WorldExchangeItem,
};

/// Shared thread-safe tracker state across all client sessions and accounts.
#[derive(Clone)]
pub struct CharacterTracker {
    characters: Arc<RwLock<HashMap<u32, Character>>>,
    client_to_object: Arc<RwLock<HashMap<SocketAddr, u32>>>,
    client_to_account: Arc<RwLock<HashMap<SocketAddr, String>>>,
    accounts: Arc<RwLock<HashMap<String, AccountSession>>>,
    market: Arc<RwLock<MarketState>>,
    active_player_id: Arc<RwLock<Option<u32>>>,
    event_tx: broadcast::Sender<CompanionEvent>,
}

impl Default for CharacterTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl CharacterTracker {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(1024);
        Self {
            characters: Arc::new(RwLock::new(HashMap::new())),
            client_to_object: Arc::new(RwLock::new(HashMap::new())),
            client_to_account: Arc::new(RwLock::new(HashMap::new())),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            market: Arc::new(RwLock::new(MarketState::default())),
            active_player_id: Arc::new(RwLock::new(None)),
            event_tx,
        }
    }

    /// Subscribe to live state updates.
    pub fn subscribe(&self) -> broadcast::Receiver<CompanionEvent> {
        self.event_tx.subscribe()
    }

    /// Retrieve snapshot of all currently tracked characters across all clients.
    pub async fn get_characters(&self) -> Vec<Character> {
        let chars = self.characters.read().await;
        chars.values().cloned().collect()
    }

    /// Retrieve character by object ID.
    pub async fn get_character_by_id(&self, object_id: u32) -> Option<Character> {
        let chars = self.characters.read().await;
        chars.get(&object_id).cloned()
    }

    /// Retrieve character by name.
    pub async fn get_character_by_name(&self, name: &str) -> Option<Character> {
        let chars = self.characters.read().await;
        chars.values().find(|c| c.name.eq_ignore_ascii_case(name)).cloned()
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

    /// Retrieve character skills.
    pub async fn get_character_skills(&self, object_id: u32) -> Vec<SkillEntry> {
        let chars = self.characters.read().await;
        chars.get(&object_id).map(|c| c.skills.clone()).unwrap_or_default()
    }

    /// Retrieve character buffs.
    pub async fn get_character_buffs(&self, object_id: u32) -> Vec<BuffEffect> {
        let chars = self.characters.read().await;
        chars.get(&object_id).map(|c| c.buffs.clone()).unwrap_or_default()
    }

    /// Retrieve character inventory.
    pub async fn get_character_inventory(&self, object_id: u32) -> Vec<InventoryItem> {
        let chars = self.characters.read().await;
        chars.get(&object_id).map(|c| c.inventory.clone()).unwrap_or_default()
    }

    /// Retrieve character warehouse items.
    pub async fn get_character_warehouse(&self, object_id: u32) -> Vec<InventoryItem> {
        let chars = self.characters.read().await;
        chars.get(&object_id).map(|c| c.warehouse.clone()).unwrap_or_default()
    }

    /// Retrieve market state snapshot.
    pub async fn get_market_state(&self) -> MarketState {
        let m = self.market.read().await;
        m.clone()
    }

    /// Retrieve active private stores.
    pub async fn get_private_stores(&self) -> Vec<PrivateStoreSession> {
        let m = self.market.read().await;
        m.private_stores.values().cloned().collect()
    }

    /// Retrieve commission market listings.
    pub async fn get_commission_items(&self) -> Vec<CommissionItem> {
        let m = self.market.read().await;
        m.commission_items.clone()
    }

    /// Retrieve world exchange listings.
    pub async fn get_world_exchange_items(&self) -> Vec<WorldExchangeItem> {
        let m = self.market.read().await;
        m.world_exchange_items.clone()
    }

    /// Retrieve Einhasad store products.
    pub async fn get_einhasad_products(&self) -> Vec<EinhasadProduct> {
        let m = self.market.read().await;
        m.einhasad_products.clone()
    }

    /// Registers a new game client connection and emits an event.
    pub async fn register_client_connection(&self, client_addr: SocketAddr, server_addr: SocketAddr) {
        info!("Client connected: {} -> {}", client_addr, server_addr);
        let _ = self.event_tx.send(CompanionEvent::ClientConnected {
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

        let _ = self.event_tx.send(CompanionEvent::ClientDisconnected {
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
            L2Packet::ItemList(il) => {
                self.handle_item_list(client_addr, il, now).await;
            }
            L2Packet::InventoryUpdate(iu) => {
                self.handle_inventory_update(client_addr, iu, now).await;
            }
            L2Packet::WarehouseList(wh) => {
                self.handle_warehouse_list(client_addr, wh, now).await;
            }
            L2Packet::SkillList(sl) => {
                self.handle_skill_list(client_addr, sl, now).await;
            }
            L2Packet::AbnormalStatusUpdate(ab) => {
                self.handle_abnormal_status(client_addr, ab.buffs, now).await;
            }
            L2Packet::MagicEffectIcons(me) => {
                self.handle_abnormal_status(client_addr, me.buffs, now).await;
            }
            L2Packet::PrivateStore(ps) => {
                self.handle_private_store(client_addr, ps, now).await;
            }
            L2Packet::CommissionList(cl) => {
                self.handle_commission_list(cl, now).await;
            }
            L2Packet::WorldExchangeList(we) => {
                self.handle_world_exchange_list(we, now).await;
            }
            L2Packet::EinhasadStore(es) => {
                self.handle_einhasad_store(es, now).await;
            }
            L2Packet::MoveToLocation(mv) => {
                self.handle_movement(client_addr, mv.object_id, mv.target_x, mv.target_y, mv.target_z, now).await;
            }
            L2Packet::Raw { opcode, payload } => {
                let _ = self.event_tx.send(CompanionEvent::RawPacketReceived {
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

        let is_new = if let Some(addr) = client_addr {
            let mut c_to_a = self.client_to_account.write().await;
            if c_to_a.get(&addr) == Some(&auth.account_name) {
                return; // Already registered to this endpoint
            }
            c_to_a.insert(addr, auth.account_name.clone());

            let mut accs = self.accounts.write().await;
            let is_new_acc = !accs.contains_key(&auth.account_name);
            let entry = accs.entry(auth.account_name.clone()).or_insert_with(|| AccountSession {
                account_name: auth.account_name.clone(),
                client_addr: addr.to_string(),
                character_roster: Vec::new(),
                active_character: None,
                last_seen_epoch_ms: now,
            });
            entry.client_addr = addr.to_string();
            entry.last_seen_epoch_ms = now;
            is_new_acc
        } else {
            true
        };

        if is_new {
            info!("Account login detected: '{}' on {:?}", auth.account_name, client_addr);
            let _ = self.event_tx.send(CompanionEvent::AccountDetected {
                client_addr,
                account_name: auth.account_name,
            });
        }
    }

    async fn handle_char_select_info(&self, client_addr: Option<SocketAddr>, info: CharSelectInfoPacket, now: u64) {
        if info.character_slots.is_empty() {
            return;
        }

        let account_name = if !info.account_name.is_empty() {
            info.account_name.clone()
        } else if let Some(addr) = client_addr {
            let c_to_a = self.client_to_account.read().await;
            c_to_a.get(&addr).cloned().unwrap_or_else(|| "GameAccount".into())
        } else {
            "GameAccount".into()
        };

        if let Some(addr) = client_addr {
            if !info.account_name.is_empty() {
                let mut c_to_a = self.client_to_account.write().await;
                c_to_a.insert(addr, info.account_name.clone());
            }
        }

        let mut accs = self.accounts.write().await;
        let entry = accs.entry(account_name.clone()).or_insert_with(|| AccountSession {
            account_name: account_name.clone(),
            client_addr: client_addr.map(|a| a.to_string()).unwrap_or_default(),
            character_roster: Vec::new(),
            active_character: None,
            last_seen_epoch_ms: now,
        });

        entry.character_roster = info.character_slots.clone();
        entry.last_seen_epoch_ms = now;

        info!("Loaded roster for account '{}': {} characters", account_name, info.character_slots.len());

        let _ = self.event_tx.send(CompanionEvent::AccountRosterLoaded {
            client_addr,
            account_name,
            characters: info.character_slots,
        });
    }

    async fn handle_user_info(&self, client_addr: Option<SocketAddr>, info: UserInfoPacket, now: u64) {
        if info.name.is_empty() {
            return;
        }

        let mut chars = self.characters.write().await;
        let mut active = self.active_player_id.write().await;

        let account_name = if let Some(addr) = client_addr {
            let mut c_to_o = self.client_to_object.write().await;
            c_to_o.insert(addr, info.object_id);

            let mut c_to_a = self.client_to_account.write().await;
            if info.session_id > 0 && !c_to_a.contains_key(&addr) {
                let acc_id = format!("#{}", info.session_id);
                c_to_a.insert(addr, acc_id.clone());
                let _ = self.event_tx.send(CompanionEvent::AccountDetected {
                    client_addr,
                    account_name: acc_id,
                });
            }
            c_to_a.get(&addr).cloned()
        } else {
            None
        };

        let char_entry = chars.entry(info.object_id).or_insert_with(Character::default);
        char_entry.object_id = info.object_id;
        char_entry.account_name = account_name;
        char_entry.name = info.name.clone();
        if info.class_id > 0 { char_entry.class_id = info.class_id; }
        if info.level > 0 { char_entry.level = info.level; }
        if info.exp > 0 { char_entry.exp = info.exp; }
        if info.sp > 0 { char_entry.sp = info.sp; }
        char_entry.client_addr = client_addr.map(|a| a.to_string());
        char_entry.location = Location {
            x: info.x,
            y: info.y,
            z: info.z,
            heading: info.heading,
        };
        if info.cur_hp > 0 { char_entry.vitals.cur_hp = info.cur_hp; }
        if info.max_hp > 0 { char_entry.vitals.max_hp = info.max_hp; }
        if info.cur_mp > 0 { char_entry.vitals.cur_mp = info.cur_mp; }
        if info.max_mp > 0 { char_entry.vitals.max_mp = info.max_mp; }
        char_entry.last_updated_epoch_ms = now;

        *active = Some(info.object_id);
        info!("Updated player character: {} (ID: {}, Level: {}, HP: {}/{})",
            info.name, info.object_id, char_entry.level, char_entry.vitals.cur_hp, char_entry.vitals.max_hp);

        let _ = self.event_tx.send(CompanionEvent::CharacterLoaded {
            client_addr,
            character: char_entry.clone(),
        });
    }

    async fn handle_status_update(
        &self,
        client_addr: Option<SocketAddr>,
        object_id: u32,
        attrs: &[l2companion_protocol::StatusUpdateAttribute],
        now: u64,
    ) {
        if let Some(addr) = client_addr {
            let c_to_o = self.client_to_object.read().await;
            if let Some(&my_id) = c_to_o.get(&addr) {
                if object_id != my_id {
                    return; // Ignore other players / NPCs / mobs
                }
            }
        }

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

            let _ = self.event_tx.send(CompanionEvent::VitalsChanged {
                client_addr,
                object_id,
                vitals: char_entry.vitals.clone(),
            });
        }
    }

    async fn handle_item_list(&self, client_addr: Option<SocketAddr>, il: ItemListPacket, now: u64) {
        let object_id = self.resolve_client_object_id(client_addr).await;
        if let Some(obj_id) = object_id {
            let mut chars = self.characters.write().await;
            if let Some(c) = chars.get_mut(&obj_id) {
                c.inventory = il.items.clone();
                c.last_updated_epoch_ms = now;
            }
            let _ = self.event_tx.send(CompanionEvent::InventoryLoaded {
                client_addr,
                object_id: obj_id,
                items: il.items,
            });
        }
    }

    async fn handle_inventory_update(&self, client_addr: Option<SocketAddr>, iu: InventoryUpdatePacket, now: u64) {
        let object_id = self.resolve_client_object_id(client_addr).await;
        if let Some(obj_id) = object_id {
            let mut chars = self.characters.write().await;
            if let Some(c) = chars.get_mut(&obj_id) {
                for updated in &iu.items {
                    if let Some(pos) = c.inventory.iter().position(|i| i.object_id == updated.object_id) {
                        c.inventory[pos] = updated.clone();
                    } else {
                        c.inventory.push(updated.clone());
                    }
                }
                c.last_updated_epoch_ms = now;
            }
            let _ = self.event_tx.send(CompanionEvent::InventoryLoaded {
                client_addr,
                object_id: obj_id,
                items: iu.items,
            });
        }
    }

    async fn handle_warehouse_list(&self, client_addr: Option<SocketAddr>, wh: WarehouseListPacket, now: u64) {
        let object_id = self.resolve_client_object_id(client_addr).await.unwrap_or(0);
        let mut chars = self.characters.write().await;
        if let Some(c) = chars.get_mut(&object_id) {
            c.warehouse = wh.items.clone();
            c.last_updated_epoch_ms = now;
        }
        let _ = self.event_tx.send(CompanionEvent::WarehouseLoaded {
            client_addr,
            object_id,
            wh_type: wh.wh_type,
            player_adena: wh.player_adena,
            items: wh.items,
        });
    }

    async fn handle_skill_list(&self, client_addr: Option<SocketAddr>, sl: SkillListPacket, now: u64) {
        let object_id = self.resolve_client_object_id(client_addr).await;
        if let Some(obj_id) = object_id {
            let mut chars = self.characters.write().await;
            if let Some(c) = chars.get_mut(&obj_id) {
                c.skills = sl.skills.clone();
                c.last_updated_epoch_ms = now;
            }
            let _ = self.event_tx.send(CompanionEvent::SkillsUpdated {
                client_addr,
                object_id: obj_id,
                skills: sl.skills,
            });
        }
    }

    async fn handle_abnormal_status(&self, client_addr: Option<SocketAddr>, buffs: Vec<BuffEffect>, now: u64) {
        let object_id = self.resolve_client_object_id(client_addr).await;
        if let Some(obj_id) = object_id {
            let mut chars = self.characters.write().await;
            if let Some(c) = chars.get_mut(&obj_id) {
                c.buffs = buffs.clone();
                c.last_updated_epoch_ms = now;
            }
            let _ = self.event_tx.send(CompanionEvent::BuffsUpdated {
                client_addr,
                object_id: obj_id,
                buffs,
            });
        }
    }

    async fn handle_private_store(&self, client_addr: Option<SocketAddr>, ps: PrivateStorePacket, now: u64) {
        let seller_name = {
            let chars = self.characters.read().await;
            chars.get(&ps.seller_object_id).map(|c| c.name.clone())
        };

        let session = PrivateStoreSession {
            seller_object_id: ps.seller_object_id,
            seller_name,
            store_type: ps.store_type,
            store_title: ps.store_title,
            items: ps.items,
            last_seen_epoch_ms: now,
        };

        let mut m = self.market.write().await;
        m.private_stores.insert(session.seller_object_id, session.clone());

        let _ = self.event_tx.send(CompanionEvent::PrivateStoreUpdated {
            client_addr,
            store: session,
        });
    }

    async fn handle_commission_list(&self, cl: CommissionListPacket, _now: u64) {
        let mut m = self.market.write().await;
        m.commission_items = cl.items.clone();
        let _ = self.event_tx.send(CompanionEvent::CommissionMarketUpdated {
            items: cl.items,
        });
    }

    async fn handle_world_exchange_list(&self, we: WorldExchangeListPacket, _now: u64) {
        let mut m = self.market.write().await;
        m.world_exchange_items = we.items.clone();
        let _ = self.event_tx.send(CompanionEvent::WorldExchangeUpdated {
            items: we.items,
        });
    }

    async fn handle_einhasad_store(&self, es: EinhasadStorePacket, _now: u64) {
        let mut m = self.market.write().await;
        m.einhasad_products = es.products.clone();
        let _ = self.event_tx.send(CompanionEvent::EinhasadStoreUpdated {
            products: es.products,
        });
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
        if let Some(addr) = client_addr {
            let c_to_o = self.client_to_object.read().await;
            if let Some(&my_id) = c_to_o.get(&addr) {
                if object_id != my_id {
                    return; // Ignore other entities
                }
            }
        }

        let mut chars = self.characters.write().await;
        if let Some(char_entry) = chars.get_mut(&object_id) {
            char_entry.location.x = x;
            char_entry.location.y = y;
            char_entry.location.z = z;
            char_entry.last_updated_epoch_ms = now;

            let _ = self.event_tx.send(CompanionEvent::LocationChanged {
                client_addr,
                object_id,
                location: char_entry.location.clone(),
            });
        }
    }

    async fn resolve_client_object_id(&self, client_addr: Option<SocketAddr>) -> Option<u32> {
        if let Some(addr) = client_addr {
            let c_to_o = self.client_to_object.read().await;
            if let Some(&id) = c_to_o.get(&addr) {
                return Some(id);
            }
        }
        let active = self.active_player_id.read().await;
        *active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use l2companion_protocol::{AuthLoginPacket, CharSelectSlot, ItemInfo, SkillEntry, UserInfoPacket};

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
        assert!(matches!(ev1, CompanionEvent::AccountDetected { .. }));

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
        assert!(matches!(ev2, CompanionEvent::AccountRosterLoaded { .. }));

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

        // 4. Skills update
        let sl = SkillListPacket {
            skills: vec![SkillEntry {
                skill_id: 1069,
                level: 15,
                sub_level: 0,
                is_passive: false,
                is_disabled: false,
                enchant_type: 0,
            }],
        };
        tracker.handle_packet_with_client(Some(client1), L2Packet::SkillList(sl)).await;
        let skills = tracker.get_character_skills(1001).await;
        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].skill_id, 1069);

        // 5. Inventory update
        let il = ItemListPacket {
            show_window: true,
            items: vec![ItemInfo {
                object_id: 2001,
                item_id: 57,
                count: 1000000,
                ..Default::default()
            }],
        };
        tracker.handle_packet_with_client(Some(client1), L2Packet::ItemList(il)).await;
        let inv = tracker.get_character_inventory(1001).await;
        assert_eq!(inv.len(), 1);
        assert_eq!(inv[0].item_id, 57);
    }
}
