//! # Network Domain Events
//!
//! All state changes are expressed as immutable events.
//! Events are the source of truth - aggregates are projections of events.

use crate::domain::value_objects::*;
use serde::{Deserialize, Serialize};

/// Network domain events
///
/// Every state change in the network domain produces an event.
/// Events are immutable and form the audit trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    // ========================================================================
    // Device Lifecycle Events
    // ========================================================================

    /// A new device was discovered on the network
    DeviceDiscovered {
        device_id: DeviceId,
        mac: MacAddress,
        device_type: DeviceType,
        ip_address: Option<std::net::IpAddr>,
    },

    /// Device adoption has started
    DeviceAdopting {
        device_id: DeviceId,
        vendor_id: String,
    },

    /// Device has been provisioned
    DeviceProvisioned {
        device_id: DeviceId,
        model: String,
        firmware_version: String,
    },

    /// Device configuration has started
    DeviceConfiguring {
        device_id: DeviceId,
    },

    /// Device configuration completed
    DeviceConfigured {
        device_id: DeviceId,
        interfaces: Vec<InterfaceConfig>,
        vlans: Vec<VlanConfig>,
    },

    /// Device encountered an error
    DeviceError {
        device_id: DeviceId,
        message: String,
    },

    /// Device was decommissioned
    DeviceDecommissioned {
        device_id: DeviceId,
    },

    /// Device was renamed
    DeviceRenamed {
        device_id: DeviceId,
        old_name: String,
        new_name: String,
    },

    // ========================================================================
    // Connection Events
    // ========================================================================

    /// Connection established between devices
    ConnectionEstablished {
        connection_id: ConnectionId,
        source_device: DeviceId,
        source_port: PortId,
        target_device: DeviceId,
        target_port: PortId,
        connection_type: ConnectionType,
    },

    /// Connection removed
    ConnectionRemoved {
        connection_id: ConnectionId,
    },

    /// Connection link state changed
    ConnectionLinkChanged {
        connection_id: ConnectionId,
        link_up: bool,
        speed: Option<LinkSpeed>,
    },

    // ========================================================================
    // Topology Events
    // ========================================================================

    /// Topology created
    TopologyCreated {
        topology_id: TopologyId,
        name: String,
    },

    /// Device added to topology
    DeviceAddedToTopology {
        topology_id: TopologyId,
        device_id: DeviceId,
    },

    /// Device removed from topology
    DeviceRemovedFromTopology {
        topology_id: TopologyId,
        device_id: DeviceId,
    },

    // ========================================================================
    // Inventory Projection Events
    // ========================================================================

    /// Device synced to inventory (NetBox)
    DeviceSyncedToInventory {
        device_id: DeviceId,
        inventory_id: String,
        system: String,
    },

    /// IP address allocated
    IpAddressAllocated {
        device_id: DeviceId,
        address: std::net::IpAddr,
        prefix_len: u8,
        interface: String,
    },
}

impl NetworkEvent {
    /// Get the aggregate ID this event belongs to
    pub fn aggregate_id(&self) -> String {
        match self {
            // Device events
            NetworkEvent::DeviceDiscovered { device_id, .. }
            | NetworkEvent::DeviceAdopting { device_id, .. }
            | NetworkEvent::DeviceProvisioned { device_id, .. }
            | NetworkEvent::DeviceConfiguring { device_id, .. }
            | NetworkEvent::DeviceConfigured { device_id, .. }
            | NetworkEvent::DeviceError { device_id, .. }
            | NetworkEvent::DeviceDecommissioned { device_id, .. }
            | NetworkEvent::DeviceRenamed { device_id, .. }
            | NetworkEvent::DeviceSyncedToInventory { device_id, .. }
            | NetworkEvent::IpAddressAllocated { device_id, .. } => device_id.to_string(),

            // Connection events
            NetworkEvent::ConnectionEstablished { connection_id, .. }
            | NetworkEvent::ConnectionRemoved { connection_id, .. }
            | NetworkEvent::ConnectionLinkChanged { connection_id, .. } => connection_id.to_string(),

            // Topology events
            NetworkEvent::TopologyCreated { topology_id, .. }
            | NetworkEvent::DeviceAddedToTopology { topology_id, .. }
            | NetworkEvent::DeviceRemovedFromTopology { topology_id, .. } => topology_id.to_string(),
        }
    }

    /// Get the event type name
    pub fn event_type(&self) -> &'static str {
        match self {
            NetworkEvent::DeviceDiscovered { .. } => "DeviceDiscovered",
            NetworkEvent::DeviceAdopting { .. } => "DeviceAdopting",
            NetworkEvent::DeviceProvisioned { .. } => "DeviceProvisioned",
            NetworkEvent::DeviceConfiguring { .. } => "DeviceConfiguring",
            NetworkEvent::DeviceConfigured { .. } => "DeviceConfigured",
            NetworkEvent::DeviceError { .. } => "DeviceError",
            NetworkEvent::DeviceDecommissioned { .. } => "DeviceDecommissioned",
            NetworkEvent::DeviceRenamed { .. } => "DeviceRenamed",
            NetworkEvent::ConnectionEstablished { .. } => "ConnectionEstablished",
            NetworkEvent::ConnectionRemoved { .. } => "ConnectionRemoved",
            NetworkEvent::ConnectionLinkChanged { .. } => "ConnectionLinkChanged",
            NetworkEvent::TopologyCreated { .. } => "TopologyCreated",
            NetworkEvent::DeviceAddedToTopology { .. } => "DeviceAddedToTopology",
            NetworkEvent::DeviceRemovedFromTopology { .. } => "DeviceRemovedFromTopology",
            NetworkEvent::DeviceSyncedToInventory { .. } => "DeviceSyncedToInventory",
            NetworkEvent::IpAddressAllocated { .. } => "IpAddressAllocated",
        }
    }

    /// Get NATS subject for this event
    /// Format: network.{aggregate_type}.{event_type}
    pub fn nats_subject(&self) -> String {
        let aggregate_type = match self {
            NetworkEvent::DeviceDiscovered { .. }
            | NetworkEvent::DeviceAdopting { .. }
            | NetworkEvent::DeviceProvisioned { .. }
            | NetworkEvent::DeviceConfiguring { .. }
            | NetworkEvent::DeviceConfigured { .. }
            | NetworkEvent::DeviceError { .. }
            | NetworkEvent::DeviceDecommissioned { .. }
            | NetworkEvent::DeviceRenamed { .. } => "device",

            NetworkEvent::ConnectionEstablished { .. }
            | NetworkEvent::ConnectionRemoved { .. }
            | NetworkEvent::ConnectionLinkChanged { .. } => "connection",

            NetworkEvent::TopologyCreated { .. }
            | NetworkEvent::DeviceAddedToTopology { .. }
            | NetworkEvent::DeviceRemovedFromTopology { .. } => "topology",

            NetworkEvent::DeviceSyncedToInventory { .. }
            | NetworkEvent::IpAddressAllocated { .. } => "inventory",
        };

        format!("network.{}.{}", aggregate_type, self.event_type())
    }
}
