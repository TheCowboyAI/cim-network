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
