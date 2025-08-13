//! Configuration generation workflow for network devices

use cim_domain_workflow::{
    Workflow, WorkflowStep, WorkflowDefinition, WorkflowError, StepResult,
    WorkflowContext, WorkflowEngine
};
use crate::domain::{RouterId, SwitchId, RouterVendor};
use crate::domain::configuration::cisco_ios::CiscoIosGenerator;
use async_trait::async_trait;
use std::sync::Arc;

/// Configuration generation workflow for network devices
pub struct ConfigurationGenerationWorkflow {
    pub device_id: String,  // Can be RouterId or SwitchId
    pub device_type: DeviceType,
    pub vendor: RouterVendor,
    pub template_type: Option<String>,
}

#[derive(Debug, Clone)]
pub enum DeviceType {
    Router(RouterId),
    Switch(SwitchId),
}

impl ConfigurationGenerationWorkflow {
    pub fn new_for_router(
        router_id: RouterId,
        vendor: RouterVendor,
        template_type: Option<String>,
    ) -> Self {
        Self {
            device_id: router_id.to_string(),
            device_type: DeviceType::Router(router_id),
            vendor,
            template_type,
        }
    }

    pub fn new_for_switch(
        switch_id: SwitchId,
        vendor: RouterVendor,
        template_type: Option<String>,
    ) -> Self {
        Self {
            device_id: switch_id.to_string(),
            device_type: DeviceType::Switch(switch_id),
            vendor,
            template_type,
        }
    }
}

#[async_trait]
impl WorkflowDefinition for ConfigurationGenerationWorkflow {
    type Context = ConfigGenerationContext;
    type Error = ConfigGenerationError;

    fn workflow_id(&self) -> String {
        format!("config_generation_{}", self.device_id)
    }

    fn workflow_name(&self) -> String {
        "Configuration Generation".to_string()
    }

    async fn define_steps(&self) -> Result<Vec<Box<dyn WorkflowStep<Context = Self::Context, Error = Self::Error>>>, Self::Error> {
        Ok(vec![
            Box::new(ValidateInputStep::new(self.device_type.clone(), self.vendor.clone())),
            Box::new(GenerateHeaderStep::new()),
            Box::new(GenerateGlobalConfigStep::new()),
            Box::new(GenerateInterfacesStep::new()),
            Box::new(GenerateRoutingProtocolsStep::new()),
            Box::new(GenerateAccessListsStep::new()),
            Box::new(GenerateFooterStep::new()),
            Box::new(ValidateConfigurationStep::new()),
        ])
    }

    async fn create_context(&self) -> Result<Self::Context, Self::Error> {
        Ok(ConfigGenerationContext {
            device_id: self.device_id.clone(),
            device_type: self.device_type.clone(),
            vendor: self.vendor.clone(),
            template_type: self.template_type.clone(),
            config_sections: ConfigSections::default(),
            final_config: None,
            validation_results: Vec::new(),
        })
    }
}

/// Context for configuration generation workflow
#[derive(Debug, Clone)]
pub struct ConfigGenerationContext {
    pub device_id: String,
    pub device_type: DeviceType,
    pub vendor: RouterVendor,
    pub template_type: Option<String>,
    pub config_sections: ConfigSections,
    pub final_config: Option<String>,
    pub validation_results: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ConfigSections {
    pub header: Option<String>,
    pub global_config: Option<String>,
    pub interfaces: Option<String>,
    pub routing_protocols: Option<String>,
    pub access_lists: Option<String>,
    pub footer: Option<String>,
}

/// Errors that can occur during configuration generation
#[derive(Debug, thiserror::Error)]
pub enum ConfigGenerationError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Generation error: {0}")]
    Generation(String),
    #[error("Template error: {0}")]
    Template(String),
    #[error("Workflow engine error: {0}")]
    WorkflowEngine(#[from] WorkflowError),
}

// Step 1: Validate Input
pub struct ValidateInputStep {
    device_type: DeviceType,
    vendor: RouterVendor,
}

impl ValidateInputStep {
    pub fn new(device_type: DeviceType, vendor: RouterVendor) -> Self {
        Self { device_type, vendor }
    }
}

#[async_trait]
impl WorkflowStep for ValidateInputStep {
    type Context = ConfigGenerationContext;
    type Error = ConfigGenerationError;

