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

use crate::domain::ports::{
    InventoryPort, PortError, ConnectionInfo as PortConnectionInfo, IpAssignment,
};
use crate::domain::functor::{
    InventoryExtension, InventoryRepresentation, DomainObject, FunctorError,
};
use crate::domain::aggregates::{NetworkDeviceAggregate, DeviceState};
use crate::domain::value_objects::{DeviceId, DeviceType, ConnectionType};

/// NetBox adapter
///
/// Implements both:
/// - `InventoryPort` for hexagonal architecture
/// - `InventoryExtension` for Kan extension mapping
pub struct NetBoxAdapter {
    /// NetBox API base URL
    base_url: String,
    /// API token
    api_token: String,
}

impl NetBoxAdapter {
    /// Create a new NetBox adapter
    pub fn new(base_url: &str, api_token: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_token: api_token.to_string(),
        }
    }
}

#[async_trait]
impl InventoryPort for NetBoxAdapter {
    fn system_name(&self) -> &str {
        "netbox"
    }

    async fn sync_device(&self, device: &NetworkDeviceAggregate) -> Result<(), PortError> {
        // TODO: Implement NetBox device sync
        // POST /api/dcim/devices/
        // or PUT /api/dcim/devices/{id}/ for updates

        tracing::info!(
            "Syncing device {} ({}) to NetBox",
            device.name(),
            device.id()
        );

        Ok(())
    }

    async fn remove_device(&self, device_id: DeviceId) -> Result<(), PortError> {
        // TODO: Implement NetBox device removal
        // DELETE /api/dcim/devices/{id}/

        tracing::info!("Removing device {} from NetBox", device_id);

        Ok(())
    }

    async fn sync_connection(&self, connection: &PortConnectionInfo) -> Result<(), PortError> {
        // TODO: Implement NetBox cable/connection sync
        // POST /api/dcim/cables/

        tracing::info!(
            "Syncing connection {} to NetBox",
            connection.connection_id
        );

        Ok(())
    }

    async fn get_ip_assignments(&self, prefix: &str) -> Result<Vec<IpAssignment>, PortError> {
        // TODO: Implement NetBox IPAM query
        // GET /api/ipam/ip-addresses/?parent={prefix}

        tracing::debug!("Getting IP assignments for prefix {}", prefix);

        Ok(Vec::new())
    }

    async fn allocate_ip(
        &self,
        prefix: &str,
        device_id: DeviceId,
    ) -> Result<IpAssignment, PortError> {
        // TODO: Implement NetBox IP allocation
        // POST /api/ipam/prefixes/{id}/available-ips/

        tracing::info!(
            "Allocating IP from {} for device {}",
            prefix,
            device_id
        );

        Err(PortError::NotSupported(
            "IP allocation not yet implemented".to_string()
        ))
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
            DomainObject::Topology(_) => {
                Err(FunctorError::MappingFailed(
                    "Topology objects map to NetBox sites, not yet implemented".to_string()
                ))
            }
        }
    }
}
