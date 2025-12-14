//! Infrastructure Bridge Module
//!
//! Provides integration between cim-network and cim-domain-infrastructure,
//! enabling network devices to be represented as compute resources when needed.
//!
//! ## Design Rationale
//!
//! Network devices (switches, routers, APs) are distinct from general compute
//! resources (servers, VMs, containers), but there is overlap:
//!
//! - A switch is a specialized compute resource with network management capabilities
//! - A server with network interfaces can be represented in both domains
//! - Unified infrastructure views require mapping between these representations
//!
//! This module provides:
//! - Bidirectional conversion between `NetworkDeviceAggregate` and `ComputeResource`
//! - Type mappings between network and infrastructure value objects
//! - Extension traits for cross-domain operations

use crate::domain::aggregates::NetworkDeviceAggregate;
use crate::domain::value_objects::{DeviceType, DeviceId, MacAddress};
use cim_domain_infrastructure::{
    ComputeResource, ComputeResourceSpec, ComputeType, Hostname, InterfaceId,
    MessageIdentity, ResourceCapabilities, ResourceId, SystemArchitecture,
    SystemDescription, OperatingSystem, NixCapability, ManagementProtocol,
};
use std::collections::HashMap;

/// Errors that can occur during infrastructure bridge operations
#[derive(Debug, thiserror::Error)]
pub enum BridgeError {
    /// Invalid hostname for conversion
    #[error("Invalid hostname: {0}")]
    InvalidHostname(String),

    /// Invalid resource ID
    #[error("Invalid resource ID: {0}")]
    InvalidResourceId(String),

    /// Conversion not supported for this device type
    #[error("Conversion not supported for device type: {0}")]
    UnsupportedDeviceType(String),
}

/// Extension trait for converting NetworkDeviceAggregate to infrastructure types
pub trait InfrastructureBridge {
    /// Convert network device to a ComputeResourceSpec for registration
    fn to_compute_resource_spec(&self) -> Result<ComputeResourceSpec, BridgeError>;

    /// Convert network device to a ComputeResource entity
    fn to_compute_resource(&self) -> Result<ComputeResource, BridgeError>;

    /// Get infrastructure-compatible resource ID
    fn to_resource_id(&self) -> Result<ResourceId, BridgeError>;

    /// Get the system description for this network device
    fn to_system_description(&self) -> SystemDescription;
}

impl InfrastructureBridge for NetworkDeviceAggregate {
    fn to_compute_resource_spec(&self) -> Result<ComputeResourceSpec, BridgeError> {
        let resource_id = ResourceId::new(&format!("net-{}", self.id()))
            .map_err(|e| BridgeError::InvalidResourceId(e.to_string()))?;

        let hostname = Hostname::new(self.name())
            .map_err(|e| BridgeError::InvalidHostname(e.to_string()))?;

        let compute_type = device_type_to_compute_type(self.device_type());
        let system_desc = self.to_system_description();

        let mut capabilities = ResourceCapabilities::new();

        // Add network-specific metadata
        capabilities.metadata.insert(
            "mac_address".to_string(),
            self.mac().to_string(),
        );
        capabilities.metadata.insert(
            "device_type".to_string(),
            format!("{}", self.device_type()),
        );
        capabilities.metadata.insert(
            "device_state".to_string(),
            self.state().name().to_string(),
        );

        if let Some(vendor_id) = self.vendor_id() {
            capabilities.metadata.insert("vendor_id".to_string(), vendor_id.to_string());
        }

        if let Some(ip) = self.ip_address() {
            capabilities.metadata.insert("ip_address".to_string(), ip.to_string());
        }

        Ok(ComputeResourceSpec {
            id: resource_id,
            resource_type: compute_type,
            hostname,
            system: SystemArchitecture::x86_64_linux(), // Network devices typically embedded Linux
            system_description: Some(system_desc),
            capabilities,
        })
    }

    fn to_compute_resource(&self) -> Result<ComputeResource, BridgeError> {
        let spec = self.to_compute_resource_spec()?;

        // Map network interfaces to infrastructure interface IDs
        let interfaces: Vec<InterfaceId> = self
            .interfaces()
            .iter()
            .filter_map(|iface| InterfaceId::new(&iface.name).ok())
            .collect();

        Ok(ComputeResource {
            id: spec.id,
            resource_type: spec.resource_type,
            hostname: spec.hostname,
            system: spec.system,
            system_description: spec.system_description,
            capabilities: spec.capabilities,
            interfaces,
            services: vec![],
            guests: vec![],
        })
    }

    fn to_resource_id(&self) -> Result<ResourceId, BridgeError> {
        ResourceId::new(&format!("net-{}", self.id()))
            .map_err(|e| BridgeError::InvalidResourceId(e.to_string()))
    }

    fn to_system_description(&self) -> SystemDescription {
        // Network devices are typically embedded Linux systems
        // with SNMP or REST API management
        let os = match self.device_type() {
            DeviceType::Gateway => OperatingSystem::Linux,
            DeviceType::Switch => OperatingSystem::Linux,
            DeviceType::AccessPoint => OperatingSystem::Linux,
            DeviceType::Generic { .. } => OperatingSystem::Unknown,
        };

        let management = match self.device_type() {
            DeviceType::Gateway => vec![ManagementProtocol::RestApi, ManagementProtocol::Snmp],
            DeviceType::Switch => vec![ManagementProtocol::Snmp, ManagementProtocol::RestApi],
            DeviceType::AccessPoint => vec![ManagementProtocol::RestApi],
            DeviceType::Generic { .. } => vec![ManagementProtocol::Snmp],
        };

        // Most network devices don't have Nix capability
        SystemDescription::new(
            os,
            SystemArchitecture::x86_64_linux(),
            NixCapability::None,
            management,
        )
        .unwrap_or_else(|_| SystemDescription::unknown())
    }
}

