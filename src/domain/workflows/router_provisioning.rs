//! Router provisioning workflow definition

use cim_domain_workflow::{
    Workflow, WorkflowStep, WorkflowDefinition, WorkflowError, StepResult,
    WorkflowContext, WorkflowEngine
};
use crate::domain::{RouterId, RouterVendor, CorrelationId, CausationId};
use crate::domain::events::{NetworkEvent, EventMetadata, EventId, RouterConfigSnapshot, DeploymentMethod};
use crate::domain::state_machines::router::{RouterStateMachine, Planned, Provisioning, Configuring, Active, Failed};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;

/// Router provisioning workflow definition
pub struct RouterProvisioningWorkflow {
    pub router_id: RouterId,
    pub router_name: String,
    pub vendor: RouterVendor,
}

impl RouterProvisioningWorkflow {
    pub fn new(router_id: RouterId, router_name: String, vendor: RouterVendor) -> Self {
        Self {
            router_id,
            router_name,
            vendor,
        }
    }
}

#[async_trait]
impl WorkflowDefinition for RouterProvisioningWorkflow {
    type Context = RouterProvisioningContext;
    type Error = RouterProvisioningError;

    fn workflow_id(&self) -> String {
        format!("router_provisioning_{}", self.router_id)
    }

    fn workflow_name(&self) -> String {
        "Router Provisioning".to_string()
    }

    async fn define_steps(&self) -> Result<Vec<Box<dyn WorkflowStep<Context = Self::Context, Error = Self::Error>>>, Self::Error> {
        Ok(vec![
            Box::new(StartProvisioningStep::new(self.router_id, self.vendor.clone())),
            Box::new(ProvisionInfrastructureStep::new(self.router_id)),
            Box::new(ConfigureRouterStep::new(self.router_id)),
            Box::new(ActivateRouterStep::new(self.router_id)),
        ])
    }

    async fn create_context(&self) -> Result<Self::Context, Self::Error> {
        Ok(RouterProvisioningContext {
            router_id: self.router_id,
            router_name: self.router_name.clone(),
            vendor: self.vendor.clone(),
            state_machine: Some(RouterStateMachine::new(
                self.router_id,
                self.router_name.clone(),
                self.vendor.clone()
            )),
            events: Vec::new(),
            config_snapshot: None,
        })
    }
}

/// Context for router provisioning workflow
#[derive(Debug, Clone)]
pub struct RouterProvisioningContext {
    pub router_id: RouterId,
    pub router_name: String,
    pub vendor: RouterVendor,
    pub state_machine: Option<RouterStateMachine<Planned>>,
    pub events: Vec<NetworkEvent>,
    pub config_snapshot: Option<RouterConfigSnapshot>,
}

/// Errors that can occur during router provisioning
#[derive(Debug, thiserror::Error)]
pub enum RouterProvisioningError {
    #[error("State machine error: {0}")]
    StateMachine(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Infrastructure error: {0}")]
    Infrastructure(String),
    #[error("Workflow engine error: {0}")]
    WorkflowEngine(#[from] WorkflowError),
}

// Step 1: Start Provisioning
pub struct StartProvisioningStep {
    router_id: RouterId,
    vendor: RouterVendor,
}

impl StartProvisioningStep {
    pub fn new(router_id: RouterId, vendor: RouterVendor) -> Self {
        Self { router_id, vendor }
    }
}

#[async_trait]
impl WorkflowStep for StartProvisioningStep {
    type Context = RouterProvisioningContext;
    type Error = RouterProvisioningError;

    fn step_name(&self) -> String {
        "Start Provisioning".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        let correlation_id = CorrelationId::new();
        let causation_id = CausationId::new();

        if let Some(state_machine) = context.state_machine.take() {
            match state_machine.start_provisioning(correlation_id, causation_id) {
                Ok((new_state, event)) => {
                    context.events.push(event);
                    // Store provisioning state (would need to convert type in real implementation)
                    Ok(())
                }
                Err(e) => Err(RouterProvisioningError::StateMachine(e)),
            }
        } else {
            Err(RouterProvisioningError::StateMachine("No state machine available".to_string()))
        }
    }

    async fn compensate(&self, _context: &mut Self::Context) -> StepResult<Self::Error> {
        // Compensation logic - cancel provisioning request
        tracing::info!("Compensating: Cancelling provisioning for router {}", self.router_id);
        Ok(())
    }
}

// Step 2: Provision Infrastructure
pub struct ProvisionInfrastructureStep {
    router_id: RouterId,
}

impl ProvisionInfrastructureStep {
    pub fn new(router_id: RouterId) -> Self {
        Self { router_id }
    }
}

#[async_trait]
impl WorkflowStep for ProvisionInfrastructureStep {
    type Context = RouterProvisioningContext;
    type Error = RouterProvisioningError;

    fn step_name(&self) -> String {
        "Provision Infrastructure".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        // Simulate infrastructure provisioning
        tracing::info!("Provisioning infrastructure for router {}", self.router_id);
        
        // In real implementation, this would:
        // 1. Allocate cloud resources
        // 2. Configure network interfaces
        // 3. Set up routing tables
        // 4. Verify connectivity
        
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Complete provisioning transition
        let correlation_id = CorrelationId::new();
        let causation_id = CausationId::new();
        
        // Would need actual provisioning state machine here
        let event = NetworkEvent::RouterProvisioningCompleted {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.router_id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: 2,
            },
            router_id: self.router_id,
        };
        
        context.events.push(event);
        Ok(())
    }

    async fn compensate(&self, _context: &mut Self::Context) -> StepResult<Self::Error> {
        // Compensation logic - teardown infrastructure
        tracing::info!("Compensating: Tearing down infrastructure for router {}", self.router_id);
        Ok(())
    }
}

// Step 3: Configure Router
pub struct ConfigureRouterStep {
    router_id: RouterId,
}

impl ConfigureRouterStep {
    pub fn new(router_id: RouterId) -> Self {
        Self { router_id }
    }
}

#[async_trait]
impl WorkflowStep for ConfigureRouterStep {
    type Context = RouterProvisioningContext;
    type Error = RouterProvisioningError;

