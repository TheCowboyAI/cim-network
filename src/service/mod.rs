//! # Network Service Layer
//!
//! Application services that orchestrate domain operations across adapters.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                      NetworkService                                 │
//! │  ┌─────────────────────────────────────────────────────────────┐   │
//! │  │                    Orchestration                             │   │
//! │  │  Discovery → Adoption → Provisioning → Inventory Sync       │   │
//! │  └─────────────────────────────────────────────────────────────┘   │
//! │                              │                                      │
//! │        ┌────────────────────┼────────────────────┐                 │
//! │        ▼                    ▼                    ▼                 │
//! │  ┌──────────┐        ┌──────────┐        ┌──────────┐             │
//! │  │  Event   │        │  Vendor  │        │ Inventory│             │
//! │  │  Store   │        │ Adapter  │        │ Adapter  │             │
//! │  │  (NATS)  │        │ (UniFi)  │        │ (NetBox) │             │
//! │  └──────────┘        └──────────┘        └──────────┘             │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use cim_network::service::NetworkService;
//!
//! let service = NetworkService::builder()
//!     .event_store(nats_store)
//!     .vendor_adapter(unifi_adapter)
//!     .inventory_adapter(netbox_adapter)
//!     .build()
//!     .await?;
//!
//! // Discover and provision devices
//! let devices = service.discover_and_provision().await?;
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::aggregates::{NetworkDeviceAggregate, DeviceState};
use crate::domain::events::NetworkEvent;
use crate::domain::value_objects::{DeviceId, DeviceType, MacAddress};
use crate::domain::ports::{
    DeviceControlPort, InventoryPort, EventStorePort, PortError,
};

/// Network service for orchestrating domain operations
///
/// This service coordinates between:
/// - Event Store (NATS JetStream) for persistence
/// - Vendor Adapter (UniFi, etc.) for device control
/// - Inventory Adapter (NetBox) for documentation
pub struct NetworkService {
    /// Event store for persistence
    event_store: Arc<dyn EventStorePort>,
    /// Vendor adapter for device control
    vendor_adapter: Arc<dyn DeviceControlPort>,
    /// Optional inventory adapter
    inventory_adapter: Option<Arc<dyn InventoryPort>>,
    /// In-memory device cache (aggregate_id -> aggregate)
    devices: Arc<RwLock<HashMap<DeviceId, NetworkDeviceAggregate>>>,
}

impl NetworkService {
    /// Create a new network service builder
    pub fn builder() -> NetworkServiceBuilder {
        NetworkServiceBuilder::new()
    }

    /// Discover devices from the vendor controller
    ///
    /// Queries the vendor adapter for all devices and creates domain aggregates
    /// for any new devices found. Events are persisted to the event store.
    pub async fn discover_devices(&self) -> Result<Vec<DeviceId>, PortError> {
        tracing::info!("Starting device discovery via {}", self.vendor_adapter.vendor_name());

        // Get devices from vendor
        let vendor_devices = self.vendor_adapter.list_devices().await?;
        let mut discovered_ids = Vec::new();

        for vendor_device in vendor_devices {
            // Check if we already know this device
            let existing = self.find_device_by_mac(&vendor_device.mac).await;

            if existing.is_none() {
                // Create new domain aggregate
                let device_type = infer_device_type(&vendor_device.model);
                let mut aggregate = NetworkDeviceAggregate::new_discovered(
                    vendor_device.mac,
                    device_type,
                    vendor_device.ip_address,
                );

                // Set name if available
                if !vendor_device.name.is_empty() {
                    let _ = aggregate.rename(vendor_device.name.clone());
                }

                // Persist events
                let events = aggregate.take_pending_events();
                if !events.is_empty() {
                    self.event_store.append(events).await?;
                }

                let device_id = aggregate.id();
                discovered_ids.push(device_id);

                // Cache the aggregate
                let mut devices = self.devices.write().await;
                devices.insert(device_id, aggregate);

                tracing::info!(
                    "Discovered device {} ({}) - {}",
                    vendor_device.name,
                    vendor_device.mac,
                    device_id
                );
            }
        }

        tracing::info!("Discovery complete: {} new devices", discovered_ids.len());
        Ok(discovered_ids)
    }

    /// Adopt a device through the vendor controller
    ///
    /// Transitions the device from Discovered to Adopting state,
    /// then triggers adoption via the vendor adapter.
    pub async fn adopt_device(&self, device_id: DeviceId) -> Result<(), PortError> {
        let mut devices = self.devices.write().await;
        let aggregate = devices.get_mut(&device_id)
            .ok_or_else(|| PortError::DeviceNotFound(device_id))?;

        // Get vendor ID (MAC address for UniFi)
        let vendor_id = aggregate.mac().to_string();

        // Transition to adopting state
        aggregate.adopt(vendor_id.clone())
            .map_err(|e| PortError::VendorError(e.to_string()))?;

        // Persist the state change
        let events = aggregate.take_pending_events();
        self.event_store.append(events).await?;

        // Trigger adoption via vendor adapter
        self.vendor_adapter.adopt_device(&vendor_id).await?;

        tracing::info!("Device {} adoption initiated", device_id);
        Ok(())
    }

