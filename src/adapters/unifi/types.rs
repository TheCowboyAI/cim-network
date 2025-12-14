//! UniFi API types

use crate::domain::value_objects::MacAddress;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::Ipv4Addr;

/// UniFi device representation from the API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniFiDevice {
    /// UniFi device ID (usually MAC-based)
    #[serde(rename = "_id")]
    pub id: String,
    /// MAC address
    #[serde(deserialize_with = "deserialize_mac")]
    pub mac: MacAddress,
    /// Device model (e.g., "U6-Pro", "USW-24-POE")
    pub model: String,
    /// Device name
    pub name: String,
    /// IP address
    pub ip: Option<Ipv4Addr>,
    /// Whether device is adopted
    pub adopted: bool,
    /// Device type (uap, usw, ugw, etc.)
    #[serde(rename = "type")]
    pub device_type: String,
    /// Firmware version
    pub version: Option<String>,
    /// Device state (1 = connected, etc.)
    pub state: Option<i32>,
    /// Additional properties
    #[serde(flatten)]
    pub properties: HashMap<String, serde_json::Value>,
}

/// UniFi device statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniFiDeviceStats {
    /// Uptime in seconds
    pub uptime: Option<u64>,
    /// CPU usage percentage
    #[serde(rename = "cpu")]
    pub cpu_usage: Option<f64>,
    /// Memory usage percentage
    #[serde(rename = "mem")]
    pub mem_usage: Option<f64>,
    /// Temperature in Celsius
    #[serde(rename = "general_temperature")]
    pub temperature: Option<f64>,
    /// Port statistics
    #[serde(default)]
    pub port_stats: Vec<UniFiPortStats>,
}

/// UniFi port statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniFiPortStats {
    /// Port index
    pub port_idx: u32,
    /// Whether port is up
    pub up: bool,
    /// Speed in Mbps
    pub speed: Option<u32>,
    /// Full duplex
    pub full_duplex: Option<bool>,
    /// Received bytes
    #[serde(default)]
    pub rx_bytes: u64,
    /// Transmitted bytes
    #[serde(default)]
    pub tx_bytes: u64,
    /// Receive errors
    pub rx_errors: Option<u64>,
    /// Transmit errors
    pub tx_errors: Option<u64>,
}

/// UniFi API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniFiResponse<T> {
    pub meta: UniFiMeta,
    pub data: Vec<T>,
}

/// UniFi API meta information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniFiMeta {
    pub rc: String,
    pub msg: Option<String>,
}

impl UniFiMeta {
    pub fn is_ok(&self) -> bool {
        self.rc == "ok"
    }
}

/// UniFi API error
#[derive(Debug, Clone, thiserror::Error)]
pub enum UniFiError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Authentication failed: {0}")]
    Auth(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Device not found: {0}")]
    NotFound(String),
}

/// Deserialize MAC address from UniFi format (no colons)
fn deserialize_mac<'de, D>(deserializer: D) -> Result<MacAddress, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    MacAddress::parse(&s).map_err(serde::de::Error::custom)
}
