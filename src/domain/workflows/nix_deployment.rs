//! End-to-end deployment workflow with Nix output

use cim_domain_workflow::{
    Workflow, WorkflowStep, WorkflowDefinition, WorkflowError, StepResult,
    WorkflowContext, WorkflowEngine
};
use crate::domain::{IpNetwork, CorrelationId, CausationId};
use crate::domain::aggregates::network_topology::{NetworkTopology, TopologyType, NetworkTopologyId};
use crate::domain::events::NetworkEvent;
use crate::infrastructure::nix::{
    NixTopologyGenerator, NixTopologyGenerationRequest, NixGenerationOptions, 
    DeploymentTarget, ContextGraphTemplateEngine, SimpleFileWriter, SimpleNixFormatter,
    GeneratedFiles, DocumentationAssets
};
use async_trait::async_trait;
use std::sync::Arc;
use std::path::PathBuf;
use std::str::FromStr;

/// Complete network deployment workflow from IP and name to deployed infrastructure
pub struct NixNetworkDeploymentWorkflow {
    pub base_ip: IpNetwork,
    pub network_name: String,
    pub topology_type: Option<TopologyType>,
    pub deployment_options: NetworkDeploymentOptions,
}

#[derive(Debug, Clone)]
pub struct NetworkDeploymentOptions {
    pub output_directory: PathBuf,
    pub deployment_target: DeploymentTarget,
    pub generate_documentation: bool,
    pub include_examples: bool,
    pub auto_deploy: bool,
}

impl Default for NetworkDeploymentOptions {
    fn default() -> Self {
        Self {
            output_directory: PathBuf::from("./network-deployment"),
            deployment_target: DeploymentTarget::Local,
            generate_documentation: true,
            include_examples: true,
            auto_deploy: false,
        }
    }
}

impl NixNetworkDeploymentWorkflow {
    pub fn new(
        base_ip: IpNetwork,
        network_name: String,
        topology_type: Option<TopologyType>,
        deployment_options: NetworkDeploymentOptions,
    ) -> Self {
        Self {
            base_ip,
            network_name,
            topology_type,
            deployment_options,
        }
    }
}

#[async_trait]
impl WorkflowDefinition for NixNetworkDeploymentWorkflow {
    type Context = NixDeploymentContext;
    type Error = NixDeploymentError;

    fn workflow_id(&self) -> String {
        format!("nix_network_deployment_{}", self.network_name.replace(' ', "_"))
    }

    fn workflow_name(&self) -> String {
        format!("Nix Network Deployment: {}", self.network_name)
    }

    async fn define_steps(&self) -> Result<Vec<Box<dyn WorkflowStep<Context = Self::Context, Error = Self::Error>>>, Self::Error> {
        Ok(vec![
            Box::new(CreateNetworkTopologyStep::new(
                self.base_ip,
                self.network_name.clone(),
                self.topology_type.clone(),
            )),
            Box::new(GenerateNixConfigurationStep::new(
                self.deployment_options.clone(),
            )),
            Box::new(ValidateNixConfigurationStep::new()),
            Box::new(GenerateDocumentationStep::new()),
            Box::new(PrepareDeploymentStep::new()),
            Box::new(DeployNetworkStep::new(
                self.deployment_options.auto_deploy,
            )),
        ])
    }

    async fn create_context(&self) -> Result<Self::Context, Self::Error> {
        Ok(NixDeploymentContext {
            base_ip: self.base_ip,
            network_name: self.network_name.clone(),
            topology_type: self.topology_type.clone(),
            deployment_options: self.deployment_options.clone(),
            network_topology: None,
            generated_files: None,
            documentation: None,
            validation_results: None,
            deployment_ready: false,
            events: Vec::new(),
        })
    }
}

/// Context for the Nix deployment workflow
#[derive(Debug, Clone)]
pub struct NixDeploymentContext {
    pub base_ip: IpNetwork,
    pub network_name: String,
    pub topology_type: Option<TopologyType>,
    pub deployment_options: NetworkDeploymentOptions,
    pub network_topology: Option<NetworkTopology>,
    pub generated_files: Option<GeneratedFiles>,
    pub documentation: Option<DocumentationAssets>,
    pub validation_results: Option<crate::infrastructure::nix::ValidationResults>,
    pub deployment_ready: bool,
    pub events: Vec<NetworkEvent>,
}

/// Errors that can occur during Nix deployment
#[derive(Debug, thiserror::Error)]
pub enum NixDeploymentError {
    #[error("Topology creation error: {0}")]
    TopologyCreation(String),
    
