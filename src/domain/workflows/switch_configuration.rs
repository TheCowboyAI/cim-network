//! Switch configuration workflow definition

use cim_domain_workflow::{
    Workflow, WorkflowStep, WorkflowDefinition, WorkflowError, StepResult,
    WorkflowContext, WorkflowEngine
};
use crate::domain::{SwitchId, VlanId, PortId, CorrelationId, CausationId};
use crate::domain::events::{NetworkEvent, EventMetadata, EventId};
use crate::domain::value_objects::VlanConfig;
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use std::collections::HashMap;

/// Switch configuration workflow for VLAN and port management
pub struct SwitchConfigurationWorkflow {
    pub switch_id: SwitchId,
    pub switch_name: String,
    pub vlan_configs: Vec<VlanConfigRequest>,
    pub port_assignments: Vec<PortAssignmentRequest>,
}

#[derive(Debug, Clone)]
pub struct VlanConfigRequest {
    pub vlan_id: VlanId,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PortAssignmentRequest {
    pub port_id: PortId,
    pub vlan_id: VlanId,
    pub access_mode: bool,
}

impl SwitchConfigurationWorkflow {
    pub fn new(
        switch_id: SwitchId,
        switch_name: String,
        vlan_configs: Vec<VlanConfigRequest>,
        port_assignments: Vec<PortAssignmentRequest>,
    ) -> Self {
        Self {
            switch_id,
            switch_name,
            vlan_configs,
            port_assignments,
        }
    }
}

#[async_trait]
impl WorkflowDefinition for SwitchConfigurationWorkflow {
    type Context = SwitchConfigContext;
    type Error = SwitchConfigError;

    fn workflow_id(&self) -> String {
        format!("switch_config_{}", self.switch_id)
    }

    fn workflow_name(&self) -> String {
        "Switch Configuration".to_string()
    }

    async fn define_steps(&self) -> Result<Vec<Box<dyn WorkflowStep<Context = Self::Context, Error = Self::Error>>>, Self::Error> {
        Ok(vec![
            Box::new(ValidateSwitchConfigStep::new(self.switch_id)),
            Box::new(CreateVlansStep::new(self.vlan_configs.clone())),
            Box::new(ConfigurePortsStep::new(self.port_assignments.clone())),
            Box::new(ConfigureSpanningTreeStep::new(self.switch_id)),
            Box::new(VerifyConfigurationStep::new(self.switch_id)),
        ])
    }

    async fn create_context(&self) -> Result<Self::Context, Self::Error> {
        Ok(SwitchConfigContext {
            switch_id: self.switch_id,
            switch_name: self.switch_name.clone(),
            created_vlans: HashMap::new(),
            configured_ports: HashMap::new(),
            spanning_tree_configured: false,
            events: Vec::new(),
            validation_errors: Vec::new(),
        })
    }
}

/// Context for switch configuration workflow
#[derive(Debug, Clone)]
pub struct SwitchConfigContext {
    pub switch_id: SwitchId,
    pub switch_name: String,
    pub created_vlans: HashMap<VlanId, VlanConfig>,
    pub configured_ports: HashMap<PortId, VlanId>,
    pub spanning_tree_configured: bool,
    pub events: Vec<NetworkEvent>,
    pub validation_errors: Vec<String>,
}

/// Errors that can occur during switch configuration
#[derive(Debug, thiserror::Error)]
pub enum SwitchConfigError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("VLAN configuration error: {0}")]
    VlanConfig(String),
    #[error("Port configuration error: {0}")]
    PortConfig(String),
    #[error("Spanning tree error: {0}")]
    SpanningTree(String),
    #[error("Workflow engine error: {0}")]
    WorkflowEngine(#[from] WorkflowError),
}

// Step 1: Validate Switch Configuration
pub struct ValidateSwitchConfigStep {
    switch_id: SwitchId,
}

impl ValidateSwitchConfigStep {
    pub fn new(switch_id: SwitchId) -> Self {
        Self { switch_id }
    }
}

#[async_trait]
impl WorkflowStep for ValidateSwitchConfigStep {
    type Context = SwitchConfigContext;
    type Error = SwitchConfigError;

    fn step_name(&self) -> String {
        "Validate Switch Configuration".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Validating switch configuration for {}", self.switch_id);
        
        // Basic validation checks
        let mut errors = Vec::new();
        
        // Check if switch exists (simulated)
        if context.switch_name.is_empty() {
            errors.push("Switch name cannot be empty".to_string());
        }
        
        // Check for VLAN ID conflicts (simulated)
        let mut vlan_ids = std::collections::HashSet::new();
        for vlan_request in &context.created_vlans {
            if !vlan_ids.insert(vlan_request.0) {
                errors.push(format!("Duplicate VLAN ID: {}", vlan_request.0));
            }
        }
        
        if !errors.is_empty() {
            context.validation_errors = errors.clone();
            return Err(SwitchConfigError::Validation(
                format!("Validation failed: {}", errors.join(", "))
            ));
        }
        
        tracing::info!("Switch configuration validation passed");
        Ok(())
    }

