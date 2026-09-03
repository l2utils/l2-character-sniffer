//! GraphQL Schema, QueryRoot, and SubscriptionRoot implementation.

use std::sync::Arc;
use async_graphql::{Context, EmptyMutation, Object, Schema, Subscription};
use futures_util::Stream;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

use crate::event::CompanionEvent;
use crate::model::{
    AccountSession, BuffEffect, Character, CommissionItem, EinhasadProduct, InventoryItem,
    MarketState, PrivateStoreSession, SkillEntry, Vitals, WorldExchangeItem,
};
use crate::state::CharacterTracker;

pub type AppSchema = Schema<QueryRoot, EmptyMutation, SubscriptionRoot>;

/// Creates and configures the GraphQL schema with shared tracker state.
pub fn build_schema(tracker: Arc<CharacterTracker>) -> AppSchema {
    Schema::build(QueryRoot, EmptyMutation, SubscriptionRoot)
        .data(tracker)
        .finish()
}

pub struct QueryRoot;

#[Object]
impl QueryRoot {
    /// Retrieve all detected account sessions and character rosters.
    async fn accounts(&self, ctx: &Context<'_>) -> Vec<AccountSession> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        tracker.get_accounts().await
    }

    /// Retrieve all currently tracked in-world active characters.
    async fn characters(&self, ctx: &Context<'_>) -> Vec<Character> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        tracker.get_characters().await
    }

    /// Lookup a single character by object ID or character name.
    async fn character(
        &self,
        ctx: &Context<'_>,
        id: Option<u32>,
        name: Option<String>,
    ) -> Option<Character> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        if let Some(obj_id) = id {
            tracker.get_character_by_id(obj_id).await
        } else if let Some(n) = name {
            tracker.get_character_by_name(&n).await
        } else {
            None
        }
    }

    /// Retrieve skills learned by a character.
    async fn skills(&self, ctx: &Context<'_>, character_id: u32) -> Vec<SkillEntry> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        tracker.get_character_skills(character_id).await
    }

    /// Retrieve active buff effects on a character.
    async fn buffs(&self, ctx: &Context<'_>, character_id: u32) -> Vec<BuffEffect> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        tracker.get_character_buffs(character_id).await
    }

    /// Retrieve inventory bag and equipped gear of a character.
    async fn inventory(&self, ctx: &Context<'_>, character_id: u32) -> Vec<InventoryItem> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        tracker.get_character_inventory(character_id).await
    }

    /// Retrieve personal warehouse storage items of a character.
    async fn warehouse(&self, ctx: &Context<'_>, character_id: u32) -> Vec<InventoryItem> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        tracker.get_character_warehouse(character_id).await
    }

    /// Retrieve full marketplace snapshot (private shops, auction, world exchange, einhasad store).
    async fn market(&self, ctx: &Context<'_>) -> MarketState {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        tracker.get_market_state().await
    }

    /// Retrieve active player buy/sell/manufacture stores.
    async fn private_stores(&self, ctx: &Context<'_>) -> Vec<PrivateStoreSession> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        tracker.get_private_stores().await
    }

    /// Retrieve auction house commission listings.
    async fn commission_items(&self, ctx: &Context<'_>) -> Vec<CommissionItem> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        tracker.get_commission_items().await
    }

    /// Retrieve World Exchange trade listings.
    async fn world_exchange_items(&self, ctx: &Context<'_>) -> Vec<WorldExchangeItem> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        tracker.get_world_exchange_items().await
    }

    /// Retrieve Einhasad Gold Coin store offerings.
    async fn einhasad_products(&self, ctx: &Context<'_>) -> Vec<EinhasadProduct> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        tracker.get_einhasad_products().await
    }
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Live push stream of all telemetry events emitted (JSON serialized).
    async fn events(&self, ctx: &Context<'_>) -> impl Stream<Item = String> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        let rx = tracker.subscribe();
        BroadcastStream::new(rx).filter_map(|msg| match msg {
            Ok(event) => serde_json::to_string(&event).ok(),
            Err(_) => None,
        })
    }

    /// Stream live HP/MP changes for a given character object ID.
    async fn character_vitals(
        &self,
        ctx: &Context<'_>,
        object_id: Option<u32>,
    ) -> impl Stream<Item = Vitals> {
        let tracker = ctx.data_unchecked::<Arc<CharacterTracker>>();
        let rx = tracker.subscribe();
        BroadcastStream::new(rx).filter_map(move |msg| match msg {
            Ok(CompanionEvent::VitalsChanged {
                object_id: ev_obj,
                vitals,
                ..
            }) => {
                if let Some(target_id) = object_id {
                    if target_id == ev_obj {
                        Some(vitals)
                    } else {
                        None
                    }
                } else {
                    Some(vitals)
                }
            }
            _ => None,
        })
    }
}
