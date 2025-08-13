//! Network deployment API - single entry point for deploying networks from IP and name

use crate::domain::{IpNetwork, NetworkError};
use crate::domain::aggregates::network_topology::{NetworkTopology, TopologyType};
use crate::domain::workflows::nix_deployment::{
    NetworkDeploymentOptions, NetworkDeploymentResult, NixDeploymentError
};
use crate::infrastructure::nix::{
    NixTopologyGenerator, NixTopologyGenerationRequest, NixGenerationOptions,
    ContextGraphTemplateEngine, SimpleFileWriter, SimpleNixFormatter, DeploymentTarget
};
use std::path::PathBuf;
use std::str::FromStr;
use serde::{Serialize, Deserialize};

/// Simple network deployment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDeploymentRequest {
    /// IP network (e.g., "192.168.1.0/24")
    pub ip_network: String,
    /// Network name (e.g., "home-lab")
    pub network_name: String,
    /// Optional topology type (auto-detected if not specified)
    pub topology_type: Option<TopologyTypeSpec>,
    /// Output directory for generated files
    pub output_directory: Option<PathBuf>,
    /// Whether to generate documentation
    pub generate_documentation: Option<bool>,
    /// Whether to include examples
    pub include_examples: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopologyTypeSpec {
    /// Single router with specified interfaces
    SingleRouter { interface_count: u8 },
    /// Router connected to switches
    RouterSwitch { switch_count: u8, ports_per_switch: u8 },
    /// Three-tier architecture
    ThreeTier { 
        core_count: u8, 
        distribution_count: u8, 
        access_count: u8,
        hosts_per_access: u8,
    },
    /// Spine-leaf architecture
    SpineLeaf { 
        spine_count: u8, 
        leaf_count: u8,
        hosts_per_leaf: u8,
    },
}

impl From<TopologyTypeSpec> for TopologyType {
    fn from(spec: TopologyTypeSpec) -> Self {
        match spec {
            TopologyTypeSpec::SingleRouter { interface_count } => {
                TopologyType::SingleRouter { interface_count }
            }
            TopologyTypeSpec::RouterSwitch { switch_count, ports_per_switch } => {
                TopologyType::RouterSwitch { switch_count, ports_per_switch }
            }
            TopologyTypeSpec::ThreeTier { core_count, distribution_count, access_count, hosts_per_access } => {
                TopologyType::ThreeTier { core_count, distribution_count, access_count, hosts_per_access }
            }
            TopologyTypeSpec::SpineLeaf { spine_count, leaf_count, hosts_per_leaf } => {
                TopologyType::SpineLeaf { spine_count, leaf_count, hosts_per_leaf }
            }
        }
    }
}

/// Network deployment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDeploymentResponse {
    /// Success status
    pub success: bool,
    /// Topology ID if successful
    pub topology_id: Option<String>,
    /// Generated file paths
    pub generated_files: Vec<String>,
    /// Documentation file paths
    pub documentation_files: Vec<String>,
    /// Number of devices created
    pub device_count: usize,
    /// Number of events generated
    pub event_count: usize,
    /// Error message if failed
    pub error_message: Option<String>,
    /// Deployment instructions
    pub deployment_instructions: Vec<String>,
}

/// Main API for network deployment
pub struct NetworkDeploymentAPI;

impl NetworkDeploymentAPI {
    /// Deploy a complete network from just IP and name
    pub async fn deploy_network(
        request: NetworkDeploymentRequest,
    ) -> Result<NetworkDeploymentResponse, NetworkDeploymentError> {
        tracing::info!("Starting network deployment for '{}'", request.network_name);
        
        // Parse IP network
        let ip_network = IpNetwork::from_str(&request.ip_network)
            .map_err(|e| NetworkDeploymentError::InvalidInput(
                format!("Invalid IP network '{}': {}", request.ip_network, e)
            ))?;
        
        // Create network topology
        let mut topology = NetworkTopology::from_ip_and_name(
            ip_network,
            request.network_name.clone(),
            request.topology_type.map(|t| t.into()),
        ).map_err(|e| NetworkDeploymentError::TopologyCreation(e.to_string()))?;
        
        // Generate Nix topology configuration
        topology.generate_nix_topology()
            .map_err(|e| NetworkDeploymentError::TopologyGeneration(e.to_string()))?;
        
        // Create Nix generator
        let generator = NixTopologyGenerator::new(
            Box::new(ContextGraphTemplateEngine::new()),
            Box::new(SimpleFileWriter),
            Box::new(SimpleNixFormatter),
        );
        
        // Prepare generation options
        let output_directory = request.output_directory
            .unwrap_or_else(|| PathBuf::from("./network-deployment"));
        
        let nix_options = NixGenerationOptions {
            deployment_target: DeploymentTarget::Local,
            generate_documentation: request.generate_documentation.unwrap_or(true),
            include_examples: request.include_examples.unwrap_or(true),
            custom_modules: std::collections::HashMap::new(),
            flake_inputs: std::collections::HashMap::new(),
            template_overrides: None,
            output_directory: output_directory.clone(),
        };
        
        let generation_request = NixTopologyGenerationRequest {
            network_topology: topology.clone(),
            options: nix_options,
            correlation_id: crate::domain::CorrelationId::new(),
            causation_id: crate::domain::CausationId::new(),
        };
        
        // Generate Nix configuration
        let generation_response = generator.generate_topology(generation_request).await
            .map_err(|e| NetworkDeploymentError::NixGeneration(e.to_string()))?;
        
        // Build response
        let mut generated_file_paths = vec![
            generation_response.generated_files.flake_nix.to_string_lossy().to_string(),
            generation_response.generated_files.topology_nix.to_string_lossy().to_string(),
        ];
        
        for module_path in generation_response.generated_files.nixos_modules.values() {
            generated_file_paths.push(module_path.to_string_lossy().to_string());
        }
        
        for script_path in &generation_response.generated_files.deployment_scripts {
            generated_file_paths.push(script_path.to_string_lossy().to_string());
        }
        
        let mut documentation_file_paths = Vec::new();
        if let Some(mermaid) = &generation_response.documentation.mermaid_diagram {
            documentation_file_paths.push(mermaid.to_string_lossy().to_string());
        }
        if let Some(readme) = &generation_response.documentation.readme {
            documentation_file_paths.push(readme.to_string_lossy().to_string());
        }
        
        let deployment_instructions = vec![
            "1. Navigate to the output directory".to_string(),
            format!("   cd {}", output_directory.display()),
            "2. Validate the topology:".to_string(),
            "   nix run .#validate".to_string(),
            "3. Generate network diagram:".to_string(),
            "   nix run .#topology".to_string(),
            "4. Deploy the network:".to_string(),
            "   nix run .#deploy".to_string(),
            "5. Or deploy individual devices:".to_string(),
        ];
        
        let response = NetworkDeploymentResponse {
            success: true,
            topology_id: Some(topology.id().to_string()),
            generated_files: generated_file_paths,
            documentation_files: documentation_file_paths,
            device_count: topology.devices().len(),
            event_count: generation_response.events.len(),
            error_message: None,
            deployment_instructions,
        };
        
        tracing::info!("Network deployment completed successfully for '{}'", request.network_name);
        Ok(response)
    }
    
