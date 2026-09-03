//! # l2companion-capture
//!
//! Packet capture engine using pcap / pcapng with multi-client TCP stream demuxing for Lineage 2 network traffic.

pub mod device;
pub mod stream;

pub use device::{default_device, find_device, list_devices, NetworkInterface};
pub use stream::{
    CaptureBuilder, CaptureError, CaptureSession, ClientStream, PacketDirection, SessionMessage,
    SessionPacket,
};