    /// Mark a device as provisioned
    ///
    /// Called when the vendor confirms the device is fully adopted.
    pub async fn mark_provisioned(
        &self,
        device_id: DeviceId,
        model: String,
        firmware_version: String,
    ) -> Result<(), PortError> {
        let mut devices = self.devices.write().await;
        let aggregate = devices.get_mut(&device_id)
            .ok_or_else(|| PortError::DeviceNotFound(device_id))?;

        aggregate.mark_provisioned(model, firmware_version)
            .map_err(|e| PortError::VendorError(e.to_string()))?;

        // Persist events
        let events = aggregate.take_pending_events();
        self.event_store.append(events).await?;

        // Sync to inventory if configured
        if let Some(ref inventory) = self.inventory_adapter {
            inventory.sync_device(aggregate).await?;
            tracing::info!("Device {} synced to inventory", device_id);
        }

        tracing::info!("Device {} provisioned", device_id);
        Ok(())
    }

    /// Sync a device to inventory
    pub async fn sync_to_inventory(&self, device_id: DeviceId) -> Result<(), PortError> {
        let inventory = self.inventory_adapter.as_ref()
            .ok_or_else(|| PortError::NotSupported("No inventory adapter configured".to_string()))?;

        let devices = self.devices.read().await;
        let aggregate = devices.get(&device_id)
            .ok_or_else(|| PortError::DeviceNotFound(device_id))?;

        inventory.sync_device(aggregate).await?;

        // Record the sync event
        let event = NetworkEvent::DeviceSyncedToInventory {
            device_id,
            inventory_id: format!("{}-{}", inventory.system_name(), device_id),
            system: inventory.system_name().to_string(),
        };
        self.event_store.append(vec![event]).await?;

        Ok(())
    }

    /// Decommission a device
    pub async fn decommission_device(&self, device_id: DeviceId) -> Result<(), PortError> {
        let mut devices = self.devices.write().await;
        let aggregate = devices.get_mut(&device_id)
            .ok_or_else(|| PortError::DeviceNotFound(device_id))?;

        aggregate.decommission()
            .map_err(|e| PortError::VendorError(e.to_string()))?;

        // Persist events
        let events = aggregate.take_pending_events();
        self.event_store.append(events).await?;

        // Remove from inventory
        if let Some(ref inventory) = self.inventory_adapter {
            let _ = inventory.remove_device(device_id).await;
        }

        tracing::info!("Device {} decommissioned", device_id);
        Ok(())
    }

    /// Get a device by ID
    pub async fn get_device(&self, device_id: DeviceId) -> Option<NetworkDeviceAggregate> {
        let devices = self.devices.read().await;
        devices.get(&device_id).cloned()
    }

    /// List all devices
    pub async fn list_devices(&self) -> Vec<NetworkDeviceAggregate> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }

    /// List devices by state
    pub async fn list_devices_by_state(&self, state: DeviceState) -> Vec<NetworkDeviceAggregate> {
        let devices = self.devices.read().await;
        devices.values()
            .filter(|d| d.state() == state)
            .cloned()
            .collect()
    }

    /// Find device by MAC address
    async fn find_device_by_mac(&self, mac: &MacAddress) -> Option<DeviceId> {
        let devices = self.devices.read().await;
        devices.values()
            .find(|d| d.mac() == *mac)
            .map(|d| d.id())
    }

    /// Replay events from the event store to rebuild state
    pub async fn replay_events(&self, aggregate_id: &str) -> Result<Option<NetworkDeviceAggregate>, PortError> {
        let events = self.event_store.load_events(aggregate_id).await?;

        if events.is_empty() {
            return Ok(None);
        }

        // Reconstruct aggregate from events
        let mut aggregate: Option<NetworkDeviceAggregate> = None;

        for event in events {
            match event {
                NetworkEvent::DeviceDiscovered { device_id, mac, device_type, ip_address } => {
                    aggregate = Some(NetworkDeviceAggregate::from_discovered_event(
                        device_id, mac, device_type, ip_address,
                    ));
                }
                NetworkEvent::DeviceAdopting { vendor_id, .. } => {
                    if let Some(ref mut agg) = aggregate {
                        let _ = agg.adopt(vendor_id);
                        agg.take_pending_events(); // Discard during replay
                    }
                }
                NetworkEvent::DeviceProvisioned { model, firmware_version, .. } => {
                    if let Some(ref mut agg) = aggregate {
                        let _ = agg.mark_provisioned(model, firmware_version);
                        agg.take_pending_events();
                    }
                }
                NetworkEvent::DeviceDecommissioned { .. } => {
                    if let Some(ref mut agg) = aggregate {
                        let _ = agg.decommission();
                        agg.take_pending_events();
                    }
                }
                NetworkEvent::DeviceRenamed { new_name, .. } => {
                    if let Some(ref mut agg) = aggregate {
                        let _ = agg.rename(new_name);
                        agg.take_pending_events();
                    }
                }
                _ => {} // Other events don't affect device aggregate
            }
        }

        // Cache the reconstructed aggregate
        if let Some(ref agg) = aggregate {
            let mut devices = self.devices.write().await;
            devices.insert(agg.id(), agg.clone());
        }

        Ok(aggregate)
    }

    /// Full discovery and provisioning workflow
    ///
    /// 1. Discover devices from vendor
    /// 2. Adopt any unadopted devices
    /// 3. Sync all to inventory
    pub async fn discover_and_provision(&self) -> Result<Vec<DeviceId>, PortError> {
        // Step 1: Discover
        let discovered = self.discover_devices().await?;

        // Step 2: Adopt discovered devices
        for device_id in &discovered {
            if let Err(e) = self.adopt_device(*device_id).await {
                tracing::warn!("Failed to adopt device {}: {}", device_id, e);
            }
        }

        // Step 3: Sync all devices to inventory
        if self.inventory_adapter.is_some() {
            let devices = self.list_devices().await;
            for device in devices {
                if let Err(e) = self.sync_to_inventory(device.id()).await {
                    tracing::warn!("Failed to sync device {} to inventory: {}", device.id(), e);
                }
            }
        }

        Ok(discovered)
    }
}