    fn step_name(&self) -> String {
        "Configure Router".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        // Simulate router configuration
        tracing::info!("Configuring router {}", self.router_id);
        
        // In real implementation, this would:
        // 1. Generate configuration files
        // 2. Apply configuration to device
        // 3. Verify configuration is active
        // 4. Run connectivity tests
        
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        // Create mock configuration snapshot
        let config_snapshot = RouterConfigSnapshot {
            config_content: format!("! Configuration for router {}\nhostname router-{}", self.router_id, self.router_id),
            checksum: "abc123".to_string(),
            generated_at: Utc::now(),
        };
        
        context.config_snapshot = Some(config_snapshot.clone());
        
        // Generate configuration applied event
        let correlation_id = CorrelationId::new();
        let causation_id = CausationId::new();
        
        let event = NetworkEvent::RouterConfigurationApplied {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.router_id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: 3,
            },
            router_id: self.router_id,
            configuration: config_snapshot,
            deployment_method: DeploymentMethod::Nix { flake_ref: "nixos#network".to_string() },
        };
        
        context.events.push(event);
        Ok(())
    }

    async fn compensate(&self, _context: &mut Self::Context) -> StepResult<Self::Error> {
        // Compensation logic - revert configuration
        tracing::info!("Compensating: Reverting configuration for router {}", self.router_id);
        Ok(())
    }
}

// Step 4: Activate Router
pub struct ActivateRouterStep {
    router_id: RouterId,
}

impl ActivateRouterStep {
    pub fn new(router_id: RouterId) -> Self {
        Self { router_id }
    }
}

#[async_trait]
impl WorkflowStep for ActivateRouterStep {
    type Context = RouterProvisioningContext;
    type Error = RouterProvisioningError;

    fn step_name(&self) -> String {
        "Activate Router".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        // Simulate router activation
        tracing::info!("Activating router {}", self.router_id);
        
        // In real implementation, this would:
        // 1. Enable routing protocols
        // 2. Announce routes
        // 3. Verify traffic flow
        // 4. Update monitoring systems
        
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        // Router is now active - no additional events needed for this step
        tracing::info!("Router {} is now active and ready for traffic", self.router_id);
        Ok(())
    }

    async fn compensate(&self, _context: &mut Self::Context) -> StepResult<Self::Error> {
        // Compensation logic - deactivate router
        tracing::info!("Compensating: Deactivating router {}", self.router_id);
        Ok(())
    }
}

/// Factory function to create and execute router provisioning workflow
pub async fn execute_router_provisioning_workflow(
    engine: Arc<dyn WorkflowEngine>,
    router_id: RouterId,
    router_name: String,
    vendor: RouterVendor,
) -> Result<Vec<NetworkEvent>, RouterProvisioningError> {
    let workflow = RouterProvisioningWorkflow::new(router_id, router_name, vendor);
    
    match engine.execute_workflow(workflow).await {
        Ok(context) => Ok(context.events),
        Err(e) => Err(RouterProvisioningError::WorkflowEngine(e)),
    }
}