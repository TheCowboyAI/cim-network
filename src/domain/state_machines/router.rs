//! Router state machine with phantom types for compile-time safety

use crate::domain::{RouterId, CorrelationId, CausationId, EventId};
use crate::domain::events::{RouterVendor, EventMetadata, NetworkEvent, MaintenanceWindow, DeploymentMethod, RouterConfigSnapshot};
use chrono::{DateTime, Utc};

// State marker types
#[derive(Debug, Clone)]
pub struct Planned;

#[derive(Debug, Clone)]
pub struct Provisioning;

#[derive(Debug, Clone)]
pub struct Configuring;

#[derive(Debug, Clone)]
pub struct Active;

#[derive(Debug, Clone)]
pub struct Failed {
    pub reason: String,
}

#[derive(Debug, Clone)]
pub struct Maintenance {
    pub window: MaintenanceWindow,
}

/// Router state machine with phantom type for state
pub struct RouterStateMachine<S> {
    id: RouterId,
    name: String,
    vendor: RouterVendor,
    created_at: DateTime<Utc>,
    state_data: S,
}

// Implementations for RouterStateMachine in different states

impl RouterStateMachine<Planned> {
    /// Create a new router in planned state
    pub fn new(id: RouterId, name: String, vendor: RouterVendor) -> Self {
        Self {
            id,
            name,
            vendor,
            created_at: Utc::now(),
            state_data: Planned,
        }
    }
    
    /// Start provisioning the router
    pub fn start_provisioning(
        self, 
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<(RouterStateMachine<Provisioning>, NetworkEvent), String> {
        let event = NetworkEvent::RouterProvisioningStarted {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: 1,
            },
            router_id: self.id,
            vendor: self.vendor.clone(),
        };
        
        let new_state = RouterStateMachine {
            id: self.id,
            name: self.name,
            vendor: self.vendor,
            created_at: self.created_at,
            state_data: Provisioning,
        };
        
        Ok((new_state, event))
    }
}

impl RouterStateMachine<Provisioning> {
    /// Create from parts (for testing)
    pub fn from_parts(id: RouterId, name: String, vendor: RouterVendor) -> Self {
        Self {
            id,
            name,
            vendor,
            created_at: Utc::now(),
            state_data: Provisioning,
        }
    }
    
    /// Mark provisioning as complete
    pub fn provisioning_complete(
        self,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<(RouterStateMachine<Configuring>, NetworkEvent), String> {
        let event = NetworkEvent::RouterProvisioningCompleted {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: 2,
            },
            router_id: self.id,
        };
        
        let new_state = RouterStateMachine {
            id: self.id,
            name: self.name,
            vendor: self.vendor,
            created_at: self.created_at,
            state_data: Configuring,
        };
        
        Ok((new_state, event))
    }
}

impl RouterStateMachine<Configuring> {
    /// Create from parts (for testing)
    pub fn from_parts(id: RouterId, name: String, vendor: RouterVendor) -> Self {
        Self {
            id,
            name,
            vendor,
            created_at: Utc::now(),
            state_data: Configuring,
        }
    }
    
    /// Apply configuration successfully
    pub fn configuration_applied(
        self,
        config: RouterConfigSnapshot,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<(RouterStateMachine<Active>, NetworkEvent), String> {
        let event = NetworkEvent::RouterConfigurationApplied {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: 3,
            },
            router_id: self.id,
            configuration: config,
            deployment_method: DeploymentMethod::Nix { flake_ref: "nixos#network".to_string() },
        };
        
        let new_state = RouterStateMachine {
            id: self.id,
            name: self.name,
            vendor: self.vendor,
            created_at: self.created_at,
            state_data: Active,
        };
        
        Ok((new_state, event))
    }
    
    /// Configuration failed
    pub fn configuration_failed(
        self,
        reason: String,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<(RouterStateMachine<Failed>, NetworkEvent), String> {
        let event = NetworkEvent::RouterConfigurationFailed {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: 3,
            },
            router_id: self.id,
            reason: reason.clone(),
        };
        
        let new_state = RouterStateMachine {
            id: self.id,
            name: self.name,
            vendor: self.vendor,
            created_at: self.created_at,
            state_data: Failed { reason },
        };
        
        Ok((new_state, event))
    }
}

impl RouterStateMachine<Active> {
    /// Create from parts (for testing)
    pub fn from_parts(id: RouterId, name: String, vendor: RouterVendor) -> Self {
        Self {
            id,
            name,
            vendor,
            created_at: Utc::now(),
            state_data: Active,
        }
    }
    
    /// Schedule maintenance
    pub fn schedule_maintenance(
        self,
        window: MaintenanceWindow,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<(RouterStateMachine<Maintenance>, NetworkEvent), String> {
        let event = NetworkEvent::RouterMaintenanceScheduled {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: 4,
            },
            router_id: self.id,
            window: window.clone(),
        };
        
        let new_state = RouterStateMachine {
            id: self.id,
            name: self.name,
            vendor: self.vendor,
            created_at: self.created_at,
            state_data: Maintenance { window },
        };
        
        Ok((new_state, event))
    }
}

impl RouterStateMachine<Failed> {
    /// Create from parts with failure reason (for testing)
    pub fn from_parts_with_failure(
        id: RouterId, 
        name: String, 
        vendor: RouterVendor,
        failure_reason: String,
    ) -> Self {
        RouterStateMachine {
            id,
            name,
            vendor,
            created_at: Utc::now(),
            state_data: Failed { reason: failure_reason },
        }
    }
    
    /// Retry configuration
    pub fn retry_configuration(
        self,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<(RouterStateMachine<Configuring>, NetworkEvent), String> {
        let event = NetworkEvent::RouterConfigurationRetryStarted {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: 4,
            },
            router_id: self.id,
            previous_failure: self.state_data.reason.clone(),
        };
        
        let new_state = RouterStateMachine {
            id: self.id,
            name: self.name,
            vendor: self.vendor,
            created_at: self.created_at,
            state_data: Configuring,
        };
        
        Ok((new_state, event))
    }
    
    /// Get the failure reason
    pub fn failure_reason(&self) -> &str {
        &self.state_data.reason
    }
}

impl RouterStateMachine<Maintenance> {
    /// Get the maintenance window
    pub fn maintenance_window(&self) -> &MaintenanceWindow {
        &self.state_data.window
    }
}

// Common methods for all states
impl<S> RouterStateMachine<S> {
    /// Get the router name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get the router ID
    pub fn id(&self) -> RouterId {
        self.id
    }
    
    /// Get the vendor
    pub fn vendor(&self) -> &RouterVendor {
        &self.vendor
    }
}