    async fn compensate(&self, _context: &mut Self::Context) -> StepResult<Self::Error> {
        // No compensation needed for validation
        Ok(())
    }
}

// Step 2: Create VLANs
pub struct CreateVlansStep {
    vlan_requests: Vec<VlanConfigRequest>,
}

impl CreateVlansStep {
    pub fn new(vlan_requests: Vec<VlanConfigRequest>) -> Self {
        Self { vlan_requests }
    }
}

#[async_trait]
impl WorkflowStep for CreateVlansStep {
    type Context = SwitchConfigContext;
    type Error = SwitchConfigError;

    fn step_name(&self) -> String {
        "Create VLANs".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Creating {} VLANs on switch {}", 
                      self.vlan_requests.len(), context.switch_id);
        
        for vlan_request in &self.vlan_requests {
            // Create VLAN configuration
            let vlan_config = VlanConfig {
                vlan_id: vlan_request.vlan_id,
                name: vlan_request.name.clone(),
                description: vlan_request.description.clone(),
            };
            
            // Simulate VLAN creation
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            
            context.created_vlans.insert(vlan_request.vlan_id, vlan_config);
            
            // Generate VLAN created event
            let correlation_id = CorrelationId::new();
            let causation_id = CausationId::new();
            
            let event = NetworkEvent::VlanCreated {
                metadata: EventMetadata {
                    event_id: EventId::new(),
                    aggregate_id: context.switch_id.into(),
                    correlation_id,
                    causation_id,
                    timestamp: Utc::now(),
                    version: 1,
                },
                switch_id: context.switch_id,
                vlan_id: vlan_request.vlan_id,
                name: vlan_request.name.clone(),
                description: vlan_request.description.clone(),
            };
            
            context.events.push(event);
            tracing::info!("Created VLAN {} ({})", vlan_request.vlan_id, vlan_request.name);
        }
        
        Ok(())
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Compensating: Removing created VLANs");
        
        // Remove created VLANs
        for vlan_request in &self.vlan_requests {
            context.created_vlans.remove(&vlan_request.vlan_id);
            
            // Generate VLAN removed event
            let correlation_id = CorrelationId::new();
            let causation_id = CausationId::new();
            
            let event = NetworkEvent::VlanRemoved {
                metadata: EventMetadata {
                    event_id: EventId::new(),
                    aggregate_id: context.switch_id.into(),
                    correlation_id,
                    causation_id,
                    timestamp: Utc::now(),
                    version: 1,
                },
                switch_id: context.switch_id,
                vlan_id: vlan_request.vlan_id,
            };
            
            context.events.push(event);
        }
        
        Ok(())
    }
}

// Step 3: Configure Ports
pub struct ConfigurePortsStep {
    port_assignments: Vec<PortAssignmentRequest>,
}

impl ConfigurePortsStep {
    pub fn new(port_assignments: Vec<PortAssignmentRequest>) -> Self {
        Self { port_assignments }
    }
}

#[async_trait]
impl WorkflowStep for ConfigurePortsStep {
    type Context = SwitchConfigContext;
    type Error = SwitchConfigError;

    fn step_name(&self) -> String {
        "Configure Ports".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Configuring {} ports on switch {}", 
                      self.port_assignments.len(), context.switch_id);
        
        for port_assignment in &self.port_assignments {
            // Verify VLAN exists
            if !context.created_vlans.contains_key(&port_assignment.vlan_id) {
                return Err(SwitchConfigError::PortConfig(
                    format!("VLAN {} does not exist for port assignment", port_assignment.vlan_id)
                ));
            }
            
            // Simulate port configuration
            tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
            
            context.configured_ports.insert(port_assignment.port_id, port_assignment.vlan_id);
            
            // Generate port assigned event
            let correlation_id = CorrelationId::new();
            let causation_id = CausationId::new();
            
            let event = NetworkEvent::PortAssignedToVlan {
                metadata: EventMetadata {
                    event_id: EventId::new(),
                    aggregate_id: context.switch_id.into(),
                    correlation_id,
                    causation_id,
                    timestamp: Utc::now(),
                    version: 1,
                },
                switch_id: context.switch_id,
                port_id: port_assignment.port_id,
                vlan_id: port_assignment.vlan_id,
                access_mode: port_assignment.access_mode,
            };
            
            context.events.push(event);
            tracing::info!("Configured port {} for VLAN {}", 
                          port_assignment.port_id, port_assignment.vlan_id);
        }
        
        Ok(())
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Compensating: Removing port configurations");
        
