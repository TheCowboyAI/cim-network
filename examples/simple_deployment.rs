//! Simple network deployment example
//! 
//! This example demonstrates how to deploy a complete network infrastructure
//! from just an IP range and name using the CIM Network module.

use cim_network::{NetworkDeploymentAPI, deploy_network_simple};
use cim_network::api::deployment::{NetworkDeploymentRequest, TopologyTypeSpec};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::init();
    
    println!("🌐 CIM Network Deployment Examples");
    println!("==================================\n");
    
    // Example 1: Simple network deployment
    println!("Example 1: Simple Network (Auto-detected topology)");
    match deploy_network_simple("192.168.1.0/24", "home-lab").await {
        Ok(_) => println!("✅ Simple network deployment completed\n"),
        Err(e) => println!("❌ Simple network deployment failed: {}\n", e),
    }
    
    // Example 2: Router-switch network
    println!("Example 2: Router-Switch Network");
    let response = NetworkDeploymentAPI::create_router_switch_network(
        "10.0.0.0/16",
        "office-network".to_string(),
        2, // 2 switches
        24, // 24 ports each
        Some(PathBuf::from("./office-network-deployment")),
    ).await?;
    
    if response.success {
        println!("✅ Router-switch network created successfully!");
        println!("   Topology ID: {}", response.topology_id.unwrap_or_else(|| "N/A".to_string()));
        println!("   Devices: {}", response.device_count);
        println!("   Files: {}", response.generated_files.len());
    }
    
    println!();
    
    // Example 3: Custom three-tier network
    println!("Example 3: Three-Tier Data Center Network");
    let request = NetworkDeploymentRequest {
        ip_network: "172.16.0.0/12".to_string(),
        network_name: "datacenter-network".to_string(),
        topology_type: Some(TopologyTypeSpec::ThreeTier {
            core_count: 2,
            distribution_count: 4,
            access_count: 8,
            hosts_per_access: 32,
        }),
        output_directory: Some(PathBuf::from("./datacenter-deployment")),
        generate_documentation: Some(true),
        include_examples: Some(true),
    };
    
    match NetworkDeploymentAPI::deploy_network(request).await {
        Ok(response) => {
            if response.success {
                println!("✅ Three-tier network created successfully!");
                println!("   Devices: {}", response.device_count);
                println!("   Generated files: {}", response.generated_files.len());
                println!("   Documentation files: {}", response.documentation_files.len());
                
                println!("\n📋 Next steps:");
                for instruction in response.deployment_instructions.iter().take(5) {
                    println!("   {}", instruction);
                }
            }
        },
        Err(e) => println!("❌ Three-tier network deployment failed: {}", e),
    }
    
    println!("\n🎉 All examples completed!");
    println!("Check the generated directories for your deployable network configurations.");
    
    Ok(())
}

/// Example showing the complete workflow from IP to deployed network
#[allow(dead_code)]
async fn complete_workflow_example() -> Result<(), Box<dyn std::error::Error>> {
    use cim_network::domain::aggregates::network_topology::{NetworkTopology, TopologyType};
    use cim_network::infrastructure::nix::*;
    use cim_network::domain::{CorrelationId, CausationId, IpNetwork};
    use std::str::FromStr;
    
    // Step 1: Create network topology from IP and name
    let ip_network = IpNetwork::from_str("192.168.1.0/24")?;
    let mut topology = NetworkTopology::from_ip_and_name(
        ip_network,
        "example-network".to_string(),
        Some(TopologyType::RouterSwitch {
            switch_count: 1,
            ports_per_switch: 24,
        }),
    )?;
    
    // Step 2: Generate Nix topology configuration
    topology.generate_nix_topology()?;
    
    // Step 3: Generate complete Nix files
    let generator = NixTopologyGenerator::new(
        Box::new(SimpleTemplateEngine),
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
        output_directory: PathBuf::from("./example-deployment"),
    };
    
    let request = NixTopologyGenerationRequest {
        network_topology: topology,
        options,
        correlation_id: CorrelationId::new(),
        causation_id: CausationId::new(),
    };
    
    // Step 4: Generate all files
    let response = generator.generate_topology(request).await?;
    
    println!("✅ Complete workflow example:");
    println!("   Generated files: {}", response.generated_files.nixos_modules.len() + 2);
    println!("   Events: {}", response.events.len());
    println!("   Validation: {:?}", response.validation_results.overall_status);
    
    // Files are now ready for deployment with:
    // cd ./example-deployment
    // nix run .#validate
    // nix run .#topology
    // nix run .#deploy
    
    Ok(())
}

/// Demonstrate different topology types
#[allow(dead_code)]
async fn topology_examples() -> Result<(), Box<dyn std::error::Error>> {
    println!("🏗️  Network Topology Examples");
    println!("============================");
    
    // Single router topology
    let single_router = TopologyTypeSpec::SingleRouter { interface_count: 8 };
    println!("Single Router: 8 interfaces");
    
    // Router-switch topology
    let router_switch = TopologyTypeSpec::RouterSwitch {
        switch_count: 3,
        ports_per_switch: 48,
    };
    println!("Router-Switch: 3 switches with 48 ports each");
    
    // Three-tier topology
    let three_tier = TopologyTypeSpec::ThreeTier {
        core_count: 2,
        distribution_count: 6,
        access_count: 12,
        hosts_per_access: 24,
    };
    println!("Three-Tier: 2 core + 6 distribution + 12 access (24 hosts each)");
    
    // Spine-leaf topology
    let spine_leaf = TopologyTypeSpec::SpineLeaf {
        spine_count: 4,
        leaf_count: 16,
        hosts_per_leaf: 48,
    };
    println!("Spine-Leaf: 4 spines + 16 leaves (48 hosts each)");
    
    Ok(())
}