    /// Create a simple network with minimal configuration
    pub async fn create_simple_network(
        ip_network: &str,
        network_name: String,
    ) -> Result<NetworkDeploymentResponse, NetworkDeploymentError> {
        let request = NetworkDeploymentRequest {
            ip_network: ip_network.to_string(),
            network_name,
            topology_type: None, // Auto-detect
            output_directory: None, // Use default
            generate_documentation: Some(true),
            include_examples: Some(true),
        };
        
        Self::deploy_network(request).await
    }
    
    /// Create a router-switch network
    pub async fn create_router_switch_network(
        ip_network: &str,
        network_name: String,
        switch_count: u8,
        ports_per_switch: u8,
        output_dir: Option<PathBuf>,
    ) -> Result<NetworkDeploymentResponse, NetworkDeploymentError> {
        let request = NetworkDeploymentRequest {
            ip_network: ip_network.to_string(),
            network_name,
            topology_type: Some(TopologyTypeSpec::RouterSwitch { 
                switch_count, 
                ports_per_switch 
            }),
            output_directory: output_dir,
            generate_documentation: Some(true),
            include_examples: Some(true),
        };
        
        Self::deploy_network(request).await
    }
}

/// Errors that can occur during network deployment
#[derive(Debug, thiserror::Error)]
pub enum NetworkDeploymentError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Topology creation failed: {0}")]
    TopologyCreation(String),
    
    #[error("Topology generation failed: {0}")]
    TopologyGeneration(String),
    
    #[error("Nix generation failed: {0}")]
    NixGeneration(String),
    
    #[error("Network error: {0}")]
    Network(#[from] NetworkError),
}

/// CLI-like interface for simple usage
pub async fn deploy_network_simple(ip: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Deploying network '{}' with IP range {}", name, ip);
    
    let response = NetworkDeploymentAPI::create_simple_network(ip, name.to_string()).await?;
    
    if response.success {
        println!("✅ Network deployment successful!");
        println!("📁 Generated {} files:", response.generated_files.len());
        for file in &response.generated_files {
            println!("   {}", file);
        }
        
        if !response.documentation_files.is_empty() {
            println!("📚 Documentation files:");
            for file in &response.documentation_files {
                println!("   {}", file);
            }
        }
        
        println!("🚀 Deployment instructions:");
        for instruction in &response.deployment_instructions {
            println!("   {}", instruction);
        }
    } else {
        println!("❌ Network deployment failed: {}", 
                response.error_message.unwrap_or_else(|| "Unknown error".to_string()));
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_network_deployment_request() {
        let request = NetworkDeploymentRequest {
            ip_network: "192.168.1.0/24".to_string(),
            network_name: "test-network".to_string(),
            topology_type: Some(TopologyTypeSpec::SingleRouter { interface_count: 4 }),
            output_directory: Some(PathBuf::from("/tmp/test-deployment")),
            generate_documentation: Some(true),
            include_examples: Some(true),
        };
        
        assert_eq!(request.ip_network, "192.168.1.0/24");
        assert_eq!(request.network_name, "test-network");
        assert!(request.generate_documentation.unwrap());
    }
    
    #[test]
    fn test_topology_type_conversion() {
        let spec = TopologyTypeSpec::RouterSwitch { 
            switch_count: 2, 
            ports_per_switch: 24 
        };
        
        let topology_type: TopologyType = spec.into();
        match topology_type {
            TopologyType::RouterSwitch { switch_count, ports_per_switch } => {
                assert_eq!(switch_count, 2);
                assert_eq!(ports_per_switch, 24);
            }
            _ => panic!("Expected RouterSwitch topology type"),
        }
    }
}