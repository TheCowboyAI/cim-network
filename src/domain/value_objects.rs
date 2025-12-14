//! Value objects for the network domain
//!
//! Immutable domain primitives following cim-domain patterns.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::net::IpAddr;
use uuid::Uuid;

/// Network device identifier (UUID v7 for time-ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DeviceId(Uuid);

impl DeviceId {
    /// Create a new device ID using UUID v7
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    /// Create from existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the inner UUID
    pub fn inner(&self) -> Uuid {
        self.0
    }
}

impl Default for DeviceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Network topology identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TopologyId(Uuid);

impl TopologyId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn inner(&self) -> Uuid {
        self.0
    }
}

impl Default for TopologyId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TopologyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Connection identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(Uuid);

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }

    pub fn inner(&self) -> Uuid {
        self.0
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// MAC address value object
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    /// Create from bytes
    pub fn from_bytes(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }

    /// Parse from string (supports : and - separators)
    pub fn parse(s: &str) -> Result<Self, MacAddressError> {
        let cleaned = s.replace([':', '-'], "");
        if cleaned.len() != 12 {
            return Err(MacAddressError::InvalidLength);
        }

        let mut bytes = [0u8; 6];
        for (i, chunk) in cleaned.as_bytes().chunks(2).enumerate() {
            let hex_str =
                std::str::from_utf8(chunk).map_err(|_| MacAddressError::InvalidFormat)?;
            bytes[i] =
                u8::from_str_radix(hex_str, 16).map_err(|_| MacAddressError::InvalidFormat)?;
        }

        Ok(Self(bytes))
    }

    /// Get bytes
    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }
}

impl fmt::Display for MacAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

/// MAC address parsing error
#[derive(Debug, Clone, thiserror::Error)]
pub enum MacAddressError {
    #[error("Invalid MAC address length")]
    InvalidLength,
    #[error("Invalid MAC address format")]
    InvalidFormat,
}

/// Network device type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    /// Gateway/Router device
    Gateway,
    /// Network switch
    Switch,
    /// Wireless access point
    AccessPoint,
    /// Generic network device
    Generic { model: String },
}

impl fmt::Display for DeviceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceType::Gateway => write!(f, "Gateway"),
            DeviceType::Switch => write!(f, "Switch"),
            DeviceType::AccessPoint => write!(f, "AccessPoint"),
            DeviceType::Generic { model } => write!(f, "Generic({})", model),
        }
    }
}

/// Port identifier on a device
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortId {
    /// Port number or name
    pub name: String,
    /// Port index (if applicable)
    pub index: Option<u32>,
}

impl PortId {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            index: None,
        }
    }

    pub fn with_index(name: impl Into<String>, index: u32) -> Self {
        Self {
            name: name.into(),
            index: Some(index),
        }
    }
}

impl fmt::Display for PortId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.index {
            Some(idx) => write!(f, "{}[{}]", self.name, idx),
            None => write!(f, "{}", self.name),
        }
    }
}

/// Network interface configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InterfaceConfig {
    /// Interface name
    pub name: String,
    /// IP address (if configured)
    pub ip_address: Option<IpAddr>,
    /// Subnet prefix length
    pub prefix_len: Option<u8>,
    /// VLAN ID (if tagged)
    pub vlan_id: Option<u16>,
    /// Whether interface is enabled
    pub enabled: bool,
}

/// VLAN configuration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VlanConfig {
    /// VLAN ID (1-4094)
    pub id: u16,
    /// VLAN name
    pub name: String,
    /// Whether this is the native/untagged VLAN
    pub native: bool,
}

impl VlanConfig {
    pub fn new(id: u16, name: impl Into<String>) -> Result<Self, VlanError> {
        if id == 0 || id > 4094 {
            return Err(VlanError::InvalidId(id));
        }
        Ok(Self {
            id,
            name: name.into(),
            native: false,
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum VlanError {
    #[error("Invalid VLAN ID {0}: must be 1-4094")]
    InvalidId(u16),
}

/// Connection type between devices
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Physical ethernet connection
    Ethernet,
    /// Fiber optic connection
    Fiber,
    /// Wireless connection
    Wireless,
    /// Logical/virtual connection
    Virtual,
    /// Uplink to parent device
    Uplink,
}

/// Link speed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkSpeed {
    /// 10 Mbps
    Mbps10,
    /// 100 Mbps
    Mbps100,
    /// 1 Gbps
    Gbps1,
    /// 2.5 Gbps
    Gbps2_5,
    /// 5 Gbps
    Gbps5,
    /// 10 Gbps
    Gbps10,
    /// 25 Gbps
    Gbps25,
    /// 40 Gbps
    Gbps40,
    /// 100 Gbps
    Gbps100,
}

impl fmt::Display for LinkSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkSpeed::Mbps10 => write!(f, "10 Mbps"),
            LinkSpeed::Mbps100 => write!(f, "100 Mbps"),
            LinkSpeed::Gbps1 => write!(f, "1 Gbps"),
            LinkSpeed::Gbps2_5 => write!(f, "2.5 Gbps"),
            LinkSpeed::Gbps5 => write!(f, "5 Gbps"),
            LinkSpeed::Gbps10 => write!(f, "10 Gbps"),
            LinkSpeed::Gbps25 => write!(f, "25 Gbps"),
            LinkSpeed::Gbps40 => write!(f, "40 Gbps"),
            LinkSpeed::Gbps100 => write!(f, "100 Gbps"),
        }
    }
}
