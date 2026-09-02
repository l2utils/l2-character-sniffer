//! # l2companion CLI
//!
//! Command line tool to inspect network interfaces, capture packets, and monitor multi-client character statistics live.

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use anyhow::Result;
use clap::{Parser, Subcommand};
use l2companion_capture::{list_devices, SessionMessage, CaptureBuilder};
use l2companion_service::{start_api_server, CharacterTracker, CompanionEvent};
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(name = "l2companion", version, about = "Lineage 2 Character Data Telemetry & Telemetry Server CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all network interfaces available for packet capture
    Devices,
    /// Start sniffing network packets (live or offline pcap) and display character updates
    Sniff {
        /// Network interface name to capture on (default: auto-detect)
        #[arg(short, long)]
        device: Option<String>,

        /// Read from offline pcap/pcapng file instead of live interface
        #[arg(short, long)]
        pcap: Option<String>,

        /// Custom BPF filter (default: "tcp port 7777 or tcp port 2106")
        #[arg(short, long)]
        filter: Option<String>,

        /// Port to start the REST and WebSocket telemetry API server on (e.g. 3000)
        #[arg(long)]
        port: Option<u16>,
    },
    /// Replay and analyze an offline pcap/pcapng capture file
    Analyze {
        /// Path to the .pcap or .pcapng file
        #[arg(default_value = "captures/l2-multi-client.pcapng")]
        path: String,

        /// Port to start the REST and WebSocket telemetry API server on (e.g. 3000)
        #[arg(long)]
        port: Option<u16>,
    },
    /// Run as a persistent background capture & REST/WebSocket API server daemon
    Serve {
        /// Port to run the telemetry API server on
        #[arg(long, default_value_t = 3000)]
        port: u16,

        /// Network interface name to capture on (default: auto-detect)
        #[arg(short, long)]
        device: Option<String>,

        /// Read from offline pcap/pcapng file
        #[arg(short, long)]
        pcap: Option<String>,

        /// Custom BPF filter
        #[arg(short, long)]
        filter: Option<String>,
    },
}

fn init_windows_dll_path() {
    #[cfg(target_os = "windows")]
    unsafe {
        #[link(name = "kernel32")]
        extern "system" {
            fn SetDllDirectoryA(lpPathName: *const u8) -> i32;
        }
        let npcap_path = b"C:\\Windows\\System32\\Npcap\0";
        SetDllDirectoryA(npcap_path.as_ptr());
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    init_windows_dll_path();
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Devices => {
            println!("Enumerating network capture interfaces:\n");
            match list_devices() {
                Ok(devices) => {
                    if devices.is_empty() {
                        println!("No network interfaces found. (Is Npcap installed and running?)");
                    }
                    for (i, dev) in devices.into_iter().enumerate() {
                        println!("{}. Device: {}", i + 1, dev.name);
                        if let Some(desc) = dev.description {
                            println!("   Description: {}", desc);
                        }
                        if !dev.addresses.is_empty() {
                            println!("   IP Addresses: {}", dev.addresses.join(", "));
                        }
                        println!();
                    }
                }
                Err(e) => {
                    eprintln!("Error listing capture devices: {e}");
                }
            }
        }
        Commands::Sniff {
            device,
            pcap,
            filter,
            port,
        } => {
            run_capture_session(device, pcap, filter, port).await?;
        }
        Commands::Analyze { path, port } => {
            run_capture_session(None, Some(path), None, port).await?;
        }
        Commands::Serve {
            port,
            device,
            pcap,
            filter,
        } => {
            println!("Starting Lineage 2 Telemetry Daemon & Web Service on port {port}...");
            run_capture_session(device, pcap, filter, Some(port)).await?;
        }
    }

    Ok(())
}