/// Builder for NetworkService
pub struct NetworkServiceBuilder {
    event_store: Option<Arc<dyn EventStorePort>>,
    vendor_adapter: Option<Arc<dyn DeviceControlPort>>,
    inventory_adapter: Option<Arc<dyn InventoryPort>>,
}

impl NetworkServiceBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            event_store: None,
            vendor_adapter: None,
            inventory_adapter: None,
        }
    }

    /// Set the event store
    pub fn event_store<E: EventStorePort + 'static>(mut self, store: E) -> Self {
        self.event_store = Some(Arc::new(store));
        self
    }

    /// Set the event store from Arc
    pub fn event_store_arc(mut self, store: Arc<dyn EventStorePort>) -> Self {
        self.event_store = Some(store);
        self
    }

    /// Set the vendor adapter
    pub fn vendor_adapter<V: DeviceControlPort + 'static>(mut self, adapter: V) -> Self {
        self.vendor_adapter = Some(Arc::new(adapter));
        self
    }

    /// Set the vendor adapter from Arc
    pub fn vendor_adapter_arc(mut self, adapter: Arc<dyn DeviceControlPort>) -> Self {
        self.vendor_adapter = Some(adapter);
        self
    }

    /// Set the inventory adapter
    pub fn inventory_adapter<I: InventoryPort + 'static>(mut self, adapter: I) -> Self {
        self.inventory_adapter = Some(Arc::new(adapter));
        self
    }

    /// Set the inventory adapter from Arc
    pub fn inventory_adapter_arc(mut self, adapter: Arc<dyn InventoryPort>) -> Self {
        self.inventory_adapter = Some(adapter);
        self
    }

    /// Build the service
    pub fn build(self) -> Result<NetworkService, PortError> {
        let event_store = self.event_store
            .ok_or_else(|| PortError::NotSupported("Event store is required".to_string()))?;

        let vendor_adapter = self.vendor_adapter
            .ok_or_else(|| PortError::NotSupported("Vendor adapter is required".to_string()))?;

        Ok(NetworkService {
            event_store,
            vendor_adapter,
            inventory_adapter: self.inventory_adapter,
            devices: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

impl Default for NetworkServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Infer device type from model string
fn infer_device_type(model: &str) -> DeviceType {
    let model_lower = model.to_lowercase();

    if model_lower.contains("gateway") || model_lower.contains("ugw") || model_lower.contains("udm") {
        DeviceType::Gateway
    } else if model_lower.contains("switch") || model_lower.contains("usw") {
        DeviceType::Switch
    } else if model_lower.contains("ap") || model_lower.contains("uap") || model_lower.contains("u6") {
        DeviceType::AccessPoint
    } else {
        DeviceType::Generic { model: model.to_string() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_device_type() {
        assert!(matches!(infer_device_type("USW-24-POE"), DeviceType::Switch));
        assert!(matches!(infer_device_type("UAP-AC-Pro"), DeviceType::AccessPoint));
        assert!(matches!(infer_device_type("UDM-Pro"), DeviceType::Gateway));
        assert!(matches!(infer_device_type("U6-Pro"), DeviceType::AccessPoint));
        assert!(matches!(infer_device_type("Unknown"), DeviceType::Generic { .. }));
    }
}
