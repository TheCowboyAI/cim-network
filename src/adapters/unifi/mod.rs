//! # UniFi Controller Adapter
//!
//! Implements network management ports for Ubiquiti UniFi equipment.
//!
//! ## Supported Operations
//!
//! - Device discovery and adoption
//! - Configuration management
//! - Statistics and monitoring
//! - Firmware upgrades
//!
//! ## API Integration
//!
//! Connects to UniFi Network Application (controller) via REST API.
//! Supports both local controllers and UniFi Cloud.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::ports::*;
use crate::domain::functor::*;
use crate::domain::events::*;
use crate::domain::value_objects::*;

mod client;
mod types;

pub use client::UniFiClient;
pub use types::*;

/// UniFi Controller adapter
///
/// Implements both:
/// - `DeviceControlPort` for hexagonal architecture
/// - `VendorExtension` for Kan extension mapping
pub struct UniFiAdapter {
    /// HTTP client for UniFi API
    client: Arc<UniFiClient>,
    /// Mapping from domain DeviceId to UniFi device ID
    device_mapping: Arc<RwLock<HashMap<DeviceId, String>>>,
    /// Mapping from UniFi device ID to domain DeviceId
    reverse_mapping: Arc<RwLock<HashMap<String, DeviceId>>>,
    /// Site ID (UniFi sites)
    site_id: String,
}

impl UniFiAdapter {
    /// Create a new UniFi adapter
    pub async fn new(
        controller_url: &str,
        username: &str,
        password: &str,
        site_id: &str,
    ) -> Result<Self, PortError> {
        let client = UniFiClient::new(controller_url, username, password)
            .await
            .map_err(|e| PortError::ConnectionFailed(e.to_string()))?;

        Ok(Self {
            client: Arc::new(client),
            device_mapping: Arc::new(RwLock::new(HashMap::new())),
            reverse_mapping: Arc::new(RwLock::new(HashMap::new())),
            site_id: site_id.to_string(),
        })
    }

    /// Map a domain device to UniFi device ID
    pub async fn map_device(&self, device_id: DeviceId, unifi_id: String) {
        let mut mapping = self.device_mapping.write().await;
        let mut reverse = self.reverse_mapping.write().await;
        mapping.insert(device_id, unifi_id.clone());
        reverse.insert(unifi_id, device_id);
    }

    /// Get UniFi device ID for a domain device
    pub async fn get_unifi_id(&self, device_id: DeviceId) -> Option<String> {
        let mapping = self.device_mapping.read().await;
        mapping.get(&device_id).cloned()
    }

    /// Get domain device ID for a UniFi device
    pub async fn get_device_id(&self, unifi_id: &str) -> Option<DeviceId> {
        let reverse = self.reverse_mapping.read().await;
        reverse.get(unifi_id).copied()
    }

    /// Convert UniFi device to vendor device representation
    fn to_vendor_device(&self, unifi_device: &UniFiDevice, device_id: Option<DeviceId>) -> VendorDevice {
        VendorDevice {
            vendor_id: unifi_device.id.clone(),
            device_id,
            mac: unifi_device.mac,
            model: unifi_device.model.clone(),
            name: unifi_device.name.clone(),
            ip_address: unifi_device.ip.map(|ip| std::net::IpAddr::V4(ip)),
            adopted: unifi_device.adopted,
            properties: unifi_device.properties.clone(),
        }
    }
}

#[async_trait]
impl DeviceControlPort for UniFiAdapter {
    fn vendor_name(&self) -> &str {
        "unifi"
    }

    async fn connect(&self) -> Result<(), PortError> {
        self.client.login()
            .await
            .map_err(|e| PortError::ConnectionFailed(e.to_string()))
    }

    async fn disconnect(&self) -> Result<(), PortError> {
        self.client.logout()
            .await
            .map_err(|e| PortError::ConnectionFailed(e.to_string()))
    }

    fn is_connected(&self) -> bool {
        self.client.is_authenticated()
    }

    async fn list_devices(&self) -> Result<Vec<VendorDevice>, PortError> {
        let unifi_devices = self.client
            .list_devices(&self.site_id)
            .await
            .map_err(|e| PortError::VendorError(e.to_string()))?;

        let mut vendor_devices = Vec::new();
        for device in unifi_devices {
            let device_id = self.get_device_id(&device.id).await;
            vendor_devices.push(self.to_vendor_device(&device, device_id));
        }

        Ok(vendor_devices)
    }

    async fn get_device(&self, vendor_id: &str) -> Result<VendorDevice, PortError> {
        let unifi_device = self.client
            .get_device(&self.site_id, vendor_id)
            .await
            .map_err(|e| PortError::VendorError(e.to_string()))?;

        let device_id = self.get_device_id(vendor_id).await;
        Ok(self.to_vendor_device(&unifi_device, device_id))
    }

