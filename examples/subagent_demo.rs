//! Network Topology Sub-Agent Demo
//!
//! This example demonstrates the NetworkTopologySubAgent that can be invoked
//! by Claude Code to build network topologies interactively.

use cim_network::agents::{
    NetworkTopologySubAgent, SubAgentRequest, SubAgentTask, 
    create_subagent_request, create_build_topology_request
};
use std::collections::HashMap;
use std::env;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    
    // Check if we're being called with a request file (from Python wrapper)
    if args.len() >= 3 && args[1] == "--request-file" {
        return handle_request_file(&args[2]).await;
    }
    
    // Otherwise, run the interactive demo
    println!("🤖 Network Topology Sub-Agent Demo");
    println!("===================================");
    println!();

    // Create a new sub-agent instance
    let mut subagent = NetworkTopologySubAgent::new();
    println!("✅ Created sub-agent: {}", subagent.metadata().name);
    println!("   Version: {}", subagent.metadata().version);
    println!("   Description: {}", subagent.metadata().description);
    println!();

    // Example 1: Start topology building
    let build_request = create_build_topology_request(
        Some("10.0.0.0/8".to_string()),
        Some("production".to_string()),
        Some("enterprise".to_string()),
        Some("multi-region-cloud".to_string()),
    );

    println!("📋 Starting topology building...");
    let response = subagent.process_request(build_request).await;
    println!("   Success: {}", response.success);
    println!("   Message: {}", response.message);
    println!();

    // Example 2: Add a data center
    let mut dc_params = HashMap::new();
    dc_params.insert("name".to_string(), "Primary DC".to_string());
    dc_params.insert("region".to_string(), "us-west-1".to_string());
    dc_params.insert("az".to_string(), "us-west-1a".to_string());

    let add_dc_request = create_subagent_request(SubAgentTask::AddLocation {
        location_id: "dc1".to_string(),
        location_type: "datacenter".to_string(),
        parameters: dc_params,
    });

    println!("🏢 Adding data center...");
    let response = subagent.process_request(add_dc_request).await;
    println!("   Success: {}", response.success);
    println!("   Message: {}", response.message);
    println!();

    // Example 3: Add an office
    let mut office_params = HashMap::new();
    office_params.insert("name".to_string(), "Corporate HQ".to_string());
    office_params.insert("address".to_string(), "123 Innovation Drive".to_string());
    office_params.insert("size".to_string(), "large".to_string());

    let add_office_request = create_subagent_request(SubAgentTask::AddLocation {
        location_id: "hq".to_string(),
        location_type: "office".to_string(),
        parameters: office_params,
    });

    println!("🏢 Adding office...");
    let response = subagent.process_request(add_office_request).await;
    println!("   Success: {}", response.success);
    println!("   Message: {}", response.message);
    println!();

    // Example 4: Connect data center to office
    let mut conn_params = HashMap::new();
    conn_params.insert("bandwidth".to_string(), "10Gbps".to_string());
    conn_params.insert("redundant".to_string(), "true".to_string());

    let connect_request = create_subagent_request(SubAgentTask::ConnectLocations {
        from: "dc1".to_string(),
        to: "hq".to_string(),
        connection_type: "fiber".to_string(),
        parameters: conn_params,
    });

    println!("🔗 Connecting locations...");
    let response = subagent.process_request(connect_request).await;
    println!("   Success: {}", response.success);
    println!("   Message: {}", response.message);
    println!();

    // Example 5: Generate NixOS configuration
    let config_request = create_subagent_request(SubAgentTask::GenerateConfiguration {
        format: "nixos".to_string(),
    });

    println!("⚙️ Generating NixOS configuration...");
    let response = subagent.process_request(config_request).await;
    println!("   Success: {}", response.success);
    if response.success {
        println!("   Configuration generated successfully!");
    } else {
        println!("   Error: {}", response.message);
    }
    println!();

    // Example 6: Get topology status
    let status_request = create_subagent_request(SubAgentTask::GetStatus);

    println!("📊 Getting topology status...");
    let response = subagent.process_request(status_request).await;
    println!("   Success: {}", response.success);
    println!("   Status: {}", response.message);
    println!();

    // Show session information
    println!("📋 Session Information:");
    let session_info = subagent.session_info();
    println!("   {}", serde_json::to_string_pretty(&session_info)?);
    println!();

    println!("🎉 Sub-Agent Demo Complete!");
    println!();
    println!("🧠 Key Capabilities Demonstrated:");
    println!("• Interactive topology building through Claude Code integration");
    println!("• Event-driven architecture with full audit trail");
    println!("• Multiple location types: data centers, offices, cloud regions, virtual segments");
    println!("• Various connection types: fiber, VPN, internet, direct connect, virtual");
    println!("• Configuration generation in multiple formats");
    println!("• Intelligent suggestions and validation");
    println!("• Request/response protocol for sub-agent integration");

    Ok(())
}

async fn handle_request_file(request_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read and parse the request from the file
    let request_json = fs::read_to_string(request_file)?;
    let request: SubAgentRequest = serde_json::from_str(&request_json)?;
    
    // Create sub-agent and process the request
    let mut subagent = NetworkTopologySubAgent::new();
    let response = subagent.process_request(request).await;
    
    // Output the response as JSON
    println!("{}", serde_json::to_string_pretty(&response)?);
    
    Ok(())
}