/// Convert DeviceType to ComputeType
pub fn device_type_to_compute_type(device_type: &DeviceType) -> ComputeType {
    match device_type {
        // Network devices are physical appliances
        DeviceType::Gateway => ComputeType::Physical,
        DeviceType::Switch => ComputeType::Physical,
        DeviceType::AccessPoint => ComputeType::Physical,
        DeviceType::Generic { .. } => ComputeType::Physical,
    }
}

/// Convert ComputeType to closest DeviceType
pub fn compute_type_to_device_type(compute_type: &ComputeType, model: Option<&str>) -> DeviceType {
    match compute_type {
        ComputeType::Physical => {
            if let Some(m) = model {
                DeviceType::Generic { model: m.to_string() }
            } else {
                DeviceType::Generic { model: "Unknown".to_string() }
            }
        }
        ComputeType::VirtualMachine => DeviceType::Generic { model: "Virtual".to_string() },
        ComputeType::Container => DeviceType::Generic { model: "Container".to_string() },
    }
}

/// Create a NetworkDeviceAggregate from infrastructure ComputeResource
/// Only applicable for devices with network device metadata
pub fn compute_resource_to_network_device(
    resource: &ComputeResource,
) -> Option<NetworkDeviceAggregate> {
    // Check if this resource has network device metadata
    let mac_str = resource.capabilities.metadata.get("mac_address")?;
    let mac = MacAddress::parse(mac_str).ok()?;

    let device_type_str = resource.capabilities.metadata.get("device_type")
        .map(|s| s.as_str())
        .unwrap_or("Generic(Unknown)");

    let device_type = parse_device_type(device_type_str);

    let ip_address = resource.capabilities.metadata
        .get("ip_address")
        .and_then(|s| s.parse().ok());

    Some(NetworkDeviceAggregate::new_discovered(mac, device_type, ip_address))
}

/// Parse device type from string representation
fn parse_device_type(s: &str) -> DeviceType {
    match s {
        "Gateway" => DeviceType::Gateway,
        "Switch" => DeviceType::Switch,
        "AccessPoint" => DeviceType::AccessPoint,
        other => {
            if other.starts_with("Generic(") && other.ends_with(')') {
                let model = &other[8..other.len()-1];
                DeviceType::Generic { model: model.to_string() }
            } else {
                DeviceType::Generic { model: other.to_string() }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::MacAddress;

    fn create_test_device() -> NetworkDeviceAggregate {
        let mac = MacAddress::parse("00:11:22:33:44:55").unwrap();
        NetworkDeviceAggregate::new_discovered(
            mac,
            DeviceType::Switch,
            Some("192.168.1.100".parse().unwrap()),
        )
    }

    #[test]
    fn test_to_compute_resource_spec() {
        let device = create_test_device();
        let spec = device.to_compute_resource_spec();

        assert!(spec.is_ok());
        let spec = spec.unwrap();

        assert!(spec.id.as_str().starts_with("net-"));
        assert_eq!(spec.resource_type, ComputeType::Physical);
        assert!(spec.system_description.is_some());
        assert!(spec.capabilities.metadata.contains_key("mac_address"));
        assert!(spec.capabilities.metadata.contains_key("device_type"));
    }

    #[test]
    fn test_to_compute_resource() {
        let device = create_test_device();
        let resource = device.to_compute_resource();

        assert!(resource.is_ok());
        let resource = resource.unwrap();

        assert_eq!(resource.resource_type, ComputeType::Physical);
        assert!(resource.capabilities.metadata.get("mac_address")
            .map(|s| s == "00:11:22:33:44:55")
            .unwrap_or(false));
    }

    #[test]
    fn test_system_description() {
        let device = create_test_device();
        let desc = device.to_system_description();

        assert_eq!(desc.nix_capability(), NixCapability::None);
        assert!(desc.management_protocols().contains(&ManagementProtocol::Snmp));
    }

    #[test]
    fn test_device_type_mapping() {
        assert_eq!(
            device_type_to_compute_type(&DeviceType::Gateway),
            ComputeType::Physical
        );
        assert_eq!(
            device_type_to_compute_type(&DeviceType::Switch),
            ComputeType::Physical
        );
    }

    #[test]
    fn test_roundtrip_conversion() {
        let device = create_test_device();
        let resource = device.to_compute_resource().unwrap();

        // Convert back
        let recovered = compute_resource_to_network_device(&resource);
        assert!(recovered.is_some());

        let recovered = recovered.unwrap();
        assert_eq!(recovered.mac(), device.mac());
    }

    #[test]
    fn test_parse_device_type() {
        assert_eq!(parse_device_type("Gateway"), DeviceType::Gateway);
        assert_eq!(parse_device_type("Switch"), DeviceType::Switch);
        assert_eq!(parse_device_type("AccessPoint"), DeviceType::AccessPoint);
        assert_eq!(
            parse_device_type("Generic(Custom)"),
            DeviceType::Generic { model: "Custom".to_string() }
        );
    }
}
