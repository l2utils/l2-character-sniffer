//! Packet capture engine with multi-client TCP stream demuxing.

use std::collections::HashMap;
use std::fs::File;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use bytes::BytesMut;
use l2companion_protocol::{L2FrameCodec, L2Packet};
use pcap::Capture;
use pcap_file::pcapng::PcapNgReader;
use pcap_file::pcap::PcapReader;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

#[derive(Debug, Error)]
pub enum CaptureError {
    #[error("Pcap error: {0}")]
    Pcap(#[from] pcap::Error),
    #[error("Pcap-file error: {0}")]
    PcapFile(String),
    #[error("Interface not found: {0}")]
    InterfaceNotFound(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Identifies packet direction relative to the client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketDirection {
    ServerToClient,
    ClientToServer,
}

/// A captured packet associated with a specific client session.
#[derive(Debug, Clone)]
pub struct SessionPacket {
    pub client_addr: SocketAddr,
    pub server_addr: SocketAddr,
    pub direction: PacketDirection,
    pub packet: L2Packet,
}

/// Messages emitted by the capture worker.
#[derive(Debug, Clone)]
pub enum SessionMessage {
    ClientConnected {
        client_addr: SocketAddr,
        server_addr: SocketAddr,
    },
    ClientDisconnected {
        client_addr: SocketAddr,
        reason: String,
    },
    Packet(SessionPacket),
}

/// Per-client TCP stream state for reassembling and decoding packets.
pub struct ClientStream {
    pub client_addr: SocketAddr,
    pub server_addr: SocketAddr,
    pub rx_buffer: BytesMut,
    pub codec: L2FrameCodec,
    pub packet_count: u64,
}

impl ClientStream {
    pub fn new(client_addr: SocketAddr, server_addr: SocketAddr) -> Self {
        Self {
            client_addr,
            server_addr,
            rx_buffer: BytesMut::with_capacity(65535),
            codec: L2FrameCodec::default(),
            packet_count: 0,
        }
    }

    pub fn ingest_server_payload(
        &mut self,
        payload: &[u8],
        tx: &mpsc::Sender<SessionMessage>,
    ) {
        self.rx_buffer.extend_from_slice(payload);

        while let Ok(Some(frame)) = self.codec.decode(&mut self.rx_buffer) {
            if !frame.is_empty() {
                self.packet_count += 1;
                let opcode = frame[0];
                let parsed = L2Packet::parse(opcode, &frame[1..]);

                let session_packet = SessionPacket {
                    client_addr: self.client_addr,
                    server_addr: self.server_addr,
                    direction: PacketDirection::ServerToClient,
                    packet: parsed,
                };
                let _ = tx.blocking_send(SessionMessage::Packet(session_packet));
            }
        }
    }
}

pub struct CaptureBuilder {
    device_name: Option<String>,
    pcap_file: Option<String>,
    bpf_filter: String,
    snaplen: i32,
    promiscuous: bool,
    timeout_ms: i32,
    game_ports: Vec<u16>,
}

impl Default for CaptureBuilder {
    fn default() -> Self {
        Self {
            device_name: None,
            pcap_file: None,
            bpf_filter: "tcp port 7777 or tcp port 2106".to_string(),
            snaplen: 65535,
            promiscuous: true,
            timeout_ms: 1000,
            game_ports: vec![7777, 2106],
        }
    }
}

impl CaptureBuilder {
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

    pub fn game_ports(mut self, ports: Vec<u16>) -> Self {
        self.game_ports = ports;
        self
    }

    pub fn build(self) -> Result<CaptureSession, CaptureError> {
        if let Some(file_path) = self.pcap_file {
            return Ok(CaptureSession::OfflineFile {
                path: file_path,
                game_ports: self.game_ports,
            });
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

        Ok(CaptureSession::Live {
            cap,
            filter: self.bpf_filter,
            game_ports: self.game_ports,
        })
    }
}

pub enum CaptureSession {
    Live {
        cap: Capture<pcap::Active>,
        filter: String,
        game_ports: Vec<u16>,
    },
    OfflineFile {
        path: String,
        game_ports: Vec<u16>,
    },
}

impl CaptureSession {
    /// Starts background capture thread streaming parsed packets with client session context.
    pub fn spawn_worker(mut self, tx: mpsc::Sender<SessionMessage>) -> tokio::task::JoinHandle<()> {
        tokio::task::spawn_blocking(move || {
            let mut streams: HashMap<SocketAddr, ClientStream> = HashMap::new();

            match &mut self {
                CaptureSession::Live { cap, filter, game_ports } => {
                    if let Err(e) = cap.filter(filter, true) {
                        warn!("Failed to apply BPF filter '{filter}': {e}");
                    }
                    info!("Telemetry live capture started on network interface");

                    loop {
                        match cap.next_packet() {
                            Ok(packet) => {
                                Self::process_raw_frame(packet.data, game_ports, &mut streams, &tx);
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
                CaptureSession::OfflineFile { path, game_ports } => {
                    info!("Telemetry processing offline capture: {path}");
                    if let Err(e) = Self::read_offline_file(path, game_ports, &mut streams, &tx) {
                        error!("Error processing offline capture file: {e}");
                    }
                }
            }
            info!("Telemetry capture worker finished (tracked {} active client streams)", streams.len());
        })
    }

    fn read_offline_file(
        path: &str,
        game_ports: &[u16],
        streams: &mut HashMap<SocketAddr, ClientStream>,
        tx: &mpsc::Sender<SessionMessage>,
    ) -> Result<(), CaptureError> {
        let file = File::open(path)?;

        // Try reading as PCAPNG first
        if let Ok(mut reader) = PcapNgReader::new(file) {
            info!("Parsed capture as PCAPNG format");
            while let Some(block) = reader.next_block() {
                if let Ok(pcap_file::pcapng::Block::EnhancedPacket(epb)) = block {
                    Self::process_raw_frame(&epb.data, game_ports, streams, tx);
                } else if let Ok(pcap_file::pcapng::Block::SimplePacket(spb)) = block {
                    Self::process_raw_frame(&spb.data, game_ports, streams, tx);
                }
            }
            return Ok(());
        }

        // Fallback to legacy PCAP format
        let file2 = File::open(path)?;
        if let Ok(mut reader) = PcapReader::new(file2) {
            info!("Parsed capture as legacy PCAP format");
            while let Some(pkt) = reader.next_packet() {
                if let Ok(p) = pkt {
                    Self::process_raw_frame(&p.data, game_ports, streams, tx);
                }
            }
            return Ok(());
        }

        Err(CaptureError::PcapFile("Unsupported pcap/pcapng format".into()))
    }

    /// Extracts IPv4/TCP headers, detects SYN/FIN/RST connection lifecycle, and routes payloads.
    fn process_raw_frame(
        raw: &[u8],
        game_ports: &[u16],
        streams: &mut HashMap<SocketAddr, ClientStream>,
        tx: &mpsc::Sender<SessionMessage>,
    ) {
        // Minimum Ethernet (14) + IPv4 (20) + TCP (20) = 54 bytes
        if raw.len() < 54 {
            return;
        }

        // Check for Ethernet II IPv4 frame (EtherType 0x0800)
        let ethertype = u16::from_be_bytes([raw[12], raw[13]]);
        if ethertype != 0x0800 {
            return;
        }

        // IPv4 Header
        let ip_header = &raw[14..];
        let ip_version = (ip_header[0] >> 4) & 0x0F;
        let ip_ihl = ((ip_header[0] & 0x0F) * 4) as usize;
        let ip_proto = ip_header[9];

        if ip_version != 4 || ip_proto != 6 || raw.len() < 14 + ip_ihl + 20 {
            return; // Not TCP IPv4
        }

        let src_ip = IpAddr::V4(Ipv4Addr::new(ip_header[12], ip_header[13], ip_header[14], ip_header[15]));
        let dst_ip = IpAddr::V4(Ipv4Addr::new(ip_header[16], ip_header[17], ip_header[18], ip_header[19]));

        // TCP Header
        let tcp_header = &raw[14 + ip_ihl..];
        let src_port = u16::from_be_bytes([tcp_header[0], tcp_header[1]]);
        let dst_port = u16::from_be_bytes([tcp_header[2], tcp_header[3]]);
        let tcp_offset = (((tcp_header[12] >> 4) & 0x0F) * 4) as usize;
        let tcp_flags = tcp_header[13];

        let is_fin = (tcp_flags & 0x01) != 0;
        let is_rst = (tcp_flags & 0x04) != 0;

        let total_header_len = 14 + ip_ihl + tcp_offset;
        let src_socket = SocketAddr::new(src_ip, src_port);
        let dst_socket = SocketAddr::new(dst_ip, dst_port);

        // Determine client and server addresses
        let (client_addr, server_addr, is_server_to_client) = if game_ports.contains(&src_port) {
            (dst_socket, src_socket, true)
        } else if game_ports.contains(&dst_port) {
            (src_socket, dst_socket, false)
        } else {
            return; // Irrelevant port
        };

        // Handle disconnect (FIN or RST)
        if (is_fin || is_rst) && streams.contains_key(&client_addr) {
            streams.remove(&client_addr);
            let reason = if is_rst {
                "Connection reset (RST)".to_string()
            } else {
                "Connection closed (FIN)".to_string()
            };
            let _ = tx.blocking_send(SessionMessage::ClientDisconnected {
                client_addr,
                reason,
            });
            return;
        }

        // If this is a new client connection, register it and notify
        if !streams.contains_key(&client_addr) {
            streams.insert(client_addr, ClientStream::new(client_addr, server_addr));
            let _ = tx.blocking_send(SessionMessage::ClientConnected {
                client_addr,
                server_addr,
            });
        }

        // Process server -> client payload if present
        if is_server_to_client && raw.len() > total_header_len {
            let payload = &raw[total_header_len..];
            if let Some(stream) = streams.get_mut(&client_addr) {
                stream.ingest_server_payload(payload, tx);
            }
        }
    }
}
