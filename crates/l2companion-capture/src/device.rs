//! Network device discovery and enumeration.

use pcap::Device;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub description: Option<String>,
    pub addresses: Vec<String>,
}

/// Enumerates all available network capture interfaces on the host.
pub fn list_devices() -> Result<Vec<NetworkInterface>, pcap::Error> {
    let devices = Device::list()?;
    let result = devices
        .into_iter()
        .map(|d| {
            let addresses = d
                .addresses
                .into_iter()
                .map(|a| a.addr.to_string())
                .collect();
            NetworkInterface {
                name: d.name,
                description: d.desc,
                addresses,
            }
        })
        .collect();
    Ok(result)
}

/// Returns the default network interface if available.
pub fn default_device() -> Result<Option<NetworkInterface>, pcap::Error> {
    let devices = list_devices()?;
    Ok(devices.into_iter().next())
}