    async fn adopt_device(&self, vendor_id: &str) -> Result<(), PortError> {
        self.client
            .adopt_device(&self.site_id, vendor_id)
            .await
            .map_err(|e| PortError::VendorError(e.to_string()))
    }

    async fn apply_config(&self, vendor_id: &str, config: VendorConfig) -> Result<(), PortError> {
        self.client
            .set_device_config(&self.site_id, vendor_id, &config.payload)
            .await
            .map_err(|e| PortError::VendorError(e.to_string()))
    }

    async fn restart_device(&self, vendor_id: &str) -> Result<(), PortError> {
        self.client
            .restart_device(&self.site_id, vendor_id)
            .await
            .map_err(|e| PortError::VendorError(e.to_string()))
    }

    async fn get_device_stats(&self, vendor_id: &str) -> Result<DeviceStats, PortError> {
        let stats = self.client
            .get_device_stats(&self.site_id, vendor_id)
            .await
            .map_err(|e| PortError::VendorError(e.to_string()))?;

        Ok(DeviceStats {
            uptime_seconds: stats.uptime.unwrap_or(0),
            cpu_percent: stats.cpu_usage,
            memory_percent: stats.mem_usage,
            temperature_celsius: stats.temperature,
            port_stats: stats.port_stats.into_iter().map(|ps| PortStats {
                port_id: PortId::with_index("port", ps.port_idx),
                link_up: ps.up,
                speed: ps.speed.and_then(speed_from_mbps),
                rx_bytes: ps.rx_bytes,
                tx_bytes: ps.tx_bytes,
                rx_errors: ps.rx_errors.unwrap_or(0),
                tx_errors: ps.tx_errors.unwrap_or(0),
            }).collect(),
        })
    }
}

impl VendorExtension for UniFiAdapter {
    fn vendor_name(&self) -> &str {
        "unifi"
    }

    fn extend(&self, domain_obj: &DomainObject) -> Result<VendorRepresentation, FunctorError> {
        match domain_obj {
            DomainObject::Device(device) => {
                // Create UniFi-specific representation
                let payload = serde_json::json!({
                    "type": match device.device_type() {
                        DeviceType::Gateway => "ugw",
                        DeviceType::Switch => "usw",
                        DeviceType::AccessPoint => "uap",
                        DeviceType::Generic { model } => model.as_str(),
                    },
                    "mac": device.mac().to_string(),
                    "name": device.name(),
                    "state": device.state().name(),
                });

                Ok(VendorRepresentation {
                    vendor: "unifi".to_string(),
                    vendor_id: device.vendor_id().unwrap_or("pending").to_string(),
                    device_id: device.id(),
                    payload,
                })
            }
            _ => Err(FunctorError::MappingFailed(
                "Only Device objects can be extended to UniFi".to_string()
            )),
        }
    }

    fn to_domain_event(&self, vendor_event: &serde_json::Value) -> Result<NetworkEvent, FunctorError> {
        // Parse UniFi event and convert to domain event
        let event_type = vendor_event.get("key")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FunctorError::MappingFailed("Missing event key".to_string()))?;

        match event_type {
            "EVT_AP_Connected" | "EVT_SW_Connected" | "EVT_GW_Connected" => {
                let mac_str = vendor_event.get("ap")
                    .or_else(|| vendor_event.get("sw"))
                    .or_else(|| vendor_event.get("gw"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| FunctorError::MappingFailed("Missing device MAC".to_string()))?;

                let mac = MacAddress::parse(mac_str)
                    .map_err(|e| FunctorError::MappingFailed(e.to_string()))?;

                // This would need to look up the device ID from MAC
                // For now, return a placeholder
                Ok(NetworkEvent::DeviceProvisioned {
                    device_id: DeviceId::new(), // Would need lookup
                    model: "unknown".to_string(),
                    firmware_version: "unknown".to_string(),
                })
            }
            _ => Err(FunctorError::MappingFailed(format!("Unknown event type: {}", event_type))),
        }
    }
}

/// Convert Mbps to LinkSpeed
fn speed_from_mbps(mbps: u32) -> Option<LinkSpeed> {
    match mbps {
        10 => Some(LinkSpeed::Mbps10),
        100 => Some(LinkSpeed::Mbps100),
        1000 => Some(LinkSpeed::Gbps1),
        2500 => Some(LinkSpeed::Gbps2_5),
        5000 => Some(LinkSpeed::Gbps5),
        10000 => Some(LinkSpeed::Gbps10),
        25000 => Some(LinkSpeed::Gbps25),
        40000 => Some(LinkSpeed::Gbps40),
        100000 => Some(LinkSpeed::Gbps100),
        _ => None,
    }
}