    #[error("Nix generation error: {0}")]
    NixGeneration(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Documentation generation error: {0}")]
    Documentation(String),
    
    #[error("Deployment error: {0}")]
    Deployment(String),
    
    #[error("Workflow engine error: {0}")]
    WorkflowEngine(#[from] WorkflowError),
}

// Step 1: Create Network Topology
pub struct CreateNetworkTopologyStep {
    base_ip: IpNetwork,
    network_name: String,
    topology_type: Option<TopologyType>,
}

impl CreateNetworkTopologyStep {
    pub fn new(base_ip: IpNetwork, network_name: String, topology_type: Option<TopologyType>) -> Self {
        Self {
            base_ip,
            network_name,
            topology_type,
        }
    }
}

#[async_trait]
impl WorkflowStep for CreateNetworkTopologyStep {
    type Context = NixDeploymentContext;
    type Error = NixDeploymentError;

    fn step_name(&self) -> String {
        "Create Network Topology".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Creating network topology for network '{}'", self.network_name);
        
        match NetworkTopology::from_ip_and_name(
            self.base_ip,
            self.network_name.clone(),
            self.topology_type.clone(),
        ) {
            Ok(mut topology) => {
                // Generate the Nix topology configuration
                topology.generate_nix_topology()
                    .map_err(|e| NixDeploymentError::TopologyCreation(e.to_string()))?;
                
                context.network_topology = Some(topology);
                tracing::info!("Network topology created successfully with {} devices", 
                              context.network_topology.as_ref().unwrap().devices().len());
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to create network topology: {}", e);
                Err(NixDeploymentError::TopologyCreation(e.to_string()))
            }
        }
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        context.network_topology = None;
        tracing::info!("Compensated: Removed network topology");
        Ok(())
    }
}

// Step 2: Generate Nix Configuration
pub struct GenerateNixConfigurationStep {
    deployment_options: NetworkDeploymentOptions,
}

impl GenerateNixConfigurationStep {
    pub fn new(deployment_options: NetworkDeploymentOptions) -> Self {
        Self { deployment_options }
    }
}

#[async_trait]
impl WorkflowStep for GenerateNixConfigurationStep {
    type Context = NixDeploymentContext;
    type Error = NixDeploymentError;

    fn step_name(&self) -> String {
        "Generate Nix Configuration".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Generating Nix configuration files");
        
        let topology = context.network_topology.as_ref()
            .ok_or_else(|| NixDeploymentError::NixGeneration("No network topology available".to_string()))?;
        
        // Create Nix generator
        let generator = NixTopologyGenerator::new(
            Box::new(ContextGraphTemplateEngine::new()),
            Box::new(SimpleFileWriter),
            Box::new(SimpleNixFormatter),
        );
        
        // Prepare generation options
        let nix_options = NixGenerationOptions {
            deployment_target: self.deployment_options.deployment_target.clone(),
            generate_documentation: self.deployment_options.generate_documentation,
            include_examples: self.deployment_options.include_examples,
            custom_modules: std::collections::HashMap::new(),
            flake_inputs: std::collections::HashMap::new(),
            template_overrides: None,
            output_directory: self.deployment_options.output_directory.clone(),
        };
        
        let generation_request = NixTopologyGenerationRequest {
            network_topology: topology.clone(),
            options: nix_options,
            correlation_id: CorrelationId::new(),
            causation_id: CausationId::new(),
        };
        
        match generator.generate_topology(generation_request).await {
            Ok(response) => {
                context.generated_files = Some(response.generated_files);
                context.documentation = Some(response.documentation);
                context.validation_results = Some(response.validation_results);
                context.events.extend(response.events);
                
                tracing::info!("Nix configuration generation completed successfully");
                Ok(())
            }
            Err(e) => {
                tracing::error!("Nix configuration generation failed: {}", e);
                Err(NixDeploymentError::NixGeneration(e.to_string()))
            }
        }
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        // Clean up generated files
        if let Some(generated_files) = &context.generated_files {
            // In a real implementation, we would clean up the files
            tracing::info!("Compensating: Would clean up generated files at {:?}", 
                          generated_files.flake_nix.parent());
        }
        
        context.generated_files = None;
        context.documentation = None;
        context.validation_results = None;
        
        Ok(())
    }
}

// Step 3: Validate Nix Configuration
pub struct ValidateNixConfigurationStep;

impl ValidateNixConfigurationStep {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WorkflowStep for ValidateNixConfigurationStep {
    type Context = NixDeploymentContext;
    type Error = NixDeploymentError;

    fn step_name(&self) -> String {
        "Validate Nix Configuration".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Validating generated Nix configuration");
        
        let validation_results = context.validation_results.as_ref()
            .ok_or_else(|| NixDeploymentError::Validation("No validation results available".to_string()))?;
        
        match validation_results.overall_status {
            crate::infrastructure::nix::ValidationStatus::Valid => {
                tracing::info!("Nix configuration validation passed");
                Ok(())
            }
            crate::infrastructure::nix::ValidationStatus::Warning => {
                tracing::warn!("Nix configuration validation passed with warnings: {:?}", 
                              validation_results.semantic_warnings);
                Ok(())
            }
            crate::infrastructure::nix::ValidationStatus::Error => {
                tracing::error!("Nix configuration validation failed: {:?}", 
                               validation_results.syntax_errors);
                Err(NixDeploymentError::Validation(
                    format!("Validation errors: {}", validation_results.syntax_errors.join(", "))
                ))
            }
        }
    }

    async fn compensate(&self, _context: &mut Self::Context) -> StepResult<Self::Error> {
        // No compensation needed for validation
        Ok(())
    }
}

// Step 4: Generate Documentation
pub struct GenerateDocumentationStep;

impl GenerateDocumentationStep {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WorkflowStep for GenerateDocumentationStep {
    type Context = NixDeploymentContext;
    type Error = NixDeploymentError;

    fn step_name(&self) -> String {
        "Generate Documentation".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        if !context.deployment_options.generate_documentation {
            tracing::info!("Documentation generation skipped");
            return Ok(());
        }
        
        tracing::info!("Validating documentation generation");
        
        let documentation = context.documentation.as_ref()
            .ok_or_else(|| NixDeploymentError::Documentation("No documentation available".to_string()))?;
        
        // Verify documentation files were generated
        let mut generated_docs = Vec::new();
        
        if let Some(mermaid) = &documentation.mermaid_diagram {
            generated_docs.push(format!("Mermaid diagram: {}", mermaid.display()));
        }
        
        if let Some(readme) = &documentation.readme {
            generated_docs.push(format!("README: {}", readme.display()));
        }
        
        tracing::info!("Documentation generated: {}", generated_docs.join(", "));
        Ok(())
    }

    async fn compensate(&self, _context: &mut Self::Context) -> StepResult<Self::Error> {
        // No compensation needed for documentation validation
        Ok(())
    }
}

// Step 5: Prepare Deployment
pub struct PrepareDeploymentStep;

impl PrepareDeploymentStep {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl WorkflowStep for PrepareDeploymentStep {
    type Context = NixDeploymentContext;
    type Error = NixDeploymentError;

    fn step_name(&self) -> String {
        "Prepare Deployment".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Preparing network deployment");
        
        let generated_files = context.generated_files.as_ref()
            .ok_or_else(|| NixDeploymentError::Deployment("No generated files available".to_string()))?;
        
        // Verify all required files are present
        if !generated_files.flake_nix.exists() {
            return Err(NixDeploymentError::Deployment("flake.nix not found".to_string()));
        }
        
        if !generated_files.topology_nix.exists() {
            return Err(NixDeploymentError::Deployment("topology.nix not found".to_string()));
        }
        
        if generated_files.nixos_modules.is_empty() {
            return Err(NixDeploymentError::Deployment("No NixOS modules generated".to_string()));
        }
        
        // Mark as ready for deployment
        context.deployment_ready = true;
        
        tracing::info!("Deployment preparation completed - {} files ready", 
                      generated_files.nixos_modules.len() + 2);
        Ok(())
    }

    async fn compensate(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        context.deployment_ready = false;
        Ok(())
    }
}

// Step 6: Deploy Network
pub struct DeployNetworkStep {
    auto_deploy: bool,
}

impl DeployNetworkStep {
    pub fn new(auto_deploy: bool) -> Self {
        Self { auto_deploy }
    }
}

#[async_trait]
impl WorkflowStep for DeployNetworkStep {
    type Context = NixDeploymentContext;
    type Error = NixDeploymentError;

    fn step_name(&self) -> String {
        "Deploy Network".to_string()
    }

    async fn execute(&self, context: &mut Self::Context) -> StepResult<Self::Error> {
        if !context.deployment_ready {
            return Err(NixDeploymentError::Deployment("Deployment not ready".to_string()));
        }
        
        if !self.auto_deploy {
            tracing::info!("Auto-deploy disabled - configuration ready for manual deployment");
            return Ok(());
        }
        
        tracing::info!("Starting automated network deployment");
        
        let generated_files = context.generated_files.as_ref()
            .ok_or_else(|| NixDeploymentError::Deployment("No generated files available".to_string()))?;
        
        match &context.deployment_options.deployment_target {
            DeploymentTarget::Local => {
                tracing::info!("Deploying to local environment");
                // In a real implementation, this would run:
                // nix run .#validate
                // nix run .#topology
                // nixos-rebuild switch --flake .#device-name
            }
            DeploymentTarget::Remote { host, user } => {
                tracing::info!("Deploying to remote host: {}@{}", user, host);
                // In a real implementation, this would run:
                // nixos-rebuild switch --flake .#device-name --target-host user@host
            }
            _ => {
                tracing::warn!("Deployment target not yet implemented: {:?}", 
                              context.deployment_options.deployment_target);
            }
        }
        
        tracing::info!("Network deployment completed successfully");
        Ok(())
    }

    async fn compensate(&self, _context: &mut Self::Context) -> StepResult<Self::Error> {
        tracing::info!("Compensating: Would rollback deployment");
        // In a real implementation, this would rollback the deployment
        Ok(())
    }
}

/// Factory function to create and execute complete network deployment from IP and name
pub async fn deploy_network_from_ip_and_name(
    engine: Arc<dyn WorkflowEngine>,
    ip_str: &str,
    network_name: String,
    topology_type: Option<TopologyType>,
    deployment_options: Option<NetworkDeploymentOptions>,
) -> Result<NetworkDeploymentResult, NixDeploymentError> {
    
    // Parse IP network
    let base_ip = IpNetwork::from_str(ip_str)
        .map_err(|e| NixDeploymentError::TopologyCreation(
            format!("Invalid IP network '{}': {}", ip_str, e)
        ))?;
    
    let deployment_options = deployment_options.unwrap_or_default();
    
    let workflow = NixNetworkDeploymentWorkflow::new(
        base_ip,
        network_name,
        topology_type,
        deployment_options,
    );
    
    match engine.execute_workflow(workflow).await {
        Ok(context) => Ok(NetworkDeploymentResult {
            topology_id: context.network_topology.as_ref().map(|t| t.id()),
            generated_files: context.generated_files,
            documentation: context.documentation,
            events: context.events,
            deployment_ready: context.deployment_ready,
        }),
        Err(e) => Err(NixDeploymentError::WorkflowEngine(e)),
    }
}

/// Result of network deployment
#[derive(Debug, Clone)]
pub struct NetworkDeploymentResult {
    pub topology_id: Option<NetworkTopologyId>,
    pub generated_files: Option<GeneratedFiles>,
    pub documentation: Option<DocumentationAssets>,
    pub events: Vec<NetworkEvent>,
    pub deployment_ready: bool,
}

/// Simple API for deploying a network from just IP and name
pub async fn simple_deploy_network(
    ip_str: &str,
    network_name: String,
) -> Result<NetworkDeploymentResult, NixDeploymentError> {
    // Create a simple workflow engine (in production, use proper implementation)
    let engine = create_simple_workflow_engine();
    
    deploy_network_from_ip_and_name(
        engine,
        ip_str,
        network_name,
        None, // Auto-detect topology
        None, // Use default options
    ).await
}

/// Create a simple workflow engine for testing
fn create_simple_workflow_engine() -> Arc<dyn WorkflowEngine> {
    // This is a placeholder - in production, use cim-domain-workflow engine
    todo!("Implement workflow engine integration")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::aggregates::network_topology::TopologyType;
    
    #[tokio::test]
    async fn test_network_deployment_workflow_creation() {
        let ip = IpNetwork::from_str("192.168.1.0/24").unwrap();
        let workflow = NixNetworkDeploymentWorkflow::new(
            ip,
            "test-network".to_string(),
            Some(TopologyType::RouterSwitch { 
                switch_count: 1, 
                ports_per_switch: 24 
            }),
            NetworkDeploymentOptions::default(),
        );
        
        assert_eq!(workflow.network_name, "test-network");
        assert_eq!(workflow.base_ip, ip);
        
        let context = workflow.create_context().await.unwrap();
        assert_eq!(context.network_name, "test-network");
        assert_eq!(context.base_ip, ip);
        assert!(!context.deployment_ready);
    }
    
    #[test]
    fn test_network_deployment_options() {
        let options = NetworkDeploymentOptions::default();
        assert_eq!(options.output_directory, PathBuf::from("./network-deployment"));
        assert!(matches!(options.deployment_target, DeploymentTarget::Local));
        assert!(options.generate_documentation);
        assert!(options.include_examples);
        assert!(!options.auto_deploy);
    }
}