        for port_assignment in &self.port_assignments {
            context.configured_ports.remove(&port_assignment.port_id);
            
            // Generate port unassigned event
            let correlation_id = CorrelationId::new();
            let causation_id = CausationId::new();
            
            let event = NetworkEvent::PortUnassignedFromVlan {
                metadata: EventMetadata {
                    event_id: EventId::new(),
                    aggregate_id: context.switch_id.into(),
                    correlation_id,
                    causation_id,
                    timestamp: Utc::now(),
                    version: 1,
                },
                switch_id: context.switch_id,
                port_id: port_assignment.port_id,
                previous_vlan_id: port_assignment.vlan_id,
            };
            
            context.events.push(event);
        }
        
        Ok(())
    }
}

// Step 4: Configure Spanning Tree
pub struct ConfigureSpanningTreeStep {
    switch_id: SwitchId,
}

impl ConfigureSpanningTreeStep {
    pub fn new(switch_id: SwitchId) -> Self {
        Self { switch_id }
    }
}

#[async_trait]
impl WorkflowStep for ConfigureSpanningTreeStep {
    type Context = SwitchConfigContext;
    type Error = SwitchConfigError;

    fn step_name(&self) -> String {
        "Configure Spanning Tree Protocol".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Configuring spanning tree protocol on switch {}", self.switch_id);
        
        // Simulate spanning tree configuration
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        
        context.spanning_tree_configured = true;
        
        // Generate spanning tree configured event
        let correlation_id = CorrelationId::new();
        let causation_id = CausationId::new();
        
        let event = NetworkEvent::SpanningTreeConfigured {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.switch_id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: 1,
            },
            switch_id: self.switch_id,
            protocol: "RSTP".to_string(), // Rapid Spanning Tree Protocol
            bridge_priority: 32768,
        };
        
        context.events.push(event);
        tracing::info!("Spanning tree protocol configured successfully");
        Ok(())
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Compensating: Disabling spanning tree protocol");
        
        context.spanning_tree_configured = false;
        
        // Generate spanning tree disabled event
        let correlation_id = CorrelationId::new();
        let causation_id = CausationId::new();
        
        let event = NetworkEvent::SpanningTreeDisabled {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.switch_id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: 1,
            },
            switch_id: self.switch_id,
        };
        
        context.events.push(event);
        Ok(())
    }
}

// Step 5: Verify Configuration
pub struct VerifyConfigurationStep {
    switch_id: SwitchId,
}

impl VerifyConfigurationStep {
    pub fn new(switch_id: SwitchId) -> Self {
        Self { switch_id }
    }
}

#[async_trait]
impl WorkflowStep for VerifyConfigurationStep {
    type Context = SwitchConfigContext;
    type Error = SwitchConfigError;

    fn step_name(&self) -> String {
        "Verify Switch Configuration".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Verifying switch configuration for {}", self.switch_id);
        
        // Verify VLANs are created
        if context.created_vlans.is_empty() {
            return Err(SwitchConfigError::Validation(
                "No VLANs were created".to_string()
            ));
        }
        
        // Verify ports are configured
        for (port_id, vlan_id) in &context.configured_ports {
            if !context.created_vlans.contains_key(vlan_id) {
                return Err(SwitchConfigError::Validation(
                    format!("Port {} is assigned to non-existent VLAN {}", port_id, vlan_id)
                ));
            }
        }
        
        // Verify spanning tree is configured
        if !context.spanning_tree_configured {
            return Err(SwitchConfigError::Validation(
                "Spanning tree protocol is not configured".to_string()
            ));
        }
        
        // Generate configuration verified event
        let correlation_id = CorrelationId::new();
        let causation_id = CausationId::new();
        
        let event = NetworkEvent::SwitchConfigurationVerified {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.switch_id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: 1,
            },
            switch_id: self.switch_id,
            vlans_configured: context.created_vlans.len() as u32,
            ports_configured: context.configured_ports.len() as u32,
            spanning_tree_enabled: context.spanning_tree_configured,
        };
        
        context.events.push(event);
        tracing::info!("Switch configuration verification completed successfully");
        Ok(())
    }

    async fn compensate(&self, _context: &mut Self::Context) -> StepResult<Self::Error> {
        // No compensation needed for verification
        Ok(())
    }
}

/// Factory function to create and execute switch configuration workflow
pub async fn execute_switch_configuration_workflow(
    engine: Arc<dyn WorkflowEngine>,
    switch_id: SwitchId,
    switch_name: String,
    vlan_configs: Vec<VlanConfigRequest>,
    port_assignments: Vec<PortAssignmentRequest>,
) -> Result<Vec<NetworkEvent>, SwitchConfigError> {
    let workflow = SwitchConfigurationWorkflow::new(
        switch_id,
        switch_name,
        vlan_configs,
        port_assignments,
    );
    
    match engine.execute_workflow(workflow).await {
        Ok(context) => Ok(context.events),
        Err(e) => Err(SwitchConfigError::WorkflowEngine(e)),
    }
}