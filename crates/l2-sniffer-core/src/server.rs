//! Embedded REST and WebSocket API Server for live sniffer telemetry.

use std::net::SocketAddr;
use std::sync::Arc;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    http::StatusCode,
    response::Response,
    routing::get,
    Json, Router,
};
use futures_util::{SinkExt, StreamExt};
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

use crate::model::{
    AccountSession, BuffEffect, Character, CommissionItem, EinhasadProduct, InventoryItem,
    MarketState, PrivateStoreSession, SkillEntry, WorldExchangeItem,
};
use crate::state::CharacterTracker;

/// Shared application state for the web server.
#[derive(Clone)]
pub struct AppState {
    pub tracker: Arc<CharacterTracker>,
}

/// Creates the Axum router with all REST and WebSocket routes and CORS enabled.
pub fn create_router(tracker: Arc<CharacterTracker>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = AppState { tracker };

    Router::new()
        .route("/api/accounts", get(list_accounts))
        .route("/api/characters", get(list_characters))
        .route("/api/characters/{id}", get(get_character))
        .route("/api/characters/{id}/skills", get(get_character_skills))
        .route("/api/characters/{id}/buffs", get(get_character_buffs))
        .route("/api/characters/{id}/inventory", get(get_character_inventory))
        .route("/api/characters/{id}/warehouse", get(get_character_warehouse))
        .route("/api/markets", get(get_market_state))
        .route("/api/markets/stores", get(get_private_stores))
        .route("/api/markets/commission", get(get_commission_items))
        .route("/api/markets/world-exchange", get(get_world_exchange))
        .route("/api/markets/einhasad", get(get_einhasad_products))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(state)
}

/// Starts the embedded API server on the specified port.
pub async fn start_api_server(
    tracker: Arc<CharacterTracker>,
    port: u16,
) -> Result<(SocketAddr, tokio::task::JoinHandle<()>), std::io::Error> {
    let app = create_router(tracker);
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;

    info!("🚀 Sniffer Telemetry API Server listening on http://{}", local_addr);
    info!("📡 WebSocket stream active at ws://{}/ws", local_addr);

    let handle = tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            error!("API Server error: {e}");
        }
    });

    Ok((local_addr, handle))
}

// =================== REST Route Handlers ===================

async fn list_accounts(State(state): State<AppState>) -> Json<Vec<AccountSession>> {
    Json(state.tracker.get_accounts().await)
}

async fn list_characters(State(state): State<AppState>) -> Json<Vec<Character>> {
    Json(state.tracker.get_characters().await)
}

async fn get_character(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<Json<Character>, StatusCode> {
    if let Some(char_info) = state.tracker.get_character_by_id(id).await {
        Ok(Json(char_info))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_character_skills(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Json<Vec<SkillEntry>> {
    Json(state.tracker.get_character_skills(id).await)
}

async fn get_character_buffs(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Json<Vec<BuffEffect>> {
    Json(state.tracker.get_character_buffs(id).await)
}

async fn get_character_inventory(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Json<Vec<InventoryItem>> {
    Json(state.tracker.get_character_inventory(id).await)
}

async fn get_character_warehouse(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Json<Vec<InventoryItem>> {
    Json(state.tracker.get_character_warehouse(id).await)
}

async fn get_market_state(State(state): State<AppState>) -> Json<MarketState> {
    Json(state.tracker.get_market_state().await)
}

async fn get_private_stores(State(state): State<AppState>) -> Json<Vec<PrivateStoreSession>> {
    Json(state.tracker.get_private_stores().await)
}

async fn get_commission_items(State(state): State<AppState>) -> Json<Vec<CommissionItem>> {
    Json(state.tracker.get_commission_items().await)
}

async fn get_world_exchange(State(state): State<AppState>) -> Json<Vec<WorldExchangeItem>> {
    Json(state.tracker.get_world_exchange_items().await)
}

async fn get_einhasad_products(State(state): State<AppState>) -> Json<Vec<EinhasadProduct>> {
    Json(state.tracker.get_einhasad_products().await)
}

// =================== WebSocket Stream Handler ===================

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state.tracker))
}

async fn handle_socket(socket: WebSocket, tracker: Arc<CharacterTracker>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tracker.subscribe();

    // Spawn client reader (to detect disconnects / ping-pong)
    let mut read_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Close(_) = msg {
                break;
            }
        }
    });

    // Stream live events from broadcast channel to WebSocket
    let mut write_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&event) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // When either ends, abort the other
    tokio::select! {
        _ = (&mut read_task) => write_task.abort(),
        _ = (&mut write_task) => read_task.abort(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use http_body_util::BodyExt;
    use tower::ServiceExt;
    use l2_sniffer_protocol::{AuthLoginPacket, CharSelectInfoPacket, CharSelectSlot, L2Packet, UserInfoPacket};

    #[tokio::test]
    async fn test_api_routes() {
        let tracker = Arc::new(CharacterTracker::new());
        let client_addr: SocketAddr = "127.0.0.1:54321".parse().unwrap();

        // Populate mock state
        tracker.handle_packet_with_client(Some(client_addr), L2Packet::AuthLogin(AuthLoginPacket {
            account_name: "TestAccount".into(),
            session_key1: 1,
            session_key2: 2,
        })).await;

        tracker.handle_packet_with_client(Some(client_addr), L2Packet::CharSelectInfo(CharSelectInfoPacket {
            account_name: "TestAccount".into(),
            character_slots: vec![CharSelectSlot {
                name: "TestHero".into(),
                char_id: 5001,
                level: 80,
                class_id: 10,
                cur_hp: 4500.0,
                max_hp: 4500.0,
                ..Default::default()
            }],
        })).await;

        tracker.handle_packet_with_client(Some(client_addr), L2Packet::UserInfo(UserInfoPacket {
            object_id: 5001,
            name: "TestHero".into(),
            level: 80,
            cur_hp: 4500,
            max_hp: 4500,
            ..Default::default()
        })).await;

        let app = create_router(tracker);

        // Test GET /api/accounts
        let response = app.clone().oneshot(
            Request::builder().uri("/api/accounts").body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let accounts: Vec<AccountSession> = serde_json::from_slice(&body).unwrap();
        assert_eq!(accounts.len(), 1);
        assert_eq!(accounts[0].account_name, "TestAccount");

        // Test GET /api/characters
        let response = app.clone().oneshot(
            Request::builder().uri("/api/characters").body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let characters: Vec<Character> = serde_json::from_slice(&body).unwrap();
        assert_eq!(characters.len(), 1);
        assert_eq!(characters[0].name, "TestHero");

        // Test GET /api/characters/5001
        let response = app.clone().oneshot(
            Request::builder().uri("/api/characters/5001").body(Body::empty()).unwrap()
        ).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let char_info: Character = serde_json::from_slice(&body).unwrap();
        assert_eq!(char_info.object_id, 5001);
        assert_eq!(char_info.level, 80);
    }
}
