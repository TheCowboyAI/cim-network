//! # Domain Ports (Hexagonal Architecture)
//!
//! Ports define the boundaries between the domain and external systems.
//! Each port is a trait that adapters must implement.
//!
//! ## Port Categories
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                     Domain Layer                                │
//! │  ┌─────────────────────────────────────────────────────────┐   │
//! │  │              Driving Ports (Inbound)                    │   │
//! │  │  • NetworkManagementPort - control plane operations     │   │
//! │  │  • DiscoveryPort - device discovery                     │   │
//! │  └─────────────────────────────────────────────────────────┘   │
//! │  ┌─────────────────────────────────────────────────────────┐   │
//! │  │              Driven Ports (Outbound)                    │   │
//! │  │  • DeviceControlPort - vendor device control            │   │
//! │  │  • InventoryPort - NetBox/DCIM projection               │   │
//! │  │  • EventStorePort - event persistence                   │   │
//! │  └─────────────────────────────────────────────────────────┘   │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Kan Extension Mapping
//!
//! The Kan extension maps domain operations to port implementations:
//! - `Lan_F(Domain)(UniFi)` → UniFi adapter implements DeviceControlPort
//! - `Lan_F(Domain)(NetBox)` → NetBox adapter implements InventoryPort
//! - Universal property ensures all adapters compose correctly

use async_trait::async_trait;
use std::collections::HashMap;

use super::value_objects::*;
use super::aggregates::*;
use super::events::*;

/// Error type for port operations
#[derive(Debug, thiserror::Error)]
pub enum PortError {
    #[error("Device not found: {0}")]
    DeviceNotFound(DeviceId),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Operation not supported: {0}")]
    NotSupported(String),

    #[error("Timeout: {0}")]
    Timeout(String),

    #[error("Vendor error: {0}")]
    VendorError(String),

    #[error("Inventory error: {0}")]
    InventoryError(String),
}

// ============================================================================
// Driving Ports (Inbound) - How the application is used
// ============================================================================

/// Network management operations (driving port)
///
/// This port defines how external systems interact with network management.
/// Commands come in through this port and result in domain events.
#[async_trait]
pub trait NetworkManagementPort: Send + Sync {
    /// Provision a new device into the network
    async fn provision_device(
        &self,
        device_type: DeviceType,
        name: String,
        mac: MacAddress,
    ) -> Result<DeviceId, PortError>;

    /// Adopt a discovered device
    async fn adopt_device(&self, device_id: DeviceId) -> Result<(), PortError>;

    /// Configure device settings
    async fn configure_device(
        &self,
        device_id: DeviceId,
        config: DeviceConfiguration,
    ) -> Result<(), PortError>;

    /// Decommission a device
    async fn decommission_device(&self, device_id: DeviceId) -> Result<(), PortError>;

    /// Create a connection between devices
    async fn connect_devices(
        &self,
        source: DeviceId,
        source_port: PortId,
        target: DeviceId,
        target_port: PortId,
        connection_type: ConnectionType,
    ) -> Result<ConnectionId, PortError>;
}

/// Device discovery operations (driving port)
#[async_trait]
pub trait DiscoveryPort: Send + Sync {
    /// Discover devices on the network
    async fn discover_devices(&self) -> Result<Vec<DiscoveredDevice>, PortError>;

    /// Get details for a specific device
    async fn get_device_details(&self, device_id: DeviceId) -> Result<DeviceDetails, PortError>;

    /// Subscribe to device events
    async fn subscribe_events(&self) -> Result<EventSubscription, PortError>;
}

// ============================================================================
// Driven Ports (Outbound) - What the application needs
// ============================================================================

/// Device control operations (driven port)
///
/// Implemented by vendor-specific adapters (UniFi, Cisco, etc.)
/// The Kan extension maps domain operations to this port.
#[async_trait]
pub trait DeviceControlPort: Send + Sync {
    /// Get the vendor name this adapter supports
    fn vendor_name(&self) -> &str;

    /// Connect to the vendor controller
    async fn connect(&self) -> Result<(), PortError>;

    /// Disconnect from the vendor controller
    async fn disconnect(&self) -> Result<(), PortError>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// List all devices from the vendor controller
    async fn list_devices(&self) -> Result<Vec<VendorDevice>, PortError>;

    /// Get device by ID
    async fn get_device(&self, vendor_id: &str) -> Result<VendorDevice, PortError>;

    /// Adopt a device
    async fn adopt_device(&self, vendor_id: &str) -> Result<(), PortError>;

    /// Apply configuration to a device
    async fn apply_config(&self, vendor_id: &str, config: VendorConfig) -> Result<(), PortError>;

    /// Restart a device
    async fn restart_device(&self, vendor_id: &str) -> Result<(), PortError>;

    /// Get device statistics
    async fn get_device_stats(&self, vendor_id: &str) -> Result<DeviceStats, PortError>;
}