    fn step_name(&self) -> String {
        "Validate Input Parameters".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        // Validate device type and vendor compatibility
        match (&context.device_type, &context.vendor) {
            (DeviceType::Router(_), RouterVendor::Cisco { .. }) => {
                tracing::info!("Validated Cisco router configuration request");
            }
            (DeviceType::Switch(_), RouterVendor::Cisco { .. }) => {
                tracing::info!("Validated Cisco switch configuration request");
            }
            _ => {
                return Err(ConfigGenerationError::Validation(
                    format!("Unsupported device/vendor combination: {:?}/{:?}", 
                           context.device_type, context.vendor)
                ));
            }
        }

        // Validate template type if provided
        if let Some(template) = &context.template_type {
            match template.as_str() {
                "EdgeRouter" | "CoreRouter" | "AccessSwitch" | "DistributionSwitch" => {
                    tracing::info!("Validated template type: {}", template);
                }
                _ => {
                    return Err(ConfigGenerationError::Validation(
                        format!("Unknown template type: {}", template)
                    ));
                }
            }
        }

        Ok(())
    }

    async fn compensate(&self, _context: &mut Self::Context) -> StepResult<Self::Error> {
        // No compensation needed for validation
        Ok(())
    }
}

// Step 2: Generate Header
pub struct GenerateHeaderStep;

impl GenerateHeaderStep {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WorkflowStep for GenerateHeaderStep {
    type Context = ConfigGenerationContext;
    type Error = ConfigGenerationError;

    fn step_name(&self) -> String {
        "Generate Configuration Header".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        let header = match &context.vendor {
            RouterVendor::Cisco { .. } => {
                let generator = CiscoIosGenerator::new();
                generator.generate_header()
                    .map_err(|e| ConfigGenerationError::Generation(e.to_string()))?
            }
            _ => {
                return Err(ConfigGenerationError::Generation(
                    "Unsupported vendor for header generation".to_string()
                ));
            }
        };

        context.config_sections.header = Some(header);
        tracing::info!("Generated configuration header");
        Ok(())
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        context.config_sections.header = None;
        Ok(())
    }
}

// Step 3: Generate Global Configuration
pub struct GenerateGlobalConfigStep;

impl GenerateGlobalConfigStep {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WorkflowStep for GenerateGlobalConfigStep {
    type Context = ConfigGenerationContext;
    type Error = ConfigGenerationError;

    fn step_name(&self) -> String {
        "Generate Global Configuration".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        let global_config = match &context.vendor {
            RouterVendor::Cisco { .. } => {
                let generator = CiscoIosGenerator::new();
                generator.generate_global_config()
                    .map_err(|e| ConfigGenerationError::Generation(e.to_string()))?
            }
            _ => {
                return Err(ConfigGenerationError::Generation(
                    "Unsupported vendor for global configuration".to_string()
                ));
            }
        };

        context.config_sections.global_config = Some(global_config);
        tracing::info!("Generated global configuration");
        Ok(())
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        context.config_sections.global_config = None;
        Ok(())
    }
}

// Step 4: Generate Interfaces
pub struct GenerateInterfacesStep;

impl GenerateInterfacesStep {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WorkflowStep for GenerateInterfacesStep {
    type Context = ConfigGenerationContext;
    type Error = ConfigGenerationError;

    fn step_name(&self) -> String {
        "Generate Interface Configuration".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        let interfaces = match &context.vendor {
            RouterVendor::Cisco { .. } => {
                let generator = CiscoIosGenerator::new();
                generator.generate_interfaces(&[]) // Empty interfaces for now
                    .map_err(|e| ConfigGenerationError::Generation(e.to_string()))?
            }
            _ => {
                return Err(ConfigGenerationError::Generation(
                    "Unsupported vendor for interface configuration".to_string()
                ));
            }
        };

        context.config_sections.interfaces = Some(interfaces);
        tracing::info!("Generated interface configuration");
        Ok(())
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        context.config_sections.interfaces = None;
        Ok(())
    }
}

// Step 5: Generate Routing Protocols
pub struct GenerateRoutingProtocolsStep;

impl GenerateRoutingProtocolsStep {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WorkflowStep for GenerateRoutingProtocolsStep {
    type Context = ConfigGenerationContext;
    type Error = ConfigGenerationError;

    fn step_name(&self) -> String {
        "Generate Routing Protocol Configuration".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        let routing = match &context.vendor {
            RouterVendor::Cisco { .. } => {
                let generator = CiscoIosGenerator::new();
                generator.generate_routing_protocols(&[], &[]) // Empty protocols for now
                    .map_err(|e| ConfigGenerationError::Generation(e.to_string()))?
            }
            _ => {
                return Err(ConfigGenerationError::Generation(
                    "Unsupported vendor for routing protocol configuration".to_string()
                ));
            }
        };

        context.config_sections.routing_protocols = Some(routing);
        tracing::info!("Generated routing protocol configuration");
        Ok(())
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        context.config_sections.routing_protocols = None;
        Ok(())
    }
}

// Step 6: Generate Access Lists
pub struct GenerateAccessListsStep;

impl GenerateAccessListsStep {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WorkflowStep for GenerateAccessListsStep {
    type Context = ConfigGenerationContext;
    type Error = ConfigGenerationError;

