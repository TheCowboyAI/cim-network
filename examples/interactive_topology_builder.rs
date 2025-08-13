//! Interactive Network Topology Builder Example
//!
//! This example demonstrates the event-driven network topology builder sub-agent
//! that builds context graphs interactively.

use cim_network::agents::{NetworkTopologyBuilderAgent, TopologyCommand, NetworkLocation, NetworkConnection};
use cim_network::agents::{OfficeSize, CloudProvider, VPNProtocol, ConfigurationFormat};
use cim_network::domain::IpNetwork;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Interactive Network Topology Builder Demo");
    println!("============================================");
    println!();

    // Create a new topology builder agent
    let mut agent = NetworkTopologyBuilderAgent::new();
    println!("✅ Created network topology builder agent");
    println!("   Session ID: {}", agent.session_id());
    println!();

    // Example 1: Add a data center
    let response = agent.process_command(TopologyCommand::AddLocation {
        location_id: "dc-west".to_string(),
        location: NetworkLocation::DataCenter {
            name: "West Coast DC".to_string(),
            region: "us-west-1".to_string(),
            availability_zone: Some("us-west-1a".to_string()),
        },
    }).await?;

    println!("📍 Added data center:");
    println!("   {}", response.message);
    println!("   Events generated: {}", response.events.len());
    println!();

    // Example 2: Add an office
    let response = agent.process_command(TopologyCommand::AddLocation {
        location_id: "hq-office".to_string(),
        location: NetworkLocation::Office {
            name: "Corporate HQ".to_string(),
            address: "123 Innovation Drive, Tech City".to_string(),
            size: OfficeSize::Large,
        },
    }).await?;

    println!("🏢 Added office:");
    println!("   {}", response.message);
    println!();

    // Example 3: Add a cloud region
    let response = agent.process_command(TopologyCommand::AddLocation {
        location_id: "aws-east".to_string(),
        location: NetworkLocation::CloudRegion {
            provider: CloudProvider::AWS,
            region: "us-east-1".to_string(),
        },
    }).await?;

    println!("☁️ Added cloud region:");
    println!("   {}", response.message);
    println!();

    // Example 4: Add a virtual segment
    let response = agent.process_command(TopologyCommand::AddLocation {
        location_id: "dmz-segment".to_string(),
        location: NetworkLocation::VirtualSegment {
            name: "DMZ Network".to_string(),
            subnet: IpNetwork::from_str("10.0.100.0/24")?,
            vlan_id: Some(100),
        },
    }).await?;

    println!("🔧 Added virtual segment:");
    println!("   {}", response.message);
    println!();

    // Example 5: Connect data center to office with fiber
    let response = agent.process_command(TopologyCommand::ConnectLocations {
        from: "dc-west".to_string(),
        to: "hq-office".to_string(),
        connection: NetworkConnection::Fiber {
            bandwidth: "10Gbps".to_string(),
            redundant: true,
        },
    }).await?;

    println!("🔗 Connected DC to office:");
    println!("   {}", response.message);
    println!();

    // Example 6: Connect office to cloud via VPN
    let response = agent.process_command(TopologyCommand::ConnectLocations {
        from: "hq-office".to_string(),
        to: "aws-east".to_string(),
        connection: NetworkConnection::VPN {
            protocol: VPNProtocol::WireGuard,
            encrypted: true,
        },
    }).await?;

    println!("🔒 Connected office to cloud:");
    println!("   {}", response.message);
    println!();

    // Example 7: Connect cloud to data center via direct connect
    let response = agent.process_command(TopologyCommand::ConnectLocations {
        from: "aws-east".to_string(),
        to: "dc-west".to_string(),
        connection: NetworkConnection::DirectConnect {
            provider: CloudProvider::AWS,
            bandwidth: "1Gbps".to_string(),
        },
    }).await?;

    println!("⚡ Connected cloud to DC:");
    println!("   {}", response.message);
    println!();

    // Example 8: List the current topology
    let response = agent.process_command(TopologyCommand::ListTopology).await?;
    println!("📋 Current topology:");
    println!("   Locations: {}", response.topology_summary.location_count);
    println!("   Connections: {}", response.topology_summary.connection_count);
    println!();

    // Example 9: Validate the topology
    let response = agent.process_command(TopologyCommand::ValidateTopology).await?;
    println!("✅ Topology validation:");
    println!("   {}", response.message);
    println!();

    // Example 10: Generate NixOS configuration
    let response = agent.process_command(TopologyCommand::GenerateConfiguration {
        format: ConfigurationFormat::NixOS,
    }).await?;

    println!("🔧 Generated NixOS configuration:");
    println!("{}", response.message);
    println!();

    // Example 11: Generate JSON representation
    let response = agent.process_command(TopologyCommand::GenerateConfiguration {
        format: ConfigurationFormat::JSON,
    }).await?;

    println!("📄 Generated JSON configuration:");
    println!("{}", response.message);
    println!();

    // Example 12: Complete the topology
    let response = agent.process_command(TopologyCommand::Complete).await?;
    println!("🎉 Completed topology:");
    println!("   {}", response.message);
    println!();

    // Show final statistics
    println!("📊 Final Statistics:");
    println!("   Total events generated: {}", agent.get_events().len());
    println!("   Session state: {:?}", agent.interaction_state());
    
    println!();
    println!("🧠 Key Concept: Event-Driven Context Graph");
    println!("===========================================");
    println!("Every action in this demo generated events that build a context graph:");
    println!("• Nodes represent network locations (data centers, offices, cloud regions, segments)");
    println!("• Edges represent connections (fiber, VPN, internet, direct connect, virtual)");
    println!("• The entire topology is reconstructable from the event stream");
    println!("• Configuration can be generated from the context graph structure");
    println!("• The sub-agent maintains conversation state and suggests next actions");

    Ok(())
}