/// Inventory/DCIM operations (driven port)
///
/// Implemented by NetBox adapter for infrastructure documentation.
/// Projects domain state to NetBox for IPAM/DCIM.
#[async_trait]
pub trait InventoryPort: Send + Sync {
    /// Get the inventory system name
    fn system_name(&self) -> &str;

    /// Sync a device to inventory
    async fn sync_device(&self, device: &NetworkDeviceAggregate) -> Result<(), PortError>;

    /// Remove device from inventory
    async fn remove_device(&self, device_id: DeviceId) -> Result<(), PortError>;

    /// Sync a connection to inventory
    async fn sync_connection(
        &self,
        connection: &ConnectionInfo,
    ) -> Result<(), PortError>;

    /// Get IP address assignments
    async fn get_ip_assignments(&self, prefix: &str) -> Result<Vec<IpAssignment>, PortError>;

    /// Allocate IP address
    async fn allocate_ip(&self, prefix: &str, device_id: DeviceId) -> Result<IpAssignment, PortError>;
}

/// Event store operations (driven port)
#[async_trait]
pub trait EventStorePort: Send + Sync {
    /// Append events to the store
    async fn append(&self, events: Vec<NetworkEvent>) -> Result<(), PortError>;

    /// Load events for an aggregate
    async fn load_events(&self, aggregate_id: &str) -> Result<Vec<NetworkEvent>, PortError>;

    /// Subscribe to events
    async fn subscribe(&self, subject: &str) -> Result<EventSubscription, PortError>;
}

// ============================================================================
// Port Data Types
// ============================================================================

/// Device configuration (domain representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfiguration {
    pub name: Option<String>,
    pub interfaces: Vec<InterfaceConfig>,
    pub vlans: Vec<VlanConfig>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Discovered device (from discovery)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub mac: MacAddress,
    pub ip_address: Option<std::net::IpAddr>,
    pub device_type: DeviceType,
    pub model: Option<String>,
    pub vendor_id: Option<String>,
    pub adopted: bool,
}

/// Device details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDetails {
    pub device_id: DeviceId,
    pub mac: MacAddress,
    pub ip_address: Option<std::net::IpAddr>,
    pub device_type: DeviceType,
    pub model: String,
    pub firmware_version: Option<String>,
    pub uptime_seconds: Option<u64>,
    pub state: DeviceState,
}

/// Vendor-specific device representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorDevice {
    /// Vendor-specific ID
    pub vendor_id: String,
    /// Our domain device ID (if mapped)
    pub device_id: Option<DeviceId>,
    /// MAC address
    pub mac: MacAddress,
    /// Device model
    pub model: String,
    /// Device name
    pub name: String,
    /// IP address
    pub ip_address: Option<std::net::IpAddr>,
    /// Adoption state
    pub adopted: bool,
    /// Vendor-specific properties
    pub properties: HashMap<String, serde_json::Value>,
}

/// Vendor-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorConfig {
    /// Configuration type
    pub config_type: String,
    /// Configuration payload (vendor-specific JSON)
    pub payload: serde_json::Value,
}

/// Device statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStats {
    pub uptime_seconds: u64,
    pub cpu_percent: Option<f64>,
    pub memory_percent: Option<f64>,
    pub temperature_celsius: Option<f64>,
    pub port_stats: Vec<PortStats>,
}

/// Port statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortStats {
    pub port_id: PortId,
    pub link_up: bool,
    pub speed: Option<LinkSpeed>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
}

/// Connection info for inventory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub connection_id: ConnectionId,
    pub source_device: DeviceId,
    pub source_port: PortId,
    pub target_device: DeviceId,
    pub target_port: PortId,
    pub connection_type: ConnectionType,
    pub speed: Option<LinkSpeed>,
}

/// IP address assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAssignment {
    pub address: std::net::IpAddr,
    pub prefix_len: u8,
    pub device_id: Option<DeviceId>,
    pub interface: Option<String>,
    pub status: IpStatus,
}

/// IP assignment status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpStatus {
    Available,
    Reserved,
    Active,
    Deprecated,
}

/// Event subscription handle
///
/// Represents an active subscription to domain events.
/// The actual message iteration is done through adapter-specific methods.
pub struct EventSubscription {
    /// Subscription ID for tracking
    id: String,
    /// Subject pattern being subscribed to
    subject: String,
}

impl EventSubscription {
    /// Create a new event subscription
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            subject: String::new(),
        }
    }

    /// Create with specific subject
    pub fn with_subject(subject: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            subject: subject.into(),
        }
    }

    /// Get the subscription ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get the subject pattern
    pub fn subject(&self) -> &str {
        &self.subject
    }
}

impl Default for EventSubscription {
    fn default() -> Self {
        Self::new()
    }
}

use serde::{Deserialize, Serialize};
