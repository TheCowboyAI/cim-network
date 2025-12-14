//! # Network Domain Aggregates
//!
//! Aggregates using cim-domain state machine patterns.
//! Each aggregate is a consistency boundary with a Moore state machine
//! controlling its lifecycle.

use crate::domain::value_objects::*;
use crate::domain::events::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// Device State Machine (Moore Machine)
// ============================================================================

/// Device lifecycle states
///
/// Moore machine: output depends only on current state
/// ```text
///                    ┌─────────────┐
///                    │  Discovered │
///                    └──────┬──────┘
///                           │ adopt
///                           ▼
///                    ┌─────────────┐
///           ┌───────▶│  Adopting   │
///           │        └──────┬──────┘
///           │               │ provisioned
///           │               ▼
///           │        ┌─────────────┐
///           │        │  Provisioned│◀──────┐
///           │        └──────┬──────┘       │
///           │               │ configure    │ configured
///           │               ▼              │
///           │        ┌─────────────┐       │
///           │        │ Configuring │───────┘
///           │        └──────┬──────┘
///           │               │ error
///           │               ▼
///           │        ┌─────────────┐
///           └────────│   Error     │
///         retry      └──────┬──────┘
///                           │ decommission
///                           ▼
///                    ┌─────────────┐
///                    │Decommissioned│ (terminal)
///                    └─────────────┘
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceState {
    /// Device discovered but not yet adopted
    Discovered,
    /// Device is being adopted
    Adopting,
    /// Device is provisioned and operational
    Provisioned,
    /// Device is being configured
    Configuring,
    /// Device encountered an error
    Error,
    /// Device has been decommissioned (terminal state)
    Decommissioned,
}

impl DeviceState {
    /// Get valid transitions from this state
    pub fn valid_transitions(&self) -> &[DeviceState] {
        match self {
            DeviceState::Discovered => &[DeviceState::Adopting, DeviceState::Decommissioned],
            DeviceState::Adopting => &[DeviceState::Provisioned, DeviceState::Error],
            DeviceState::Provisioned => &[DeviceState::Configuring, DeviceState::Decommissioned],
            DeviceState::Configuring => &[DeviceState::Provisioned, DeviceState::Error],
            DeviceState::Error => &[DeviceState::Adopting, DeviceState::Decommissioned],
            DeviceState::Decommissioned => &[], // Terminal state
        }
    }

    /// Check if transition to target state is valid
    pub fn can_transition_to(&self, target: DeviceState) -> bool {
        self.valid_transitions().contains(&target)
    }

    /// Is this a terminal state?
    pub fn is_terminal(&self) -> bool {
        matches!(self, DeviceState::Decommissioned)
    }

    /// Get state name for logging/display
    pub fn name(&self) -> &'static str {
        match self {
            DeviceState::Discovered => "Discovered",
            DeviceState::Adopting => "Adopting",
            DeviceState::Provisioned => "Provisioned",
            DeviceState::Configuring => "Configuring",
            DeviceState::Error => "Error",
            DeviceState::Decommissioned => "Decommissioned",
        }
    }
}

impl Default for DeviceState {
    fn default() -> Self {
        DeviceState::Discovered
    }
}

// ============================================================================
// Network Device Aggregate
// ============================================================================

/// Network device aggregate - consistency boundary for a single device
///
/// This aggregate:
/// - Has a Moore state machine for lifecycle management
/// - Emits domain events for all state changes
/// - Is reconstructed from events (event sourcing)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDeviceAggregate {
    /// Device identifier
    id: DeviceId,
    /// Current state
    state: DeviceState,
    /// Version for optimistic concurrency
    version: u64,
    /// MAC address (immutable after creation)
    mac: MacAddress,
    /// Device type
    device_type: DeviceType,
    /// Device name
    name: String,
    /// Model identifier
    model: Option<String>,
    /// Firmware version
    firmware_version: Option<String>,
    /// IP address
    ip_address: Option<std::net::IpAddr>,
    /// Vendor-specific ID (for adapter mapping)
    vendor_id: Option<String>,
    /// Interface configurations
    interfaces: Vec<InterfaceConfig>,
    /// VLAN configurations
    vlans: Vec<VlanConfig>,
    /// Pending events (not yet persisted)
    #[serde(skip)]
    pending_events: Vec<NetworkEvent>,
    /// Error message (if in Error state)
    error_message: Option<String>,
}

