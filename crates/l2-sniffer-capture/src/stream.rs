//! Packet capture engine and sniffer runner.

use std::path::Path;
use bytes::BytesMut;
use l2_sniffer_protocol::{L2FrameCodec, L2Packet};
use pcap::Capture;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("Pcap error: {0}")]
    Pcap(#[from] pcap::Error),
    #[error("Interface not found: {0}")]
    InterfaceNotFound(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct SnifferBuilder {
    device_name: Option<String>,
    pcap_file: Option<String>,
    bpf_filter: String,
    snaplen: i32,
    promiscuous: bool,
    timeout_ms: i32,
}

impl Default for SnifferBuilder {
    fn default() -> Self {
        Self {
            device_name: None,
            pcap_file: None,
            // Default L2 Game Server port is 7777, Login Server is 2106
            bpf_filter: "tcp port 7777 or tcp port 2106".to_string(),
            snaplen: 65535,
            promiscuous: true,
            timeout_ms: 1000,
        }
    }
}

impl SnifferBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn device(mut self, device: impl Into<String>) -> Self {
        self.device_name = Some(device.into());
        self
    }

    pub fn pcap_file(mut self, path: impl Into<String>) -> Self {
        self.pcap_file = Some(path.into());
        self
    }

    pub fn filter(mut self, filter: impl Into<String>) -> Self {
        self.bpf_filter = filter.into();
        self
    }

    pub fn build(self) -> Result<SnifferSession, CaptureError> {
        if let Some(file_path) = self.pcap_file {
            let cap = Capture::from_file(Path::new(&file_path))?;
            return Ok(SnifferSession::File(cap));
        }

        let device_name = match self.device_name {
            Some(dev) => dev,
            None => {
                let dev = pcap::Device::lookup()?
                    .ok_or_else(|| CaptureError::InterfaceNotFound("No default device found".into()))?;
                dev.name
            }
        };

        let cap: Capture<pcap::Active> = Capture::from_device(device_name.as_str())?
            .promisc(self.promiscuous)
            .snaplen(self.snaplen)
            .timeout(self.timeout_ms)
            .open()?
            .setnonblock()?;

        Ok(SnifferSession::Live(cap, self.bpf_filter))
    }
}

pub enum SnifferSession {
    Live(Capture<pcap::Active>, String),
    File(Capture<pcap::Offline>),
}

impl SnifferSession {
    /// Starts background capture thread sending framed raw payloads or parsed packets to an async channel.
    pub fn spawn_worker(mut self, tx: mpsc::Sender<L2Packet>) -> tokio::task::JoinHandle<()> {
        tokio::task::spawn_blocking(move || {
            let mut codec = L2FrameCodec::default();
            let mut stream_buffer = BytesMut::with_capacity(65535);

            match &mut self {
                SnifferSession::Live(cap, filter) => {
                    if let Err(e) = cap.filter(filter, true) {
                        warn!("Failed to apply BPF filter '{filter}': {e}");
                    }
                    info!("Sniffer live capture started");

                    loop {
                        match cap.next_packet() {
                            Ok(packet) => {
                                Self::process_packet_data(packet.data, &mut stream_buffer, &mut codec, &tx);
                            }
                            Err(pcap::Error::TimeoutExpired) => continue,
                            Err(pcap::Error::NoMorePackets) => break,
                            Err(e) => {
                                error!("Packet capture read error: {e}");
                                break;
                            }
                        }
                    }
                }
                SnifferSession::File(cap) => {
                    info!("Sniffer reading offline pcap file");
                    while let Ok(packet) = cap.next_packet() {
                        Self::process_packet_data(packet.data, &mut stream_buffer, &mut codec, &tx);
                    }
                }
            }
            info!("Sniffer capture worker finished");
        })
    }

    fn process_packet_data(
        raw: &[u8],
        buf: &mut BytesMut,
        codec: &mut L2FrameCodec,
        tx: &mpsc::Sender<L2Packet>,
    ) {
        // Strip Ethernet/IP/TCP headers or push payload
        // Standard TCP payload offset heuristic (e.g. Ethernet (14) + IP (20) + TCP (20) = 54 bytes)
        let payload_offset = if raw.len() > 54 && raw[12] == 0x08 && raw[13] == 0x00 {
            let ip_header_len = ((raw[14] & 0x0F) * 4) as usize;
            let tcp_offset_byte = 14 + ip_header_len + 12;
            if raw.len() > tcp_offset_byte {
                let tcp_header_len = (((raw[tcp_offset_byte] >> 4) & 0x0F) * 4) as usize;
                14 + ip_header_len + tcp_header_len
            } else {
                54
            }
        } else {
            0
        };

        if raw.len() > payload_offset {
            let payload = &raw[payload_offset..];
            buf.extend_from_slice(payload);

            while let Ok(Some(frame)) = codec.decode(buf) {
                if !frame.is_empty() {
                    let opcode = frame[0];
                    let parsed = L2Packet::parse(opcode, &frame[1..]);
                    let _ = tx.blocking_send(parsed);
                }
            }
        }
    }
}
