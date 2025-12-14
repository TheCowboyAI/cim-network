//! # Network Domain Commands
//!
//! Commands represent intentions to change state.
//! Commands are validated and either accepted (producing events) or rejected.

use crate::domain::value_objects::*;
use serde::{Deserialize, Serialize};

/// Network domain commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkCommand {
    // ========================================================================
    // Device Commands
    // ========================================================================

    /// Discover devices on the network
    DiscoverDevices {
        /// Optional filter by device type
        device_type: Option<DeviceType>,
    },

    /// Adopt a discovered device
    AdoptDevice {
        device_id: DeviceId,
        vendor_id: String,
    },

    /// Configure a device
    ConfigureDevice {
        device_id: DeviceId,
        name: Option<String>,
        interfaces: Vec<InterfaceConfig>,
        vlans: Vec<VlanConfig>,
    },

    /// Rename a device
    RenameDevice {
        device_id: DeviceId,
        new_name: String,
    },

    /// Restart a device
    RestartDevice {
        device_id: DeviceId,
    },

    /// Decommission a device
    DecommissionDevice {
        device_id: DeviceId,
    },

    // ========================================================================
    // Connection Commands
    // ========================================================================

    /// Connect two devices
    ConnectDevices {
        source_device: DeviceId,
        source_port: PortId,
        target_device: DeviceId,
        target_port: PortId,
        connection_type: ConnectionType,
    },

    /// Disconnect devices
    DisconnectDevices {
        connection_id: ConnectionId,
    },

    // ========================================================================
    // Topology Commands
    // ========================================================================

    /// Create a new topology
    CreateTopology {
        name: String,
    },

    /// Add device to topology
    AddDeviceToTopology {
        topology_id: TopologyId,
        device_id: DeviceId,
    },

    /// Remove device from topology
    RemoveDeviceFromTopology {
        topology_id: TopologyId,
        device_id: DeviceId,
    },

    // ========================================================================
    // Inventory Commands
    // ========================================================================

    /// Sync device to inventory system
    SyncToInventory {
        device_id: DeviceId,
        system: String, // e.g., "netbox"
    },

    /// Allocate IP address for device
    AllocateIp {
        device_id: DeviceId,
        prefix: String,
        interface: String,
    },
}