async fn run_capture_session(
    device: Option<String>,
    pcap: Option<String>,
    filter: Option<String>,
    port: Option<u16>,
) -> Result<()> {
    println!("Initializing Lineage 2 Telemetry...");
    let mut builder = CaptureBuilder::new();

    if let Some(d) = device {
        builder = builder.device(d);
    }
    if let Some(p) = pcap {
        builder = builder.pcap_file(p);
    }
    if let Some(f) = filter {
        builder = builder.filter(f);
    }

    let session = builder.build()?;
    println!("📡 Capture Source: {}", session.source_description());

    let tracker = Arc::new(CharacterTracker::new());
    let mut event_rx = tracker.subscribe();

    // Start API server if port is provided
    let _api_handle = if let Some(p) = port {
        let (addr, handle) = start_api_server(tracker.clone(), p).await?;
        println!("🚀 REST API and WebSocket live at http://{}", addr);
        Some(handle)
    } else {
        None
    };

    let (tx, mut rx) = mpsc::channel::<SessionMessage>(4096);
    let worker_handle = session.spawn_worker(tx);

    // Track per-client packet stats
    let tracker_clone = tracker.clone();
    let ingestion_handle = tokio::spawn(async move {
        let mut total_packets = 0u64;
        let mut client_stats: HashMap<SocketAddr, u64> = HashMap::new();

        while let Some(msg) = rx.recv().await {
            match msg {
                SessionMessage::ClientConnected { client_addr, server_addr } => {
                    tracker_clone.register_client_connection(client_addr, server_addr).await;
                }
                SessionMessage::ClientDisconnected { client_addr, reason } => {
                    tracker_clone.unregister_client_connection(client_addr, reason).await;
                }
                SessionMessage::Packet(sp) => {
                    total_packets += 1;
                    *client_stats.entry(sp.client_addr).or_insert(0) += 1;
                    tracker_clone.handle_packet_with_client(Some(sp.client_addr), sp.packet).await;
                }
            }
        }
        (total_packets, client_stats)
    });

    println!("Capture session running. Waiting for game clients / packets...\n");

    let display_handle = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match event {
                CompanionEvent::ClientConnected { client_addr, server_addr } => {
                    println!("✨ [CLIENT CONNECTED]    Game client detected: {} <-> Server {}", client_addr, server_addr);
                }
                CompanionEvent::ClientDisconnected { client_addr, reason } => {
                    println!("🔴 [CLIENT DISCONNECTED] Client session closed: {} ({})", client_addr, reason);
                }
                CompanionEvent::AccountDetected { client_addr, account_name } => {
                    let client_str = client_addr.map(|a| a.to_string()).unwrap_or_else(|| "-".into());
                    println!("👤 [ACCOUNT DETECTED]    Account: \"{}\" on [{}]", account_name, client_str);
                }
                CompanionEvent::AccountRosterLoaded { client_addr, account_name, characters } => {
                    let client_str = client_addr.map(|a| a.to_string()).unwrap_or_else(|| "-".into());
                    println!("\n📋 [ACCOUNT ROSTER] Account: \"{}\" [{}] ({} characters):", account_name, client_str, characters.len());
                    println!("┌──────┬────────────────┬───────┬──────────┬───────────────┬─────────┬──────────────┬────────────┬──────┬──────────────────┐");
                    println!("│ Slot │ Name           │ Level │ Class ID │ HP (Cur/Max)  │ EXP %   │ SP           │ Reputation │ VP   │ Last Login       │");
                    println!("├──────┼────────────────┼───────┼──────────┼───────────────┼─────────┼──────────────┼────────────┼──────┼──────────────────┤");
                    for (i, c) in characters.iter().enumerate() {
                        let hp_str = format!("{:.0}/{:.0}", c.cur_hp, c.max_hp);
                        let exp_str = format!("{:>6.2}%", c.exp_percent);
                        let sp_str = format_number(c.sp);
                        let rep_str = format!("{}", c.reputation);
                        let vp_str = format!("{}%", c.vitality);
                        let last_login_str = if c.last_access > 0 {
                            format_unix_timestamp(c.last_access)
                        } else {
                            "-".into()
                        };
                        println!("│ {:<4} │ {:<14} │ {:<5} │ {:<8} │ {:<13} │ {:<7} │ {:<12} │ {:<10} │ {:<4} │ {:<16} │",
                            i + 1, c.name, c.level, c.class_id, hp_str, exp_str, sp_str, rep_str, vp_str, last_login_str);
                    }
                    println!("└──────┴────────────────┴───────┴──────────┴───────────────┴─────────┴──────────────┴────────────┴──────┴──────────────────┘\n");
                }
                CompanionEvent::CharacterLoaded { client_addr, character } => {
                    let client_str = client_addr.map(|a| a.to_string()).unwrap_or_else(|| "Unknown".into());
                    let acc_str = character.account_name.as_deref().map(|a| format!("Account: \"{}\" | ", a)).unwrap_or_default();
                    println!("👤 [CHARACTER] Client: {:<21} | {}Name: {:<16} | Level: {:<3} | Class ID: {:<3} | HP: {}/{}",
                        client_str, acc_str, character.name, character.level, character.class_id, character.vitals.cur_hp, character.vitals.max_hp);
                }
                CompanionEvent::VitalsChanged { client_addr, object_id, vitals } => {
                    let client_str = client_addr.map(|a| a.to_string()).unwrap_or_else(|| "-".into());
                    println!("❤️ [VITALS]    Client: {:<21} | Obj {}: HP: {}/{} | MP: {}/{}",
                        client_str, object_id, vitals.cur_hp, vitals.max_hp, vitals.cur_mp, vitals.max_mp);
                }
                CompanionEvent::SkillsUpdated { client_addr, object_id, skills } => {
                    let client_str = client_addr.map(|a| a.to_string()).unwrap_or_else(|| "-".into());
                    let passives = skills.iter().filter(|s| s.is_passive).count();
                    let actives = skills.len() - passives;
                    println!("🔮 [SKILLS]    Client: {:<21} | Obj {}: Loaded {} skills ({} active, {} passive)",
                        client_str, object_id, skills.len(), actives, passives);
                }
                CompanionEvent::BuffsUpdated { client_addr, object_id, buffs } => {
                    let client_str = client_addr.map(|a| a.to_string()).unwrap_or_else(|| "-".into());
                    println!("🌟 [BUFFS]     Client: {:<21} | Obj {}: {} active buff effects",
                        client_str, object_id, buffs.len());
                }
                CompanionEvent::InventoryLoaded { client_addr, object_id, items } => {
                    let client_str = client_addr.map(|a| a.to_string()).unwrap_or_else(|| "-".into());
                    let equipped = items.iter().filter(|i| i.equipped).count();
                    println!("🎒 [INVENTORY] Client: {:<21} | Obj {}: {} items ({} equipped)",
                        client_str, object_id, items.len(), equipped);
                }
                CompanionEvent::WarehouseLoaded { client_addr, object_id, wh_type, player_adena, items } => {
                    let client_str = client_addr.map(|a| a.to_string()).unwrap_or_else(|| "-".into());
                    println!("🏛️ [WAREHOUSE] Client: {:<21} | Obj {}: {:?} warehouse ({} items, {} adena)",
                        client_str, object_id, wh_type, items.len(), format_number(player_adena));
                }
                CompanionEvent::PrivateStoreUpdated { client_addr, store } => {
                    let client_str = client_addr.map(|a| a.to_string()).unwrap_or_else(|| "-".into());
                    let seller = store.seller_name.unwrap_or_else(|| format!("Obj {}", store.seller_object_id));
                    println!("🏪 [STORE]     Client: {:<21} | {:?} Store by \"{}\" ('{}') - {} items listed",
                        client_str, store.store_type, seller, store.store_title, store.items.len());
                }
                CompanionEvent::CommissionMarketUpdated { items } => {
                    println!("🏛️ [AUCTION]   Auction House Commission updated: {} items listed", items.len());
                }
                CompanionEvent::WorldExchangeUpdated { items } => {
                    println!("🌐 [EXCHANGE]  World Exchange Market updated: {} items listed", items.len());
                }
                CompanionEvent::EinhasadStoreUpdated { products } => {
                    println!("🪙 [EINHASAD]  Einhasad Gold Coin Store updated: {} offerings available", products.len());
                }
                _ => {}
            }
        }
    });

    let _ = worker_handle.await;
    let (total_pkts, client_stats) = ingestion_handle.await.unwrap_or((0, HashMap::new()));
    display_handle.abort();

    let tracked = tracker.get_characters().await;
    let accounts = tracker.get_accounts().await;
    let market = tracker.get_market_state().await;

    println!("\n================== Capture Summary ==================");
    println!("Total Packets Decoded: {}", total_pkts);
    println!("Unique Client Streams: {}", client_stats.len());
    if !accounts.is_empty() {
        println!("Accounts Detected:     {}", accounts.len());
        for acc in &accounts {
            println!(" - Account \"{}\" (Client: {}) -> {} characters in roster",
                acc.account_name, acc.client_addr, acc.character_roster.len());
        }
    }
    if !tracked.is_empty() {
        println!("Tracked Characters:    {}", tracked.len());
        for c in &tracked {
            println!(" - Character \"{}\" (Lvl {}, Class {}) | Skills: {} | Buffs: {} | Inventory: {} items | Warehouse: {} items",
                c.name, c.level, c.class_id, c.skills.len(), c.buffs.len(), c.inventory.len(), c.warehouse.len());
        }
    }
    if !market.private_stores.is_empty() || !market.commission_items.is_empty() || !market.world_exchange_items.is_empty() {
        println!("\nMarket Activity Summary:");
        if !market.private_stores.is_empty() {
            println!(" - Active Private Stores: {}", market.private_stores.len());
        }
        if !market.commission_items.is_empty() {
            println!(" - Commission Auctions:   {}", market.commission_items.len());
        }
        if !market.world_exchange_items.is_empty() {
            println!(" - World Exchange Items:  {}", market.world_exchange_items.len());
        }
    }
    println!("\nActive Client Sessions:");
    let mut sorted_clients: Vec<_> = client_stats.into_iter().collect();
    sorted_clients.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

    for (addr, count) in sorted_clients {
        println!(" - Client Endpoint: {:<22} | Packets: {:>6}", addr.to_string(), count);
    }
    println!("====================================================\n");

    Ok(())
}

fn format_unix_timestamp(ts: u32) -> String {
    if ts == 0 {
        return "-".into();
    }
    let min = (ts / 60) % 60;
    let hour = (ts / 3600) % 24;
    let mut days = ts / 86400;

    let mut year = 1970;
    loop {
        let leap = if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) { 1 } else { 0 };
        let days_in_year = 365 + leap;
        if days < days_in_year {
            let month_days = [31, 28 + leap, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
            let mut month = 1;
            for md in month_days {
                if days < md {
                    let day = days + 1;
                    return format!("{:04}/{:02}/{:02} {:02}:{:02}", year, month, day, hour, min);
                }
                days -= md;
                month += 1;
            }
            break;
        }
        days -= days_in_year;
        year += 1;
    }
    format!("{ts}")
}

fn format_number(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    let chars: Vec<_> = s.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*c);
    }
    out
}
