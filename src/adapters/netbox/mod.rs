//! # NetBox Adapter
//!
//! Implements inventory projection to NetBox DCIM/IPAM.
//!
//! ## Features
//!
//! - Device synchronization to NetBox devices
//! - Interface/connection documentation
//! - IP address management (IPAM)
//! - Site and rack management
//!
//! ## Kan Extension Integration
//!
//! Implements `InventoryExtension` for categorical projection
//! from domain objects to NetBox representations.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

mod client;
mod types;

pub use client::NetBoxClient;
pub use types::*;

use crate::domain::ports::{
    InventoryPort, PortError, ConnectionInfo as PortConnectionInfo, IpAssignment, IpStatus,
};
use crate::domain::functor::{
    InventoryExtension, InventoryRepresentation, DomainObject, FunctorError,
};
use crate::domain::aggregates::{NetworkDeviceAggregate, DeviceState};
use crate::domain::value_objects::{DeviceId, DeviceType, ConnectionType};

/// NetBox adapter configuration
pub struct NetBoxConfig {
    /// Default site ID for new devices
    pub default_site_id: u64,
    /// Default device role ID
    pub default_role_id: u64,
    /// Device type mappings (model name -> NetBox device_type ID)
    pub device_type_mappings: HashMap<String, u64>,
}

impl Default for NetBoxConfig {
    fn default() -> Self {
        Self {
            default_site_id: 1,
            default_role_id: 1,
            device_type_mappings: HashMap::new(),
        }
    }
}

/// NetBox adapter
///
/// Implements both:
/// - `InventoryPort` for hexagonal architecture
/// - `InventoryExtension` for Kan extension mapping
pub struct NetBoxAdapter {
    /// HTTP client
    client: NetBoxClient,
    /// Configuration
    config: NetBoxConfig,
    /// Cache of device_id -> netbox_id mappings
    device_cache: RwLock<HashMap<DeviceId, u64>>,
}

