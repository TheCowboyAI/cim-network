//! Integration tests for Nix topology generation

use cim_network::domain::aggregates::network_topology::{NetworkTopology, TopologyType};
use cim_network::infrastructure::nix::{
    NixTopologyGenerator, NixTopologyGenerationRequest, NixGenerationOptions,
    DeploymentTarget, ContextGraphTemplateEngine, SimpleFileWriter, SimpleNixFormatter,
    ValidationStatus
};
use cim_network::domain::{IpNetwork, CorrelationId, CausationId};
use cim_network::api::deployment::{NetworkDeploymentAPI, NetworkDeploymentRequest, TopologyTypeSpec};
use std::str::FromStr;
use std::path::PathBuf;
use tempfile::TempDir;

#[tokio::test]
async fn test_single_router_topology_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_dir = temp_dir.path().to_path_buf();

    // Create single router topology
    let ip_network = IpNetwork::from_str("192.168.1.0/24").unwrap();
    let mut topology = NetworkTopology::from_ip_and_name(
        ip_network,
        "test-single-router".to_string(),
        Some(TopologyType::SingleRouter { interface_count: 4 }),
    ).unwrap();

    // Generate Nix configuration
    topology.generate_nix_topology().unwrap();
    
    let generator = NixTopologyGenerator::new(
        Box::new(ContextGraphTemplateEngine::new()),
        Box::new(SimpleFileWriter),
        Box::new(SimpleNixFormatter),
    );

    let options = NixGenerationOptions {
        deployment_target: DeploymentTarget::Local,
        generate_documentation: true,
        include_examples: true,
        custom_modules: std::collections::HashMap::new(),
        flake_inputs: std::collections::HashMap::new(),
        template_overrides: None,
        output_directory: output_dir.clone(),
    };

    let request = NixTopologyGenerationRequest {
        network_topology: topology.clone(),
        options,
        correlation_id: CorrelationId::new(),
        causation_id: CausationId::new(),
    };

    let response = generator.generate_topology(request).await.unwrap();

    // Verify generation results
    assert!(matches!(response.validation_results.overall_status, ValidationStatus::Valid));
    assert_eq!(response.events.len(), 2); // Started and completed events
    
    // Verify files were generated
    assert!(response.generated_files.flake_nix.exists());
    assert!(response.generated_files.topology_nix.exists());
    assert_eq!(response.generated_files.nixos_modules.len(), 1); // Single router
    
    // Verify documentation
    assert!(response.documentation.mermaid_diagram.is_some());
    assert!(response.documentation.readme.is_some());
    
    let mermaid_file = response.documentation.mermaid_diagram.unwrap();
    assert!(mermaid_file.exists());
    
    let readme_file = response.documentation.readme.unwrap();
    assert!(readme_file.exists());

    // Verify content of generated files
    let flake_content = tokio::fs::read_to_string(&response.generated_files.flake_nix).await.unwrap();
    assert!(flake_content.contains("test-single-router"));
    assert!(flake_content.contains("nixos-unstable"));
    assert!(flake_content.contains("nix-topology"));

    let topology_content = tokio::fs::read_to_string(&response.generated_files.topology_nix).await.unwrap();
    assert!(topology_content.contains("192.168.1.0/24"));
    assert!(topology_content.contains("router"));

    let readme_content = tokio::fs::read_to_string(&readme_file).await.unwrap();
    assert!(readme_content.contains("test-single-router"));
    assert!(readme_content.contains("Network Deployment"));
    assert!(readme_content.contains("nix run .#validate"));
}

#[tokio::test]
async fn test_router_switch_topology_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_dir = temp_dir.path().to_path_buf();

    // Create router-switch topology
    let ip_network = IpNetwork::from_str("10.0.0.0/16").unwrap();
    let mut topology = NetworkTopology::from_ip_and_name(
        ip_network,
        "test-router-switch".to_string(),
        Some(TopologyType::RouterSwitch { 
            switch_count: 2, 
            ports_per_switch: 24 
        }),
    ).unwrap();

    topology.generate_nix_topology().unwrap();
    
    let generator = NixTopologyGenerator::new(
        Box::new(ContextGraphTemplateEngine::new()),
        Box::new(SimpleFileWriter),
        Box::new(SimpleNixFormatter),
    );

    let options = NixGenerationOptions {
        deployment_target: DeploymentTarget::Local,
        generate_documentation: true,
        include_examples: true,
        custom_modules: std::collections::HashMap::new(),
        flake_inputs: std::collections::HashMap::new(),
        template_overrides: None,
        output_directory: output_dir.clone(),
    };

    let request = NixTopologyGenerationRequest {
        network_topology: topology.clone(),
        options,
        correlation_id: CorrelationId::new(),
        causation_id: CausationId::new(),
    };

    let response = generator.generate_topology(request).await.unwrap();

    // Verify topology has expected devices
    assert_eq!(topology.devices().len(), 3); // 1 router + 2 switches
    
    // Verify generation results
    assert!(matches!(response.validation_results.overall_status, ValidationStatus::Valid));
    assert_eq!(response.generated_files.nixos_modules.len(), 3); // Router + 2 switches
    
    // Verify connections exist
    assert!(!topology.connections().is_empty());
    
    // Verify Mermaid diagram contains expected elements
    let mermaid_file = response.documentation.mermaid_diagram.unwrap();
    let mermaid_content = tokio::fs::read_to_string(&mermaid_file).await.unwrap();
    assert!(mermaid_content.contains("graph TB"));
    assert!(mermaid_content.contains("router"));
    assert!(mermaid_content.contains("switch"));
}

