//! Value objects for the network domain

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use thiserror::Error;
use uuid::Uuid;

/// Network identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(Uuid);

impl NetworkId {
    /// Create a new network ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Router identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RouterId(Uuid);

impl RouterId {
    /// Create a new router ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for RouterId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Switch identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SwitchId(Uuid);

impl SwitchId {
    /// Create a new switch ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for SwitchId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Container network identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContainerNetworkId(Uuid);

impl ContainerNetworkId {
    /// Create a new container network ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl fmt::Display for ContainerNetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// VLAN ID (1-4094, excluding reserved)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VlanId(u16);

#[derive(Error, Debug)]
pub enum VlanIdError {
    #[error("VLAN ID {0} is reserved")]
    Reserved(u16),
    #[error("VLAN ID {0} is out of range (must be 1-4094)")]
    OutOfRange(u16),
}

impl VlanId {
    /// Create a new VLAN ID with validation
    pub fn try_new(id: u16) -> Result<Self, VlanIdError> {
        match id {
            0 => Err(VlanIdError::Reserved(id)),
            4095 => Err(VlanIdError::Reserved(id)),
            1..=4094 => Ok(Self(id)),
            _ => Err(VlanIdError::OutOfRange(id)),
        }
    }
    
    /// Get the inner value
    pub fn value(&self) -> u16 {
        self.0
    }
}

/// MAC address
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MacAddress([u8; 6]);

#[derive(Error, Debug)]
pub enum MacAddressError {
    #[error("Invalid MAC address format")]
    InvalidFormat,
    #[error("Invalid MAC address length")]
    InvalidLength,
}

impl MacAddress {
    /// Parse MAC address from string (supports multiple formats)
    pub fn from_str(s: &str) -> Result<Self, MacAddressError> {
        let cleaned = s.replace(&[':', '-'][..], "");
        
        if cleaned.len() != 12 {
            return Err(MacAddressError::InvalidLength);
        }
        
        let mut bytes = [0u8; 6];
        for (i, chunk) in cleaned.as_bytes().chunks(2).enumerate() {
            if i >= 6 {
                return Err(MacAddressError::InvalidLength);
            }
            let hex_str = std::str::from_utf8(chunk)
                .map_err(|_| MacAddressError::InvalidFormat)?;
            bytes[i] = u8::from_str_radix(hex_str, 16)
                .map_err(|_| MacAddressError::InvalidFormat)?;
        }
        
        Ok(Self(bytes))
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

/// Correlation ID for event correlation (MANDATORY)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(String);

impl CorrelationId {
    /// Create a new correlation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// Create from existing string
    pub fn from(s: String) -> Self {
        Self(s)
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Causation ID for event causation tracking (MANDATORY)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CausationId(String);

impl CausationId {
    /// Create a new causation ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
    
    /// Create from correlation ID (common pattern)
    pub fn from_correlation(correlation_id: &CorrelationId) -> Self {
        Self(correlation_id.to_string())
    }
}

impl fmt::Display for CausationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Event ID
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(String);

impl EventId {
    /// Create a new event ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl fmt::Display for EventId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// IP network wrapper
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IpNetwork(ipnetwork::IpNetwork);

impl FromStr for IpNetwork {
    type Err = ipnetwork::IpNetworkError;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse()?))
    }
}

impl IpNetwork {
    /// Get the inner network
    pub fn inner(&self) -> &ipnetwork::IpNetwork {
        &self.0
    }
}

/// Port number on a switch or router
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PortNumber(u32);

#[derive(Error, Debug)]
pub enum PortNumberError {
    #[error("Port number {0} is invalid (must be 1-65535)")]
    Invalid(u32),
}

impl PortNumber {
    /// Create a new port number with validation
    pub fn try_new(port: u32) -> Result<Self, PortNumberError> {
        if port == 0 || port > 65535 {
            Err(PortNumberError::Invalid(port))
        } else {
            Ok(Self(port))
        }
    }
    
    /// Get the inner value
    pub fn value(&self) -> u32 {
        self.0
    }
}

/// Aggregate identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AggregateId(String);

impl AggregateId {
    /// Create a new aggregate ID
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }
}

impl From<RouterId> for AggregateId {
    fn from(id: RouterId) -> Self {
        Self(id.to_string())
    }
}

impl From<SwitchId> for AggregateId {
    fn from(id: SwitchId) -> Self {
        Self(id.to_string())
    }
}

impl From<ContainerNetworkId> for AggregateId {
    fn from(id: ContainerNetworkId) -> Self {
        Self(id.to_string())
    }
}

/// Port speed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortSpeed {
    /// 10 Mbps
    TenMegabit,
    /// 100 Mbps
    HundredMegabit,
    /// 1 Gbps
    Gigabit,
    /// 10 Gbps
    TenGigabit,
    /// 25 Gbps
    TwentyFiveGigabit,
    /// 40 Gbps
    FortyGigabit,
    /// 100 Gbps
    HundredGigabit,
}