impl NetBoxAdapter {
    /// Create a new NetBox adapter
    pub fn new(base_url: &str, api_token: &str) -> Result<Self, NetBoxError> {
        Ok(Self {
            client: NetBoxClient::new(base_url, api_token)?,
            config: NetBoxConfig::default(),
            device_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Create with custom configuration
    pub fn with_config(base_url: &str, api_token: &str, config: NetBoxConfig) -> Result<Self, NetBoxError> {
        Ok(Self {
            client: NetBoxClient::new(base_url, api_token)?,
            config,
            device_cache: RwLock::new(HashMap::new()),
        })
    }

    /// Get the underlying client for advanced operations
    pub fn client(&self) -> &NetBoxClient {
        &self.client
    }

    /// Get device type ID for a model, using config mappings
    fn get_device_type_id(&self, device_type: &DeviceType) -> u64 {
        let model_name = match device_type {
            DeviceType::Gateway => "Gateway",
            DeviceType::Switch => "Switch",
            DeviceType::AccessPoint => "Access Point",
            DeviceType::Generic { model } => model.as_str(),
        };

        self.config.device_type_mappings
            .get(model_name)
            .copied()
            .unwrap_or(1) // Default device type ID
    }

    /// Get cached NetBox ID for a device
    fn get_cached_netbox_id(&self, device_id: &DeviceId) -> Option<u64> {
        self.device_cache.read()
            .ok()
            .and_then(|cache| cache.get(device_id).copied())
    }

    /// Cache a NetBox ID for a device
    fn cache_netbox_id(&self, device_id: DeviceId, netbox_id: u64) {
        if let Ok(mut cache) = self.device_cache.write() {
            cache.insert(device_id, netbox_id);
        }
    }

    /// Remove a device from cache
    fn uncache_device(&self, device_id: &DeviceId) {
        if let Ok(mut cache) = self.device_cache.write() {
            cache.remove(device_id);
        }
    }
}

#[async_trait]
impl InventoryPort for NetBoxAdapter {
    fn system_name(&self) -> &str {
        "netbox"
    }

    async fn sync_device(&self, device: &NetworkDeviceAggregate) -> Result<(), PortError> {
        tracing::info!(
            "Syncing device {} ({}) to NetBox",
            device.name(),
            device.id()
        );

        // Check if device already exists in NetBox
        let existing = self.client.get_device_by_name(device.name())
            .await
            .map_err(|e| PortError::InventoryError(e.to_string()))?;

        let status = match device.state() {
            DeviceState::Provisioned => "active",
            DeviceState::Discovered => "planned",
            DeviceState::Configuring => "staged",
            DeviceState::Error => "failed",
            DeviceState::Decommissioned => "decommissioning",
            _ => "inventory",
        };

        let custom_fields = serde_json::json!({
            "mac_address": device.mac().to_string(),
            "cim_device_id": device.id().to_string(),
        });

        if let Some(existing_device) = existing {
            // Update existing device
            let update = serde_json::json!({
                "status": status,
                "custom_fields": custom_fields,
            });

            self.client.update_device(existing_device.id, &update)
                .await
                .map_err(|e| PortError::InventoryError(e.to_string()))?;

            self.cache_netbox_id(device.id(), existing_device.id);
        } else {
            // Create new device
            let create = NetBoxDeviceCreate {
                name: device.name().to_string(),
                device_type: self.get_device_type_id(device.device_type()),
                site: self.config.default_site_id,
                role: self.config.default_role_id,
                status: Some(status.to_string()),
                serial: None,
                custom_fields: Some(custom_fields),
            };

            let created = self.client.create_device(&create)
                .await
                .map_err(|e| PortError::InventoryError(e.to_string()))?;

            self.cache_netbox_id(device.id(), created.id);
        }

        Ok(())
    }

    async fn remove_device(&self, device_id: DeviceId) -> Result<(), PortError> {
        tracing::info!("Removing device {} from NetBox", device_id);

        // Try to find NetBox ID from cache or by searching
        let netbox_id = if let Some(id) = self.get_cached_netbox_id(&device_id) {
            id
        } else {
            // Search by custom field
            // Note: This is a simplified approach; real implementation would search properly
            return Err(PortError::InventoryError(
                "Device not found in NetBox cache".to_string()
            ));
        };

        self.client.delete_device(netbox_id)
            .await
            .map_err(|e| PortError::InventoryError(e.to_string()))?;

        self.uncache_device(&device_id);

        Ok(())
    }

    async fn sync_connection(&self, connection: &PortConnectionInfo) -> Result<(), PortError> {
        tracing::info!(
            "Syncing connection {} to NetBox",
            connection.connection_id
        );

        let cable_type = match connection.connection_type {
            ConnectionType::Ethernet => Some("cat6".to_string()),
            ConnectionType::Fiber => Some("smf".to_string()),
            _ => None,
        };

        // Note: This requires interface IDs, which would need to be looked up
        // This is a simplified implementation
        let cable = NetBoxCableCreate {
            a_terminations: vec![NetBoxTermination {
                object_type: "dcim.interface".to_string(),
                object_id: connection.source_port.index.unwrap_or(0) as u64,
            }],
            b_terminations: vec![NetBoxTermination {
                object_type: "dcim.interface".to_string(),
                object_id: connection.target_port.index.unwrap_or(0) as u64,
            }],
            cable_type,
            status: Some("connected".to_string()),
            label: Some(connection.connection_id.to_string()),
        };

        self.client.create_cable(&cable)
            .await
            .map_err(|e| PortError::InventoryError(e.to_string()))?;

        Ok(())
    }

    async fn get_ip_assignments(&self, prefix: &str) -> Result<Vec<IpAssignment>, PortError> {
        tracing::debug!("Getting IP assignments for prefix {}", prefix);

        let ips = self.client.get_ip_addresses(prefix)
            .await
            .map_err(|e| PortError::InventoryError(e.to_string()))?;

        let assignments = ips.into_iter()
            .map(|ip| {
                // Parse address to extract IP and prefix length
                let parts: Vec<&str> = ip.address.split('/').collect();
                let address = parts[0].parse().unwrap_or_else(|_| "0.0.0.0".parse().unwrap());

                IpAssignment {
                    address,
                    prefix_len: parts.get(1)
                        .and_then(|p| p.parse().ok())
                        .unwrap_or(24),
                    device_id: None, // Would need to look up from assigned_object
                    interface: ip.description.clone(),
                    status: ip.status
                        .map(|s| match s.value.as_str() {
                            "active" => IpStatus::Active,
                            "reserved" => IpStatus::Reserved,
                            "deprecated" => IpStatus::Deprecated,
                            _ => IpStatus::Active,
                        })
                        .unwrap_or(IpStatus::Active),
                }
            })
            .collect();

        Ok(assignments)
    }

    async fn allocate_ip(
        &self,
        prefix: &str,
        device_id: DeviceId,
    ) -> Result<IpAssignment, PortError> {
        tracing::info!(
            "Allocating IP from {} for device {}",
            prefix,
            device_id
        );

        // Find the prefix
        let netbox_prefix = self.client.get_prefix(prefix)
            .await
            .map_err(|e| PortError::InventoryError(e.to_string()))?
            .ok_or_else(|| PortError::InventoryError(format!("Prefix {} not found", prefix)))?;

        let allocation = NetBoxIpAllocate {
            description: Some(format!("Allocated for device {}", device_id)),
            status: Some("active".to_string()),
            assigned_object_type: None,
            assigned_object_id: None,
        };

        let ip = self.client.allocate_ip(netbox_prefix.id, &allocation)
            .await
            .map_err(|e| PortError::InventoryError(e.to_string()))?;

        // Parse the allocated address
        let parts: Vec<&str> = ip.address.split('/').collect();
        let address = parts[0].parse()
            .map_err(|e| PortError::InventoryError(format!("Invalid IP address: {}", e)))?;

        Ok(IpAssignment {
            address,
            prefix_len: parts.get(1)
                .and_then(|p| p.parse().ok())
                .unwrap_or(24),
            device_id: Some(device_id),
            interface: None,
            status: IpStatus::Active,
        })
    }
}

impl InventoryExtension for NetBoxAdapter {
    fn system_name(&self) -> &str {
        "netbox"
    }

    fn extend(&self, domain_obj: &DomainObject) -> Result<InventoryRepresentation, FunctorError> {
        match domain_obj {
            DomainObject::Device(device) => {
                // Create NetBox device representation
                let payload = serde_json::json!({
                    "name": device.name(),
                    "device_type": {
                        "model": match device.device_type() {
                            DeviceType::Gateway => "Gateway",
                            DeviceType::Switch => "Switch",
                            DeviceType::AccessPoint => "Access Point",
                            DeviceType::Generic { model } => model.as_str(),
                        }
                    },
                    "status": match device.state() {
                        DeviceState::Provisioned => "active",
                        DeviceState::Discovered => "planned",
                        DeviceState::Configuring => "staged",
                        DeviceState::Error => "failed",
                        DeviceState::Decommissioned => "decommissioning",
                        _ => "inventory",
                    },
                    "primary_ip4": device.ip_address().map(|ip| ip.to_string()),
                    "custom_fields": {
                        "mac_address": device.mac().to_string(),
                        "cim_device_id": device.id().to_string(),
                    }
                });

                Ok(InventoryRepresentation {
                    system: "netbox".to_string(),
                    inventory_id: format!("netbox-{}", device.id()),
                    device_id: device.id(),
                    payload,
                })
            }
            DomainObject::Connection(conn) => {
                // Create NetBox cable representation
                let payload = serde_json::json!({
                    "a_terminations": [{
                        "object_type": "dcim.interface",
                        "object_id": conn.source_port.to_string(),
                    }],
                    "b_terminations": [{
                        "object_type": "dcim.interface",
                        "object_id": conn.target_port.to_string(),
                    }],
                    "type": match conn.connection_type {
                        ConnectionType::Ethernet => "cat6",
                        ConnectionType::Fiber => "smf",
                        _ => "other",
                    },
                    "status": "connected",
                });

                Ok(InventoryRepresentation {
                    system: "netbox".to_string(),
                    inventory_id: format!("netbox-cable-{}", conn.id),
                    device_id: conn.source_device, // Use source device as reference
                    payload,
                })
            }
            DomainObject::Topology(topology) => {
                // Map topology to NetBox site
                let payload = serde_json::json!({
                    "name": topology.name,
                    "slug": topology.name.to_lowercase().replace(' ', "-"),
                    "description": format!("CIM Topology {}", topology.id),
                    "custom_fields": {
                        "cim_topology_id": topology.id.to_string(),
                    }
                });

                Ok(InventoryRepresentation {
                    system: "netbox".to_string(),
                    inventory_id: format!("netbox-site-{}", topology.id),
                    device_id: DeviceId::new(), // No single device for topology
                    payload,
                })
            }
        }
    }
}
