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
        self.nats_subject_with_prefix("network")
    }

    /// Get NATS subject with a custom prefix
    /// Format: {prefix}.{aggregate_type}.{event_type}
    pub fn nats_subject_with_prefix(&self, prefix: &str) -> String {
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

        format!("{}.{}.{}", prefix, aggregate_type, self.event_type())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_device_id() -> DeviceId {
        DeviceId::new()
    }

    fn create_test_mac() -> MacAddress {
        MacAddress::parse("00:11:22:33:44:55").unwrap()
    }

    // ==========================================================================
    // Device Event Tests
    // ==========================================================================

    #[test]
    fn test_device_discovered_event() {
        let device_id = create_test_device_id();
        let mac = create_test_mac();
        let event = NetworkEvent::DeviceDiscovered {
            device_id,
            mac,
            device_type: DeviceType::Switch,
            ip_address: Some("192.168.1.1".parse().unwrap()),
        };

        assert_eq!(event.event_type(), "DeviceDiscovered");
        assert_eq!(event.aggregate_id(), device_id.to_string());
    }

    #[test]
    fn test_device_adopting_event() {
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceAdopting {
            device_id,
            vendor_id: "vendor-123".to_string(),
        };

        assert_eq!(event.event_type(), "DeviceAdopting");
        assert_eq!(event.aggregate_id(), device_id.to_string());
    }

    #[test]
    fn test_device_provisioned_event() {
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceProvisioned {
            device_id,
            model: "USW-24".to_string(),
            firmware_version: "6.6.0".to_string(),
        };

        assert_eq!(event.event_type(), "DeviceProvisioned");
    }

    #[test]
    fn test_device_configuring_event() {
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceConfiguring { device_id };

        assert_eq!(event.event_type(), "DeviceConfiguring");
    }

    #[test]
    fn test_device_configured_event() {
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceConfigured {
            device_id,
            interfaces: vec![],
            vlans: vec![],
        };

        assert_eq!(event.event_type(), "DeviceConfigured");
    }

    #[test]
    fn test_device_error_event() {
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceError {
            device_id,
            message: "Connection timeout".to_string(),
        };

        assert_eq!(event.event_type(), "DeviceError");
    }

    #[test]
    fn test_device_decommissioned_event() {
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceDecommissioned { device_id };

        assert_eq!(event.event_type(), "DeviceDecommissioned");
    }

    #[test]
    fn test_device_renamed_event() {
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceRenamed {
            device_id,
            old_name: "Old-Name".to_string(),
            new_name: "New-Name".to_string(),
        };

        assert_eq!(event.event_type(), "DeviceRenamed");
    }

    // ==========================================================================
    // Connection Event Tests
    // ==========================================================================

    #[test]
    fn test_connection_established_event() {
        let connection_id = ConnectionId::new();
        let source_device = create_test_device_id();
        let target_device = create_test_device_id();
        let event = NetworkEvent::ConnectionEstablished {
            connection_id,
            source_device,
            source_port: PortId::new("eth0"),
            target_device,
            target_port: PortId::new("port1"),
            connection_type: ConnectionType::Ethernet,
        };

        assert_eq!(event.event_type(), "ConnectionEstablished");
        assert_eq!(event.aggregate_id(), connection_id.to_string());
    }

    #[test]
    fn test_connection_removed_event() {
        let connection_id = ConnectionId::new();
        let event = NetworkEvent::ConnectionRemoved { connection_id };

        assert_eq!(event.event_type(), "ConnectionRemoved");
        assert_eq!(event.aggregate_id(), connection_id.to_string());
    }

    #[test]
    fn test_connection_link_changed_event() {
        let connection_id = ConnectionId::new();
        let event = NetworkEvent::ConnectionLinkChanged {
            connection_id,
            link_up: true,
            speed: Some(LinkSpeed::Gbps1),
        };

        assert_eq!(event.event_type(), "ConnectionLinkChanged");
    }

    // ==========================================================================
    // Topology Event Tests
    // ==========================================================================

    #[test]
    fn test_topology_created_event() {
        let topology_id = TopologyId::new();
        let event = NetworkEvent::TopologyCreated {
            topology_id,
            name: "Main Network".to_string(),
        };

        assert_eq!(event.event_type(), "TopologyCreated");
        assert_eq!(event.aggregate_id(), topology_id.to_string());
    }

    #[test]
    fn test_device_added_to_topology_event() {
        let topology_id = TopologyId::new();
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceAddedToTopology {
            topology_id,
            device_id,
        };

        assert_eq!(event.event_type(), "DeviceAddedToTopology");
        assert_eq!(event.aggregate_id(), topology_id.to_string());
    }

    #[test]
    fn test_device_removed_from_topology_event() {
        let topology_id = TopologyId::new();
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceRemovedFromTopology {
            topology_id,
            device_id,
        };

        assert_eq!(event.event_type(), "DeviceRemovedFromTopology");
    }

    // ==========================================================================
    // Inventory Event Tests
    // ==========================================================================

    #[test]
    fn test_device_synced_to_inventory_event() {
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceSyncedToInventory {
            device_id,
            inventory_id: "nb-123".to_string(),
            system: "netbox".to_string(),
        };

        assert_eq!(event.event_type(), "DeviceSyncedToInventory");
        assert_eq!(event.aggregate_id(), device_id.to_string());
    }

    #[test]
    fn test_ip_address_allocated_event() {
        let device_id = create_test_device_id();
        let event = NetworkEvent::IpAddressAllocated {
            device_id,
            address: "192.168.1.100".parse().unwrap(),
            prefix_len: 24,
            interface: "eth0".to_string(),
        };

        assert_eq!(event.event_type(), "IpAddressAllocated");
    }

    // ==========================================================================
    // NATS Subject Tests
    // ==========================================================================

    #[test]
    fn test_nats_subject_device_events() {
        let device_id = create_test_device_id();
        let mac = create_test_mac();

        let event = NetworkEvent::DeviceDiscovered {
            device_id,
            mac,
            device_type: DeviceType::Switch,
            ip_address: None,
        };
        assert_eq!(event.nats_subject(), "network.device.DeviceDiscovered");

        let event = NetworkEvent::DeviceAdopting {
            device_id,
            vendor_id: "v1".to_string(),
        };
        assert_eq!(event.nats_subject(), "network.device.DeviceAdopting");
    }

    #[test]
    fn test_nats_subject_connection_events() {
        let connection_id = ConnectionId::new();
        let event = NetworkEvent::ConnectionRemoved { connection_id };
        assert_eq!(event.nats_subject(), "network.connection.ConnectionRemoved");
    }

    #[test]
    fn test_nats_subject_topology_events() {
        let topology_id = TopologyId::new();
        let event = NetworkEvent::TopologyCreated {
            topology_id,
            name: "Test".to_string(),
        };
        assert_eq!(event.nats_subject(), "network.topology.TopologyCreated");
    }

    #[test]
    fn test_nats_subject_inventory_events() {
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceSyncedToInventory {
            device_id,
            inventory_id: "id".to_string(),
            system: "netbox".to_string(),
        };
        assert_eq!(event.nats_subject(), "network.inventory.DeviceSyncedToInventory");
    }

    #[test]
    fn test_nats_subject_with_custom_prefix() {
        let device_id = create_test_device_id();
        let event = NetworkEvent::DeviceDecommissioned { device_id };
        assert_eq!(event.nats_subject_with_prefix("cim"), "cim.device.DeviceDecommissioned");
    }

    // ==========================================================================
    // Serialization Tests
    // ==========================================================================

    #[test]
    fn test_event_serialization() {
        let device_id = create_test_device_id();
        let mac = create_test_mac();
        let event = NetworkEvent::DeviceDiscovered {
            device_id,
            mac,
            device_type: DeviceType::Gateway,
            ip_address: Some("10.0.0.1".parse().unwrap()),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("DeviceDiscovered"));
        assert!(json.contains("Gateway"));

        let deserialized: NetworkEvent = serde_json::from_str(&json).unwrap();
        if let NetworkEvent::DeviceDiscovered { device_type, .. } = deserialized {
            assert_eq!(device_type, DeviceType::Gateway);
        } else {
            panic!("Expected DeviceDiscovered");
        }
    }

    #[test]
    fn test_connection_event_serialization() {
        let connection_id = ConnectionId::new();
        let event = NetworkEvent::ConnectionLinkChanged {
            connection_id,
            link_up: true,
            speed: Some(LinkSpeed::Gbps10),
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: NetworkEvent = serde_json::from_str(&json).unwrap();

        if let NetworkEvent::ConnectionLinkChanged { link_up, speed, .. } = deserialized {
            assert!(link_up);
            assert_eq!(speed, Some(LinkSpeed::Gbps10));
        } else {
            panic!("Expected ConnectionLinkChanged");
        }
    }
}
