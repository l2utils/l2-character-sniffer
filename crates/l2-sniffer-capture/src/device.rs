use obfstr::obfstr;
use pcap::{Capture, Device};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub description: Option<String>,
    pub addresses: Vec<String>,
    pub is_physical: bool,
    pub recent_packet_count: u64,
}

/// Helper to test if a description or name represents a known virtual/WAN adapter.
fn is_virtual_adapter(desc: &str, name: &str) -> bool {
    let lower_desc = desc.to_lowercase();
    let lower_name = name.to_lowercase();
    let match_k = |k: &str| lower_desc.contains(k) || lower_name.contains(k);

    match_k(obfstr!("wan miniport"))
        || match_k(obfstr!("hyper-v"))
        || match_k(obfstr!("virtual"))
        || match_k(obfstr!("loopback"))
        || match_k(obfstr!("npcap loopback"))
        || match_k(obfstr!("tap-windows"))
        || match_k(obfstr!("vmware"))
        || match_k(obfstr!("virtualbox"))
        || match_k(obfstr!("vbox"))
        || match_k(obfstr!("wsl"))
        || match_k(obfstr!("bluetooth"))
        || match_k(obfstr!("tunnel"))
        || match_k(obfstr!("teredo"))
        || match_k(obfstr!("isatap"))
}

/// Helper to test if a description represents a physical NIC (Wi-Fi, Ethernet, Intel, Realtek, etc.).
fn is_physical_nic(desc: &str) -> bool {
    let lower = desc.to_lowercase();
    lower.contains(obfstr!("wi-fi"))
        || lower.contains(obfstr!("wifi"))
        || lower.contains(obfstr!("wireless"))
        || lower.contains(obfstr!("802.11"))
        || lower.contains(obfstr!("ethernet"))
        || lower.contains(obfstr!("gbe"))
        || lower.contains(obfstr!("gigabit"))
        || lower.contains(obfstr!("intel"))
        || lower.contains(obfstr!("realtek"))
        || lower.contains(obfstr!("killer"))
        || lower.contains(obfstr!("broadcom"))
        || lower.contains(obfstr!("qualcomm"))
        || lower.contains(obfstr!("marvell"))
        || lower.contains(obfstr!("lan"))
        || lower.contains(obfstr!("controller"))
        || lower.contains(obfstr!("adapter"))
}

/// Helper to check if a device has a valid non-loopback, non-APIPA private IPv4.
fn has_private_ipv4(addresses: &[String]) -> bool {
    addresses.iter().any(|addr| {
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
}

/// Quick non-blocking packet sampling over a brief duration to detect active traffic.
fn sample_device_traffic(device_name: &str, duration_ms: u64) -> u64 {
    let mut cap = match Capture::from_device(device_name) {
        Ok(c) => match c.snaplen(64).timeout(10).promisc(false).open() {
            Ok(opened) => match opened.setnonblock() {
                Ok(nonblock) => nonblock,
                Err(_) => return 0,
            },
            Err(_) => return 0,
        },
        Err(_) => return 0,
    };

    let start = Instant::now();
    let mut count = 0u64;
    let deadline = Duration::from_millis(duration_ms);

    while start.elapsed() < deadline {
        match cap.next_packet() {
            Ok(_) => count += 1,
            Err(pcap::Error::TimeoutExpired) => {
                std::thread::sleep(Duration::from_millis(5));
            }
            Err(_) => break,
        }
    }

    count
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
                .collect::<Vec<_>>();
            let desc = d.desc.unwrap_or_default();
            let is_virt = is_virtual_adapter(&desc, &d.name);
            let is_phys = !is_virt && (is_physical_nic(&desc) || has_private_ipv4(&addresses));

            NetworkInterface {
                name: d.name,
                description: if desc.is_empty() { None } else { Some(desc) },
                addresses,
                is_physical: is_phys,
                recent_packet_count: 0,
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
    if let Some(dev) = devices
        .iter()
        .find(|d| d.addresses.iter().any(|a| a.contains(query)))
    {
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

/// Returns the best active default network interface (prefers physical NICs with active traffic and private IPs).
pub fn default_device() -> Result<Option<NetworkInterface>, pcap::Error> {
    let mut devices = list_devices()?;
    if devices.is_empty() {
        return Ok(None);
    }

    // Sample traffic for candidate devices that are physical or have an IP address
    for dev in devices.iter_mut() {
        let desc = dev.description.as_deref().unwrap_or("");
        if !is_virtual_adapter(desc, &dev.name) && (!dev.addresses.is_empty() || dev.is_physical) {
            dev.recent_packet_count = sample_device_traffic(&dev.name, 100);
        }
    }

    // Score and rank devices
    devices.sort_by(|a, b| {
        let score = |dev: &NetworkInterface| -> i64 {
            let desc = dev.description.as_deref().unwrap_or("");
            let mut s = 0i64;

            if is_virtual_adapter(desc, &dev.name) {
                s -= 2000;
            } else {
                if dev.is_physical {
                    s += 1000;
                }
                if has_private_ipv4(&dev.addresses) {
                    s += 500;
                } else if !dev.addresses.is_empty() {
                    s += 100;
                }
                // Traffic bonus: each packet sampled adds 50 points
                s += (dev.recent_packet_count as i64) * 50;
            }
            s
        };

        score(b).cmp(&score(a))
    });

    Ok(devices.into_iter().next())
}
