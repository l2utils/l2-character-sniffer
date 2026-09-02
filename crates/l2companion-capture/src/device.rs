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

/// Finds a network interface by index (1-based), name, description substring, or IP address.
pub fn find_device(query: &str) -> Result<Option<NetworkInterface>, pcap::Error> {
    let devices = list_devices()?;

    // 1. Try numeric index (e.g. "5" or "1")
    if let Ok(idx) = query.parse::<usize>() {
        if idx >= 1 && idx <= devices.len() {
            return Ok(Some(devices[idx - 1].clone()));
        }
    }

    // 2. Try exact name match
    if let Some(dev) = devices.iter().find(|d| d.name.eq_ignore_ascii_case(query)) {
        return Ok(Some(dev.clone()));
    }

    // 3. Try IP match
    if let Some(dev) = devices.iter().find(|d| d.addresses.iter().any(|a| a.contains(query))) {
        return Ok(Some(dev.clone()));
    }

    // 4. Try description substring match
    if let Some(dev) = devices.iter().find(|d| {
        d.description
            .as_ref()
            .map(|desc| desc.to_lowercase().contains(&query.to_lowercase()))
            .unwrap_or(false)
    }) {
        return Ok(Some(dev.clone()));
    }

    Ok(None)
}

/// Returns the best active default network interface (ranked by active private IPv4 address).
pub fn default_device() -> Result<Option<NetworkInterface>, pcap::Error> {
    let devices = list_devices()?;

    // 1. Prefer devices with a standard private LAN/Wi-Fi IPv4 (192.168.x.x, 10.x.x.x, 172.16-31.x.x)
    let best_private = devices.iter().find(|d| {
        d.addresses.iter().any(|addr| {
            if let Ok(ip) = addr.parse::<std::net::IpAddr>() {
                match ip {
                    std::net::IpAddr::V4(ipv4) => {
                        let oct = ipv4.octets();
                        (oct[0] == 192 && oct[1] == 168)
                            || oct[0] == 10
                            || (oct[0] == 172 && (16..=31).contains(&oct[1]))
                    }
                    _ => false,
                }
            } else {
                false
            }
        })
    });

    if let Some(dev) = best_private {
        return Ok(Some(dev.clone()));
    }

    // 2. Fallback to any interface with a non-loopback, non-APIPA (169.254) IPv4 address
    let non_loopback = devices.iter().find(|d| {
        d.addresses.iter().any(|addr| {
            if let Ok(ip) = addr.parse::<std::net::IpAddr>() {
                match ip {
                    std::net::IpAddr::V4(ipv4) => !ipv4.is_loopback() && !ipv4.is_link_local(),
                    _ => false,
                }
            } else {
                false
            }
        })
    });

    if let Some(dev) = non_loopback {
        return Ok(Some(dev.clone()));
    }

    // 3. Fallback to first device
    Ok(devices.into_iter().next())
}