impl NetworkDeviceAggregate {
    /// Create a new device aggregate from discovery
    pub fn new_discovered(
        mac: MacAddress,
        device_type: DeviceType,
        ip_address: Option<std::net::IpAddr>,
    ) -> Self {
        let id = DeviceId::new();
        let mut device = Self {
            id,
            state: DeviceState::Discovered,
            version: 0,
            mac,
            device_type: device_type.clone(),
            name: format!("Device-{}", &id.to_string()[..8]),
            model: None,
            firmware_version: None,
            ip_address,
            vendor_id: None,
            interfaces: Vec::new(),
            vlans: Vec::new(),
            pending_events: Vec::new(),
            error_message: None,
        };

        device.apply_event(NetworkEvent::DeviceDiscovered {
            device_id: id,
            mac,
            device_type,
            ip_address,
        });

        device
    }

    /// Create from a discovered event (for replay)
    pub fn from_discovered_event(
        device_id: DeviceId,
        mac: MacAddress,
        device_type: DeviceType,
        ip_address: Option<std::net::IpAddr>,
    ) -> Self {
        Self {
            id: device_id,
            state: DeviceState::Discovered,
            version: 1,
            mac,
            device_type,
            name: format!("Device-{}", &device_id.to_string()[..8]),
            model: None,
            firmware_version: None,
            ip_address,
            vendor_id: None,
            interfaces: Vec::new(),
            vlans: Vec::new(),
            pending_events: Vec::new(),
            error_message: None,
        }
    }

    /// Reconstruct from events
    pub fn from_events(events: impl IntoIterator<Item = NetworkEvent>) -> Option<Self> {
        let mut device: Option<Self> = None;

        for event in events {
            match &event {
                NetworkEvent::DeviceDiscovered {
                    device_id,
                    mac,
                    device_type,
                    ip_address,
                } => {
                    device = Some(Self {
                        id: *device_id,
                        state: DeviceState::Discovered,
                        version: 1,
                        mac: *mac,
                        device_type: device_type.clone(),
                        name: format!("Device-{}", &device_id.to_string()[..8]),
                        model: None,
                        firmware_version: None,
                        ip_address: *ip_address,
                        vendor_id: None,
                        interfaces: Vec::new(),
                        vlans: Vec::new(),
                        pending_events: Vec::new(),
                        error_message: None,
                    });
                }
                _ => {
                    if let Some(ref mut d) = device {
                        d.apply_existing_event(&event);
                    }
                }
            }
        }

        device
    }

    // Getters
    pub fn id(&self) -> DeviceId {
        self.id
    }

    pub fn state(&self) -> DeviceState {
        self.state
    }

    pub fn version(&self) -> u64 {
        self.version
    }

    pub fn mac(&self) -> MacAddress {
        self.mac
    }