#[tokio::test]
async fn test_api_simple_network_deployment() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    let request = NetworkDeploymentRequest {
        ip_network: "172.16.0.0/24".to_string(),
        network_name: "test-api-network".to_string(),
        topology_type: Some(TopologyTypeSpec::SingleRouter { interface_count: 2 }),
        output_directory: Some(temp_dir.path().to_path_buf()),
        generate_documentation: Some(true),
        include_examples: Some(true),
    };

    let response = NetworkDeploymentAPI::deploy_network(request).await.unwrap();

    assert!(response.success);
    assert!(response.topology_id.is_some());
    assert_eq!(response.device_count, 1); // Single router
    assert!(!response.generated_files.is_empty());
    assert!(!response.documentation_files.is_empty());
    assert!(!response.deployment_instructions.is_empty());
    
    // Verify deployment instructions contain expected commands
    let instructions = response.deployment_instructions.join(" ");
    assert!(instructions.contains("nix run .#validate"));
    assert!(instructions.contains("nix run .#topology"));
    assert!(instructions.contains("nix run .#deploy"));
}

#[tokio::test]
async fn test_router_switch_api_deployment() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    let response = NetworkDeploymentAPI::create_router_switch_network(
        "192.168.10.0/24",
        "test-office".to_string(),
        1, // 1 switch
        48, // 48 ports
        Some(temp_dir.path().to_path_buf()),
    ).await.unwrap();

    assert!(response.success);
    assert_eq!(response.device_count, 2); // 1 router + 1 switch
    assert!(response.generated_files.len() >= 4); // flake.nix, topology.nix, 2 modules
    assert!(!response.documentation_files.is_empty());
    
    // Check that output directory structure was created
    let output_dir = temp_dir.path();
    assert!(output_dir.join("flake.nix").exists());
    assert!(output_dir.join("topology.nix").exists());
    assert!(output_dir.join("modules").exists());
    assert!(output_dir.join("docs").exists());
    assert!(output_dir.join("scripts").exists());
}

#[tokio::test]
async fn test_topology_auto_detection() {
    // Test different IP ranges result in different topologies
    
    // Small network should get router-switch
    let ip_small = IpNetwork::from_str("192.168.1.0/24").unwrap();
    let topology_small = NetworkTopology::from_ip_and_name(
        ip_small,
        "small-network".to_string(),
        None, // Auto-detect
    ).unwrap();
    
    // Should auto-detect as RouterSwitch for /24
    assert!(matches!(topology_small.topology_type(), 
                    TopologyType::RouterSwitch { .. }));
    
    // Large network should get three-tier
    let ip_large = IpNetwork::from_str("10.0.0.0/16").unwrap();
    let topology_large = NetworkTopology::from_ip_and_name(
        ip_large,
        "large-network".to_string(),
        None, // Auto-detect
    ).unwrap();
    
    // Should auto-detect as ThreeTier for /16
    assert!(matches!(topology_large.topology_type(), 
                    TopologyType::ThreeTier { .. }));
}