    fn step_name(&self) -> String {
        "Generate Access List Configuration".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        let access_lists = match &context.vendor {
            RouterVendor::Cisco { .. } => {
                let generator = CiscoIosGenerator::new();
                generator.generate_access_lists(&[]) // Empty ACLs for now
                    .map_err(|e| ConfigGenerationError::Generation(e.to_string()))?
            }
            _ => {
                return Err(ConfigGenerationError::Generation(
                    "Unsupported vendor for access list configuration".to_string()
                ));
            }
        };

        context.config_sections.access_lists = Some(access_lists);
        tracing::info!("Generated access list configuration");
        Ok(())
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        context.config_sections.access_lists = None;
        Ok(())
    }
}

// Step 7: Generate Footer
pub struct GenerateFooterStep;

impl GenerateFooterStep {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WorkflowStep for GenerateFooterStep {
    type Context = ConfigGenerationContext;
    type Error = ConfigGenerationError;

    fn step_name(&self) -> String {
        "Generate Configuration Footer".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        let footer = match &context.vendor {
            RouterVendor::Cisco { .. } => {
                let generator = CiscoIosGenerator::new();
                generator.generate_footer()
                    .map_err(|e| ConfigGenerationError::Generation(e.to_string()))?
            }
            _ => {
                return Err(ConfigGenerationError::Generation(
                    "Unsupported vendor for footer generation".to_string()
                ));
            }
        };

        context.config_sections.footer = Some(footer);
        
        // Combine all sections into final configuration
        let mut final_config = String::new();
        
        if let Some(header) = &context.config_sections.header {
            final_config.push_str(header);
            final_config.push('\n');
        }
        
        if let Some(global) = &context.config_sections.global_config {
            final_config.push_str(global);
            final_config.push('\n');
        }
        
        if let Some(interfaces) = &context.config_sections.interfaces {
            final_config.push_str(interfaces);
            final_config.push('\n');
        }
        
        if let Some(routing) = &context.config_sections.routing_protocols {
            final_config.push_str(routing);
            final_config.push('\n');
        }
        
        if let Some(acls) = &context.config_sections.access_lists {
            final_config.push_str(acls);
            final_config.push('\n');
        }
        
        final_config.push_str(&footer);
        
        context.final_config = Some(final_config);
        tracing::info!("Generated configuration footer and assembled final configuration");
        Ok(())
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        context.config_sections.footer = None;
        context.final_config = None;
        Ok(())
    }
}

// Step 8: Validate Configuration
pub struct ValidateConfigurationStep;

impl ValidateConfigurationStep {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WorkflowStep for ValidateConfigurationStep {
    type Context = ConfigGenerationContext;
    type Error = ConfigGenerationError;

    fn step_name(&self) -> String {
        "Validate Generated Configuration".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        if let Some(config) = &context.final_config {
            // Basic validation checks
            let mut validation_results = Vec::new();
            
            // Check if configuration is not empty
            if config.trim().is_empty() {
                return Err(ConfigGenerationError::Validation(
                    "Generated configuration is empty".to_string()
                ));
            }
            
            // Check for required sections based on vendor
            match &context.vendor {
                RouterVendor::Cisco { .. } => {
                    if !config.contains("version") {
                        validation_results.push("Warning: No version information found".to_string());
                    }
                    
                    if !config.contains("hostname") {
                        validation_results.push("Warning: No hostname configured".to_string());
                    }
                    
                    if config.contains("interface") {
                        validation_results.push("Info: Interface configuration found".to_string());
                    }
                }
                _ => {
                    validation_results.push("Info: Basic validation completed".to_string());
                }
            }
            
            context.validation_results = validation_results;
            tracing::info!("Configuration validation completed successfully");
            Ok(())
        } else {
            Err(ConfigGenerationError::Validation(
                "No configuration available for validation".to_string()
            ))
        }
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        context.validation_results.clear();
        Ok(())
    }
}

/// Factory function to create and execute configuration generation workflow
pub async fn execute_configuration_generation_workflow(
    engine: Arc<dyn WorkflowEngine>,
    device_id: String,
    device_type: DeviceType,
    vendor: RouterVendor,
    template_type: Option<String>,
) -> Result<String, ConfigGenerationError> {
    let workflow = ConfigurationGenerationWorkflow {
        device_id: device_id.clone(),
        device_type,
        vendor,
        template_type,
    };
    
    match engine.execute_workflow(workflow).await {
        Ok(context) => {
            if let Some(final_config) = context.final_config {
                Ok(final_config)
            } else {
                Err(ConfigGenerationError::Generation(
                    "Workflow completed but no final configuration was generated".to_string()
                ))
            }
        }
        Err(e) => Err(ConfigGenerationError::WorkflowEngine(e)),
    }
}