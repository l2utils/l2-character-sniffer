//! # l2-sniffer CLI
//!
//! Command line tool to inspect network interfaces, capture packets, and monitor character statistics live.

use anyhow::Result;
use clap::{Parser, Subcommand};
use l2_sniffer_capture::{list_devices, SnifferBuilder};
use l2_sniffer_core::{CharacterTracker, SnifferEvent};
use tokio::sync::mpsc;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "l2-sniffer", version, about = "Lineage 2 Character Data Sniffer CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// List all network interfaces available for packet capture
    Devices,
    /// Start sniffing network packets and display live character updates
    Sniff {
        /// Network interface name to capture on (default: auto-detect)
        #[arg(short, long)]
        device: Option<String>,

        /// Read from offline pcap file instead of live interface
        #[arg(short, long)]
        pcap: Option<String>,

        /// Custom BPF filter (default: "tcp port 7777 or tcp port 2106")
        #[arg(short, long)]
        filter: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
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
        } => {
            info!("Starting Lineage 2 Sniffer...");
            let mut builder = SnifferBuilder::new();

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
            let tracker = CharacterTracker::new();
            let mut event_rx = tracker.subscribe();

            let (tx, mut rx) = mpsc::channel(1024);
            session.spawn_worker(tx);

            // Packet ingestion task
            let tracker_clone = tracker.clone();
            tokio::spawn(async move {
                while let Some(packet) = rx.recv().await {
                    tracker_clone.handle_packet(packet).await;
                }
            });

            println!("Sniffer is running. Waiting for game packets... (Press Ctrl+C to stop)");

            // Live event display loop
            while let Ok(event) = event_rx.recv().await {
                match event {
                    SnifferEvent::CharacterLoaded(c) => {
                        println!("\n[CHARACTER LOADED] Name: {} | Level: {} | Class: {} | HP: {}/{} | Location: ({}, {}, {})",
                            c.name, c.level, c.class_id, c.vitals.cur_hp, c.vitals.max_hp, c.location.x, c.location.y, c.location.z);
                    }
                    SnifferEvent::VitalsChanged { object_id, vitals } => {
                        println!("[VITALS UPDATE] Object {}: HP: {}/{} | MP: {}/{} | CP: {}/{}",
                            object_id, vitals.cur_hp, vitals.max_hp, vitals.cur_mp, vitals.max_mp, vitals.cur_cp, vitals.max_cp);
                    }
                    SnifferEvent::LocationChanged { object_id, location } => {
                        println!("[LOCATION UPDATE] Object {}: ({}, {}, {})",
                            object_id, location.x, location.y, location.z);
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