#[tokio::test]
async fn test_nix_configuration_content_validation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let output_dir = temp_dir.path().to_path_buf();

    let ip_network = IpNetwork::from_str("192.168.100.0/24").unwrap();
    let mut topology = NetworkTopology::from_ip_and_name(
        ip_network,
        "content-test-network".to_string(),
        Some(TopologyType::SingleRouter { interface_count: 2 }),
    ).unwrap();

    topology.generate_nix_topology().unwrap();
    
    let generator = NixTopologyGenerator::new(
        Box::new(ContextGraphTemplateEngine::new()),
        Box::new(SimpleFileWriter),
        Box::new(SimpleNixFormatter),
    );

    let options = NixGenerationOptions {
        deployment_target: DeploymentTarget::Local,
        generate_documentation: true,
        include_examples: true,
        custom_modules: std::collections::HashMap::new(),
        flake_inputs: std::collections::HashMap::new(),
        template_overrides: None,
        output_directory: output_dir.clone(),
    };

    let request = NixTopologyGenerationRequest {
        network_topology: topology,
        options,
        correlation_id: CorrelationId::new(),
        causation_id: CausationId::new(),
    };

    let response = generator.generate_topology(request).await.unwrap();

    // Validate flake.nix content
    let flake_content = tokio::fs::read_to_string(&response.generated_files.flake_nix).await.unwrap();
    
    // Must contain required inputs
    assert!(flake_content.contains("nixpkgs.url"));
    assert!(flake_content.contains("nix-topology.url"));
    assert!(flake_content.contains("cim-network.url"));
    
    // Must contain required outputs
    assert!(flake_content.contains("nixosConfigurations"));
    assert!(flake_content.contains("devShells"));
    assert!(flake_content.contains("packages"));
    
    // Must contain validation and deployment tools
    assert!(flake_content.contains("validate"));
    assert!(flake_content.contains("deploy"));
    assert!(flake_content.contains("topology"));

    // Validate topology.nix content
    let topology_content = tokio::fs::read_to_string(&response.generated_files.topology_nix).await.unwrap();
    
    // Must contain network definitions
    assert!(topology_content.contains("networks ="));
    assert!(topology_content.contains("nodes ="));
    assert!(topology_content.contains("connections ="));
    
    // Must contain correct IP range
    assert!(topology_content.contains("192.168.100.0/24"));
    
    // Must contain device definitions
    assert!(topology_content.contains("deviceType"));
    assert!(topology_content.contains("interfaces"));

    // Validate device module content
    let router_module_path = response.generated_files.nixos_modules
        .values()
        .next()
        .expect("Should have at least one module");
    
    let module_content = tokio::fs::read_to_string(router_module_path).await.unwrap();
    
    // Must be valid NixOS module structure
    assert!(module_content.contains("{ config, lib, pkgs, ... }"));
    assert!(module_content.contains("system.stateVersion"));
    assert!(module_content.contains("networking"));
    assert!(module_content.contains("services"));
    
    // Must contain router-specific configuration
    assert!(module_content.contains("forwarding = true"));
    assert!(module_content.contains("openssh"));
}

#[tokio::test]
async fn test_error_handling() {
    // Test invalid IP network
    let request = NetworkDeploymentRequest {
        ip_network: "invalid-ip".to_string(),
        network_name: "test-network".to_string(),
        topology_type: None,
        output_directory: None,
        generate_documentation: None,
        include_examples: None,
    };

    let result = NetworkDeploymentAPI::deploy_network(request).await;
    assert!(result.is_err());
    
    // Test empty network name
    let request = NetworkDeploymentRequest {
        ip_network: "192.168.1.0/24".to_string(),
        network_name: "".to_string(),
        topology_type: None,
        output_directory: None,
        generate_documentation: None,
        include_examples: None,
    };

    let result = NetworkDeploymentAPI::deploy_network(request).await;
    // Should handle empty name gracefully (might succeed with empty name)
    // In production, we'd add validation for this
}

#[tokio::test]
async fn test_different_deployment_targets() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Test different deployment targets
    let targets = vec![
        DeploymentTarget::Local,
        DeploymentTarget::Remote { 
            host: "server.example.com".to_string(), 
            user: "deploy".to_string() 
        },
        DeploymentTarget::Container { 
            runtime: "docker".to_string() 
        },
        DeploymentTarget::VM { 
            hypervisor: "kvm".to_string() 
        },
    ];

    for target in targets {
        let ip_network = IpNetwork::from_str("192.168.200.0/24").unwrap();
        let mut topology = NetworkTopology::from_ip_and_name(
            ip_network,
            "target-test".to_string(),
            Some(TopologyType::SingleRouter { interface_count: 1 }),
        ).unwrap();

        topology.generate_nix_topology().unwrap();
        
        let generator = NixTopologyGenerator::new(
            Box::new(ContextGraphTemplateEngine::new()),
            Box::new(SimpleFileWriter),
            Box::new(SimpleNixFormatter),
        );

        let options = NixGenerationOptions {
            deployment_target: target.clone(),
            generate_documentation: false,
            include_examples: false,
            custom_modules: std::collections::HashMap::new(),
            flake_inputs: std::collections::HashMap::new(),
            template_overrides: None,
            output_directory: temp_dir.path().to_path_buf(),
        };

        let request = NixTopologyGenerationRequest {
            network_topology: topology,
            options,
            correlation_id: CorrelationId::new(),
            causation_id: CausationId::new(),
        };

        let response = generator.generate_topology(request).await;
        assert!(response.is_ok(), "Failed for deployment target: {:?}", target);
    }
}