impl NetworkCommand {
    /// Get the command type name
    pub fn command_type(&self) -> &'static str {
        match self {
            NetworkCommand::DiscoverDevices { .. } => "DiscoverDevices",
            NetworkCommand::AdoptDevice { .. } => "AdoptDevice",
            NetworkCommand::ConfigureDevice { .. } => "ConfigureDevice",
            NetworkCommand::RenameDevice { .. } => "RenameDevice",
            NetworkCommand::RestartDevice { .. } => "RestartDevice",
            NetworkCommand::DecommissionDevice { .. } => "DecommissionDevice",
            NetworkCommand::ConnectDevices { .. } => "ConnectDevices",
            NetworkCommand::DisconnectDevices { .. } => "DisconnectDevices",
            NetworkCommand::CreateTopology { .. } => "CreateTopology",
            NetworkCommand::AddDeviceToTopology { .. } => "AddDeviceToTopology",
            NetworkCommand::RemoveDeviceFromTopology { .. } => "RemoveDeviceFromTopology",
            NetworkCommand::SyncToInventory { .. } => "SyncToInventory",
            NetworkCommand::AllocateIp { .. } => "AllocateIp",
        }
    }

    /// Get NATS subject for this command
    pub fn nats_subject(&self) -> String {
        let aggregate_type = match self {
            NetworkCommand::DiscoverDevices { .. }
            | NetworkCommand::AdoptDevice { .. }
            | NetworkCommand::ConfigureDevice { .. }
            | NetworkCommand::RenameDevice { .. }
            | NetworkCommand::RestartDevice { .. }
            | NetworkCommand::DecommissionDevice { .. } => "device",

            NetworkCommand::ConnectDevices { .. }
            | NetworkCommand::DisconnectDevices { .. } => "connection",

            NetworkCommand::CreateTopology { .. }
            | NetworkCommand::AddDeviceToTopology { .. }
            | NetworkCommand::RemoveDeviceFromTopology { .. } => "topology",

            NetworkCommand::SyncToInventory { .. }
            | NetworkCommand::AllocateIp { .. } => "inventory",
        };

        format!("network.{}.cmd.{}", aggregate_type, self.command_type())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_device_id() -> DeviceId {
        DeviceId::new()
    }

    // ==========================================================================
    // Device Command Tests
    // ==========================================================================

    #[test]
    fn test_discover_devices_command() {
        let cmd = NetworkCommand::DiscoverDevices { device_type: None };
        assert_eq!(cmd.command_type(), "DiscoverDevices");
        assert_eq!(cmd.nats_subject(), "network.device.cmd.DiscoverDevices");
    }

    #[test]
    fn test_discover_devices_with_filter() {
        let cmd = NetworkCommand::DiscoverDevices {
            device_type: Some(DeviceType::Switch),
        };
        assert_eq!(cmd.command_type(), "DiscoverDevices");
    }

    #[test]
    fn test_adopt_device_command() {
        let device_id = create_test_device_id();
        let cmd = NetworkCommand::AdoptDevice {
            device_id,
            vendor_id: "vendor-123".to_string(),
        };
        assert_eq!(cmd.command_type(), "AdoptDevice");
        assert_eq!(cmd.nats_subject(), "network.device.cmd.AdoptDevice");
    }

    #[test]
    fn test_configure_device_command() {
        let device_id = create_test_device_id();
        let cmd = NetworkCommand::ConfigureDevice {
            device_id,
            name: Some("MyDevice".to_string()),
            interfaces: vec![],
            vlans: vec![],
        };
        assert_eq!(cmd.command_type(), "ConfigureDevice");
        assert_eq!(cmd.nats_subject(), "network.device.cmd.ConfigureDevice");
    }

    #[test]
    fn test_rename_device_command() {
        let device_id = create_test_device_id();
        let cmd = NetworkCommand::RenameDevice {
            device_id,
            new_name: "NewName".to_string(),
        };
        assert_eq!(cmd.command_type(), "RenameDevice");
        assert_eq!(cmd.nats_subject(), "network.device.cmd.RenameDevice");
    }

    #[test]
    fn test_restart_device_command() {
        let device_id = create_test_device_id();
        let cmd = NetworkCommand::RestartDevice { device_id };
        assert_eq!(cmd.command_type(), "RestartDevice");
        assert_eq!(cmd.nats_subject(), "network.device.cmd.RestartDevice");
    }

    #[test]
    fn test_decommission_device_command() {
        let device_id = create_test_device_id();
        let cmd = NetworkCommand::DecommissionDevice { device_id };
        assert_eq!(cmd.command_type(), "DecommissionDevice");
        assert_eq!(cmd.nats_subject(), "network.device.cmd.DecommissionDevice");
    }

    // ==========================================================================
    // Connection Command Tests
    // ==========================================================================

    #[test]
    fn test_connect_devices_command() {
        let source_device = create_test_device_id();
        let target_device = create_test_device_id();
        let cmd = NetworkCommand::ConnectDevices {
            source_device,
            source_port: PortId::new("eth0"),
            target_device,
            target_port: PortId::new("port1"),
            connection_type: ConnectionType::Ethernet,
        };
        assert_eq!(cmd.command_type(), "ConnectDevices");
        assert_eq!(cmd.nats_subject(), "network.connection.cmd.ConnectDevices");
    }

    #[test]
    fn test_disconnect_devices_command() {
        let connection_id = ConnectionId::new();
        let cmd = NetworkCommand::DisconnectDevices { connection_id };
        assert_eq!(cmd.command_type(), "DisconnectDevices");
        assert_eq!(cmd.nats_subject(), "network.connection.cmd.DisconnectDevices");
    }

    // ==========================================================================
    // Topology Command Tests
    // ==========================================================================

    #[test]
    fn test_create_topology_command() {
        let cmd = NetworkCommand::CreateTopology {
            name: "Main Network".to_string(),
        };
        assert_eq!(cmd.command_type(), "CreateTopology");
        assert_eq!(cmd.nats_subject(), "network.topology.cmd.CreateTopology");
    }

    #[test]
    fn test_add_device_to_topology_command() {
        let topology_id = TopologyId::new();
        let device_id = create_test_device_id();
        let cmd = NetworkCommand::AddDeviceToTopology {
            topology_id,
            device_id,
        };
        assert_eq!(cmd.command_type(), "AddDeviceToTopology");
        assert_eq!(cmd.nats_subject(), "network.topology.cmd.AddDeviceToTopology");
    }

    #[test]
    fn test_remove_device_from_topology_command() {
        let topology_id = TopologyId::new();
        let device_id = create_test_device_id();
        let cmd = NetworkCommand::RemoveDeviceFromTopology {
            topology_id,
            device_id,
        };
        assert_eq!(cmd.command_type(), "RemoveDeviceFromTopology");
        assert_eq!(cmd.nats_subject(), "network.topology.cmd.RemoveDeviceFromTopology");
    }

    // ==========================================================================
    // Inventory Command Tests
    // ==========================================================================

    #[test]
    fn test_sync_to_inventory_command() {
        let device_id = create_test_device_id();
        let cmd = NetworkCommand::SyncToInventory {
            device_id,
            system: "netbox".to_string(),
        };
        assert_eq!(cmd.command_type(), "SyncToInventory");
        assert_eq!(cmd.nats_subject(), "network.inventory.cmd.SyncToInventory");
    }

    #[test]
    fn test_allocate_ip_command() {
        let device_id = create_test_device_id();
        let cmd = NetworkCommand::AllocateIp {
            device_id,
            prefix: "192.168.1.0/24".to_string(),
            interface: "eth0".to_string(),
        };
        assert_eq!(cmd.command_type(), "AllocateIp");
        assert_eq!(cmd.nats_subject(), "network.inventory.cmd.AllocateIp");
    }

    // ==========================================================================
    // Serialization Tests
    // ==========================================================================

    #[test]
    fn test_command_serialization() {
        let device_id = create_test_device_id();
        let cmd = NetworkCommand::AdoptDevice {
            device_id,
            vendor_id: "vendor-123".to_string(),
        };

        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains("AdoptDevice"));
        assert!(json.contains("vendor-123"));

        let deserialized: NetworkCommand = serde_json::from_str(&json).unwrap();
        if let NetworkCommand::AdoptDevice { vendor_id, .. } = deserialized {
            assert_eq!(vendor_id, "vendor-123");
        } else {
            panic!("Expected AdoptDevice");
        }
    }

    #[test]
    fn test_configure_device_serialization() {
        let device_id = create_test_device_id();
        let cmd = NetworkCommand::ConfigureDevice {
            device_id,
            name: Some("Test Device".to_string()),
            interfaces: vec![InterfaceConfig {
                name: "eth0".to_string(),
                ip_address: Some("192.168.1.10".parse().unwrap()),
                prefix_len: Some(24),
                vlan_id: Some(100),
                enabled: true,
            }],
            vlans: vec![VlanConfig::new(100, "Management").unwrap()],
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: NetworkCommand = serde_json::from_str(&json).unwrap();

        if let NetworkCommand::ConfigureDevice { name, interfaces, vlans, .. } = deserialized {
            assert_eq!(name, Some("Test Device".to_string()));
            assert_eq!(interfaces.len(), 1);
            assert_eq!(vlans.len(), 1);
        } else {
            panic!("Expected ConfigureDevice");
        }
    }

    #[test]
    fn test_connect_devices_serialization() {
        let source_device = create_test_device_id();
        let target_device = create_test_device_id();
        let cmd = NetworkCommand::ConnectDevices {
            source_device,
            source_port: PortId::with_index("port", 1),
            target_device,
            target_port: PortId::with_index("port", 24),
            connection_type: ConnectionType::Fiber,
        };

        let json = serde_json::to_string(&cmd).unwrap();
        let deserialized: NetworkCommand = serde_json::from_str(&json).unwrap();

        if let NetworkCommand::ConnectDevices { connection_type, source_port, .. } = deserialized {
            assert_eq!(connection_type, ConnectionType::Fiber);
            assert_eq!(source_port.index, Some(1));
        } else {
            panic!("Expected ConnectDevices");
        }
    }
}