    pub fn device_type(&self) -> &DeviceType {
        &self.device_type
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn ip_address(&self) -> Option<std::net::IpAddr> {
        self.ip_address
    }

    pub fn vendor_id(&self) -> Option<&str> {
        self.vendor_id.as_deref()
    }

    pub fn interfaces(&self) -> &[InterfaceConfig] {
        &self.interfaces
    }

    pub fn take_pending_events(&mut self) -> Vec<NetworkEvent> {
        std::mem::take(&mut self.pending_events)
    }

    // Commands (state transitions)

    /// Adopt the device
    pub fn adopt(&mut self, vendor_id: String) -> Result<(), AggregateError> {
        self.transition_to(DeviceState::Adopting)?;
        self.vendor_id = Some(vendor_id.clone());
        self.apply_event(NetworkEvent::DeviceAdopting {
            device_id: self.id,
            vendor_id,
        });
        Ok(())
    }

    /// Mark device as provisioned
    pub fn mark_provisioned(&mut self, model: String, firmware: String) -> Result<(), AggregateError> {
        self.transition_to(DeviceState::Provisioned)?;
        self.model = Some(model.clone());
        self.firmware_version = Some(firmware.clone());
        self.apply_event(NetworkEvent::DeviceProvisioned {
            device_id: self.id,
            model,
            firmware_version: firmware,
        });
        Ok(())
    }

    /// Start configuration
    pub fn start_configuration(&mut self) -> Result<(), AggregateError> {
        self.transition_to(DeviceState::Configuring)?;
        self.apply_event(NetworkEvent::DeviceConfiguring {
            device_id: self.id,
        });
        Ok(())
    }

    /// Complete configuration
    pub fn complete_configuration(
        &mut self,
        interfaces: Vec<InterfaceConfig>,
        vlans: Vec<VlanConfig>,
    ) -> Result<(), AggregateError> {
        self.transition_to(DeviceState::Provisioned)?;
        self.interfaces = interfaces.clone();
        self.vlans = vlans.clone();
        self.apply_event(NetworkEvent::DeviceConfigured {
            device_id: self.id,
            interfaces,
            vlans,
        });
        Ok(())
    }

    /// Record an error
    pub fn record_error(&mut self, message: String) -> Result<(), AggregateError> {
        self.transition_to(DeviceState::Error)?;
        self.error_message = Some(message.clone());
        self.apply_event(NetworkEvent::DeviceError {
            device_id: self.id,
            message,
        });
        Ok(())
    }

    /// Decommission the device
    pub fn decommission(&mut self) -> Result<(), AggregateError> {
        self.transition_to(DeviceState::Decommissioned)?;
        self.apply_event(NetworkEvent::DeviceDecommissioned {
            device_id: self.id,
        });
        Ok(())
    }

    /// Update device name
    pub fn rename(&mut self, name: String) -> Result<(), AggregateError> {
        if self.state == DeviceState::Decommissioned {
            return Err(AggregateError::InvalidState {
                current: self.state,
                operation: "rename".to_string(),
            });
        }
        let old_name = std::mem::replace(&mut self.name, name.clone());
        self.apply_event(NetworkEvent::DeviceRenamed {
            device_id: self.id,
            old_name,
            new_name: name,
        });
        Ok(())
    }

    // Private helpers

    fn transition_to(&mut self, target: DeviceState) -> Result<(), AggregateError> {
        if !self.state.can_transition_to(target) {
            return Err(AggregateError::InvalidTransition {
                from: self.state,
                to: target,
            });
        }
        self.state = target;
        Ok(())
    }

    fn apply_event(&mut self, event: NetworkEvent) {
        self.version += 1;
        self.pending_events.push(event);
    }

    fn apply_existing_event(&mut self, event: &NetworkEvent) {
        match event {
            NetworkEvent::DeviceAdopting { vendor_id, .. } => {
                self.state = DeviceState::Adopting;
                self.vendor_id = Some(vendor_id.clone());
            }
            NetworkEvent::DeviceProvisioned {
                model,
                firmware_version,
                ..
            } => {
                self.state = DeviceState::Provisioned;
                self.model = Some(model.clone());
                self.firmware_version = Some(firmware_version.clone());
            }
            NetworkEvent::DeviceConfiguring { .. } => {
                self.state = DeviceState::Configuring;
            }
            NetworkEvent::DeviceConfigured {
                interfaces, vlans, ..
            } => {
                self.state = DeviceState::Provisioned;
                self.interfaces = interfaces.clone();
                self.vlans = vlans.clone();
            }
            NetworkEvent::DeviceError { message, .. } => {
                self.state = DeviceState::Error;
                self.error_message = Some(message.clone());
            }
            NetworkEvent::DeviceDecommissioned { .. } => {
                self.state = DeviceState::Decommissioned;
            }
            NetworkEvent::DeviceRenamed { new_name, .. } => {
                self.name = new_name.clone();
            }
            _ => {}
        }
        self.version += 1;
    }
}

// ============================================================================
// Aggregate Errors
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum AggregateError {
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition { from: DeviceState, to: DeviceState },

    #[error("Invalid operation '{operation}' in state {current:?}")]
    InvalidState {
        current: DeviceState,
        operation: String,
    },

    #[error("Concurrency conflict: expected version {expected}, found {actual}")]
    ConcurrencyConflict { expected: u64, actual: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_mac() -> MacAddress {
        MacAddress::parse("00:11:22:33:44:55").unwrap()
    }

    // ==========================================================================
    // DeviceState Tests
    // ==========================================================================

    #[test]
    fn test_device_state_default() {
        assert_eq!(DeviceState::default(), DeviceState::Discovered);
    }

    #[test]
    fn test_device_state_transitions_from_discovered() {
        let state = DeviceState::Discovered;
        assert!(state.can_transition_to(DeviceState::Adopting));
        assert!(state.can_transition_to(DeviceState::Decommissioned));
        assert!(!state.can_transition_to(DeviceState::Provisioned));
        assert!(!state.can_transition_to(DeviceState::Configuring));
        assert!(!state.can_transition_to(DeviceState::Error));
    }

    #[test]
    fn test_device_state_transitions_from_adopting() {
        let state = DeviceState::Adopting;
        assert!(state.can_transition_to(DeviceState::Provisioned));
        assert!(state.can_transition_to(DeviceState::Error));
        assert!(!state.can_transition_to(DeviceState::Discovered));
        assert!(!state.can_transition_to(DeviceState::Decommissioned));
    }

    #[test]
    fn test_device_state_transitions_from_provisioned() {
        let state = DeviceState::Provisioned;
        assert!(state.can_transition_to(DeviceState::Configuring));
        assert!(state.can_transition_to(DeviceState::Decommissioned));
        assert!(!state.can_transition_to(DeviceState::Adopting));
        assert!(!state.can_transition_to(DeviceState::Discovered));
    }

    #[test]
    fn test_device_state_transitions_from_configuring() {
        let state = DeviceState::Configuring;
        assert!(state.can_transition_to(DeviceState::Provisioned));
        assert!(state.can_transition_to(DeviceState::Error));
        assert!(!state.can_transition_to(DeviceState::Adopting));
    }

    #[test]
    fn test_device_state_transitions_from_error() {
        let state = DeviceState::Error;
        assert!(state.can_transition_to(DeviceState::Adopting)); // retry
        assert!(state.can_transition_to(DeviceState::Decommissioned));
        assert!(!state.can_transition_to(DeviceState::Provisioned));
    }

    #[test]
    fn test_device_state_terminal() {
        assert!(DeviceState::Decommissioned.is_terminal());
        assert!(!DeviceState::Discovered.is_terminal());
        assert!(!DeviceState::Provisioned.is_terminal());
        assert!(DeviceState::Decommissioned.valid_transitions().is_empty());
    }

    #[test]
    fn test_device_state_names() {
        assert_eq!(DeviceState::Discovered.name(), "Discovered");
        assert_eq!(DeviceState::Adopting.name(), "Adopting");
        assert_eq!(DeviceState::Provisioned.name(), "Provisioned");
        assert_eq!(DeviceState::Configuring.name(), "Configuring");
        assert_eq!(DeviceState::Error.name(), "Error");
        assert_eq!(DeviceState::Decommissioned.name(), "Decommissioned");
    }

    // ==========================================================================
    // NetworkDeviceAggregate Tests
    // ==========================================================================

    #[test]
    fn test_aggregate_new_discovered() {
        let mac = create_test_mac();
        let device = NetworkDeviceAggregate::new_discovered(
            mac,
            DeviceType::Switch,
            Some("192.168.1.1".parse().unwrap()),
        );

        assert_eq!(device.state(), DeviceState::Discovered);
        assert_eq!(device.mac(), mac);
        assert!(device.name().starts_with("Device-"));
        assert_eq!(device.version(), 1);
    }

    #[test]
    fn test_aggregate_adopt() {
        let mac = create_test_mac();
        let mut device = NetworkDeviceAggregate::new_discovered(
            mac,
            DeviceType::Switch,
            None,
        );
        device.take_pending_events(); // Clear discovery event

        let result = device.adopt("vendor-123".to_string());
        assert!(result.is_ok());
        assert_eq!(device.state(), DeviceState::Adopting);
        assert_eq!(device.vendor_id(), Some("vendor-123"));

        let events = device.take_pending_events();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], NetworkEvent::DeviceAdopting { .. }));
    }

    #[test]
    fn test_aggregate_cannot_adopt_from_provisioned() {
        let mac = create_test_mac();
        let mut device = NetworkDeviceAggregate::new_discovered(
            mac,
            DeviceType::Switch,
            None,
        );
        device.adopt("vendor-123".to_string()).unwrap();
        device.mark_provisioned("Model-X".to_string(), "1.0.0".to_string()).unwrap();
        device.take_pending_events();

        // Should fail - can't adopt from provisioned state
        let result = device.adopt("vendor-456".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AggregateError::InvalidTransition { .. }
        ));
    }

    #[test]
    fn test_aggregate_full_lifecycle() {
        let mac = create_test_mac();
        let mut device = NetworkDeviceAggregate::new_discovered(
            mac,
            DeviceType::AccessPoint,
            Some("10.0.0.50".parse().unwrap()),
        );

        // Discovery
        assert_eq!(device.state(), DeviceState::Discovered);

        // Adopt
        device.adopt("ap-001".to_string()).unwrap();
        assert_eq!(device.state(), DeviceState::Adopting);

        // Provision
        device.mark_provisioned("UAP-AC-Pro".to_string(), "6.2.0".to_string()).unwrap();
        assert_eq!(device.state(), DeviceState::Provisioned);

        // Configure
        device.start_configuration().unwrap();
        assert_eq!(device.state(), DeviceState::Configuring);

        // Complete config
        device.complete_configuration(vec![], vec![]).unwrap();
        assert_eq!(device.state(), DeviceState::Provisioned);

        // Decommission
        device.decommission().unwrap();
        assert_eq!(device.state(), DeviceState::Decommissioned);
        assert!(device.state().is_terminal());
    }

    #[test]
    fn test_aggregate_error_recovery() {
        let mac = create_test_mac();
        let mut device = NetworkDeviceAggregate::new_discovered(
            mac,
            DeviceType::Gateway,
            None,
        );

        device.adopt("gw-001".to_string()).unwrap();

        // Simulate error during adoption
        device.record_error("Connection timeout".to_string()).unwrap();
        assert_eq!(device.state(), DeviceState::Error);

        // Retry adoption
        device.adopt("gw-001".to_string()).unwrap();
        assert_eq!(device.state(), DeviceState::Adopting);
    }

    #[test]
    fn test_aggregate_rename() {
        let mac = create_test_mac();
        let mut device = NetworkDeviceAggregate::new_discovered(
            mac,
            DeviceType::Switch,
            None,
        );
        let original_name = device.name().to_string();
        device.take_pending_events();

        device.rename("Core-Switch-1".to_string()).unwrap();
        assert_eq!(device.name(), "Core-Switch-1");

        let events = device.take_pending_events();
        assert_eq!(events.len(), 1);
        match &events[0] {
            NetworkEvent::DeviceRenamed { old_name, new_name, .. } => {
                assert_eq!(old_name, &original_name);
                assert_eq!(new_name, "Core-Switch-1");
            }
            _ => panic!("Expected DeviceRenamed event"),
        }
    }

    #[test]
    fn test_aggregate_cannot_rename_decommissioned() {
        let mac = create_test_mac();
        let mut device = NetworkDeviceAggregate::new_discovered(
            mac,
            DeviceType::Switch,
            None,
        );
        device.decommission().unwrap();

        let result = device.rename("New-Name".to_string());
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AggregateError::InvalidState { operation, .. } if operation == "rename"
        ));
    }

    #[test]
    fn test_aggregate_from_events() {
        let device_id = DeviceId::new();
        let mac = create_test_mac();

        let events = vec![
            NetworkEvent::DeviceDiscovered {
                device_id,
                mac,
                device_type: DeviceType::Switch,
                ip_address: Some("192.168.1.10".parse().unwrap()),
            },
            NetworkEvent::DeviceAdopting {
                device_id,
                vendor_id: "switch-001".to_string(),
            },
            NetworkEvent::DeviceProvisioned {
                device_id,
                model: "USW-24".to_string(),
                firmware_version: "6.6.0".to_string(),
            },
            NetworkEvent::DeviceRenamed {
                device_id,
                old_name: "Device-123".to_string(),
                new_name: "Main-Switch".to_string(),
            },
        ];

        let device = NetworkDeviceAggregate::from_events(events);
        assert!(device.is_some());

        let device = device.unwrap();
        assert_eq!(device.id(), device_id);
        assert_eq!(device.mac(), mac);
        assert_eq!(device.state(), DeviceState::Provisioned);
        assert_eq!(device.name(), "Main-Switch");
        assert_eq!(device.version(), 4);
    }

    #[test]
    fn test_aggregate_from_events_empty() {
        let events: Vec<NetworkEvent> = vec![];
        let device = NetworkDeviceAggregate::from_events(events);
        assert!(device.is_none());
    }

    #[test]
    fn test_aggregate_version_increments() {
        let mac = create_test_mac();
        let mut device = NetworkDeviceAggregate::new_discovered(
            mac,
            DeviceType::Switch,
            None,
        );
        assert_eq!(device.version(), 1);

        device.adopt("v-1".to_string()).unwrap();
        assert_eq!(device.version(), 2);

        device.mark_provisioned("Model".to_string(), "1.0".to_string()).unwrap();
        assert_eq!(device.version(), 3);

        device.rename("New".to_string()).unwrap();
        assert_eq!(device.version(), 4);
    }

    #[test]
    fn test_aggregate_pending_events() {
        let mac = create_test_mac();
        let mut device = NetworkDeviceAggregate::new_discovered(
            mac,
            DeviceType::Switch,
            None,
        );

        // Should have one event from creation
        let events = device.take_pending_events();
        assert_eq!(events.len(), 1);

        // After taking, should be empty
        let events = device.take_pending_events();
        assert!(events.is_empty());

        // New action creates new event
        device.adopt("v-1".to_string()).unwrap();
        let events = device.take_pending_events();
        assert_eq!(events.len(), 1);
    }
}