#[tokio::test] 
async fn test_event_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    let ip_network = IpNetwork::from_str("192.168.50.0/24").unwrap();
    let mut topology = NetworkTopology::from_ip_and_name(
        ip_network,
        "event-test".to_string(),
        Some(TopologyType::SingleRouter { interface_count: 1 }),
    ).unwrap();

    topology.generate_nix_topology().unwrap();
    
    let generator = NixTopologyGenerator::new(
        Box::new(ContextGraphTemplateEngine::new()),
        Box::new(SimpleFileWriter),
        Box::new(SimpleNixFormatter),
    );

    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();

    let options = NixGenerationOptions {
        deployment_target: DeploymentTarget::Local,
        generate_documentation: false,
        include_examples: false,
        custom_modules: std::collections::HashMap::new(),
        flake_inputs: std::collections::HashMap::new(),
        template_overrides: None,
        output_directory: temp_dir.path().to_path_buf(),
    };

    let request = NixTopologyGenerationRequest {
        network_topology: topology.clone(),
        options,
        correlation_id,
        causation_id,
    };

    let response = generator.generate_topology(request).await.unwrap();

    // Should generate start and completion events
    assert_eq!(response.events.len(), 2);
    
    // Verify event metadata
    for event in &response.events {
        match event {
            cim_network::domain::events::NetworkEvent::NixTopologyGenerationStarted { 
                metadata, topology_id, .. 
            } => {
                assert_eq!(metadata.correlation_id, correlation_id);
                assert_eq!(metadata.causation_id, causation_id);
                assert_eq!(*topology_id, topology.id());
            }
            cim_network::domain::events::NetworkEvent::NixTopologyGenerationCompleted { 
                metadata, topology_id, file_count, .. 
            } => {
                assert_eq!(metadata.correlation_id, correlation_id);
                assert_eq!(metadata.causation_id, causation_id);
                assert_eq!(*topology_id, topology.id());
                assert!(*file_count > 0);
            }
            _ => panic!("Unexpected event type: {:?}", event),
        }
    }
}

#[tokio::test]
async fn test_documentation_generation() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Test with documentation enabled
    let response_with_docs = NetworkDeploymentAPI::create_router_switch_network(
        "192.168.30.0/24",
        "docs-test".to_string(),
        1,
        24,
        Some(temp_dir.path().join("with-docs")),
    ).await.unwrap();

    assert!(response_with_docs.success);
    assert!(!response_with_docs.documentation_files.is_empty());
    
    // Verify documentation files exist
    for doc_file in &response_with_docs.documentation_files {
        let path = PathBuf::from(doc_file);
        assert!(path.exists(), "Documentation file should exist: {}", doc_file);
    }
    
    // Test README content
    let readme_path = response_with_docs.documentation_files
        .iter()
        .find(|f| f.ends_with("README.md"))
        .expect("Should have README.md");
    
    let readme_content = tokio::fs::read_to_string(readme_path).await.unwrap();
    assert!(readme_content.contains("docs-test"));
    assert!(readme_content.contains("Quick Start"));
    assert!(readme_content.contains("Device Configuration"));
    assert!(readme_content.contains("Deployment Commands"));
}

#[test]
fn test_topology_type_conversions() {
    // Test TopologyTypeSpec to TopologyType conversions
    let single_router = TopologyTypeSpec::SingleRouter { interface_count: 8 };
    let topology_type: TopologyType = single_router.into();
    assert!(matches!(topology_type, TopologyType::SingleRouter { interface_count: 8 }));

    let router_switch = TopologyTypeSpec::RouterSwitch { 
        switch_count: 3, 
        ports_per_switch: 48 
    };
    let topology_type: TopologyType = router_switch.into();
    assert!(matches!(topology_type, TopologyType::RouterSwitch { 
        switch_count: 3, 
        ports_per_switch: 48 
    }));

    let three_tier = TopologyTypeSpec::ThreeTier {
        core_count: 2,
        distribution_count: 4,
        access_count: 8,
        hosts_per_access: 24,
    };
    let topology_type: TopologyType = three_tier.into();
    assert!(matches!(topology_type, TopologyType::ThreeTier {
        core_count: 2,
        distribution_count: 4,
        access_count: 8,
        hosts_per_access: 24,
    }));

    let spine_leaf = TopologyTypeSpec::SpineLeaf {
        spine_count: 4,
        leaf_count: 16,
        hosts_per_leaf: 32,
    };
    let topology_type: TopologyType = spine_leaf.into();
    assert!(matches!(topology_type, TopologyType::SpineLeaf {
        spine_count: 4,
        leaf_count: 16,
        hosts_per_leaf: 32,
    }));
}