//! SDN Demo - Working example of the simplified SDN architecture
//!
//! This demonstrates the core SDN pipeline:
//! 1. Start from domain context
//! 2. Build Software Defined Network using cim-graph ContextGraph
//! 3. Generate nix-topology compliant Nix configurations

use cim_network::sdn::{SDNBuilder, SDNNode, SDNConnection, SDNInterface};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 CIM Network SDN Demo");
    println!("=======================\n");

    // 1. Initialize SDN from domain context
    println!("1. 🏗️  Initializing SDN from domain context...");
    let domain_context = serde_json::json!({
        "domain_name": "enterprise-network",
        "base_network": "10.0.0.0/8",
        "environment": "production",
        "scale": "enterprise"
    });

    let mut sdn_builder = SDNBuilder::from_domain(domain_context).await?;
    println!("   ✅ SDN initialized with ContextGraph backing\n");

    // 2. Build network nodes
    println!("2. 🖥️  Adding network nodes to SDN...");
    
    // Add a server node
    let server_node = SDNNode {
        id: "server-01".to_string(),
        node_type: "server".to_string(),
        tier: "cluster".to_string(),
        interfaces: vec![
            SDNInterface {
                name: "eth0".to_string(),
                interface_type: "ethernet".to_string(),
                addresses: vec!["10.0.1.10".to_string()],
                mtu: Some(1500),
                vlan_id: None,
            }
        ],
        services: vec!["openssh".to_string(), "networking".to_string()],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("location".to_string(), "datacenter-west".to_string());
            meta.insert("role".to_string(), "application-server".to_string());
            meta
        },
    };
    
    sdn_builder.add_node(server_node).await?;
    println!("   ✅ Added server-01 (cluster tier)");

    // Add a gateway node
    let gateway_node = SDNNode {
        id: "gateway-01".to_string(),
        node_type: "gateway".to_string(),
        tier: "leaf".to_string(),
        interfaces: vec![
            SDNInterface {
                name: "eth0".to_string(),
                interface_type: "ethernet".to_string(),
                addresses: vec!["10.0.1.1".to_string()],
                mtu: Some(1500),
                vlan_id: None,
            },
            SDNInterface {
                name: "eth1".to_string(),
                interface_type: "ethernet".to_string(),
                addresses: vec!["dhcp".to_string()],
                mtu: Some(1500),
                vlan_id: None,
            }
        ],
        services: vec!["nat".to_string(), "firewall".to_string()],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("location".to_string(), "datacenter-west".to_string());
            meta.insert("role".to_string(), "network-gateway".to_string());
            meta
        },
    };
    
    sdn_builder.add_node(gateway_node).await?;
    println!("   ✅ Added gateway-01 (leaf tier)");

    // Add a workstation node
    let workstation_node = SDNNode {
        id: "workstation-01".to_string(),
        node_type: "workstation".to_string(),
        tier: "client".to_string(),
        interfaces: vec![
            SDNInterface {
                name: "wlan0".to_string(),
                interface_type: "wireless".to_string(),
                addresses: vec!["dhcp".to_string()],
                mtu: Some(1500),
                vlan_id: Some(100),
            }
        ],
        services: vec!["networkmanager".to_string()],
        metadata: {
            let mut meta = HashMap::new();
            meta.insert("location".to_string(), "office-hq".to_string());
            meta.insert("role".to_string(), "user-device".to_string());
            meta
        },
    };
    
    sdn_builder.add_node(workstation_node).await?;
    println!("   ✅ Added workstation-01 (client tier)\n");

    // 3. Connect nodes
    println!("3. 🔗 Establishing network connections...");
    
    let ethernet_connection = SDNConnection {
        id: "server-to-gateway".to_string(),
        from_node: "server-01".to_string(),
        to_node: "gateway-01".to_string(),
        connection_type: "ethernet".to_string(),
        properties: {
            let mut props = HashMap::new();
            props.insert("bandwidth".to_string(), "1Gbps".to_string());
            props.insert("redundant".to_string(), "false".to_string());
            props
        },
    };
    
    sdn_builder.connect_nodes(ethernet_connection).await?;
    println!("   ✅ Connected server-01 → gateway-01 (ethernet)");

    let wireless_connection = SDNConnection {
        id: "workstation-to-gateway".to_string(),
        from_node: "workstation-01".to_string(),
        to_node: "gateway-01".to_string(),
        connection_type: "wireless".to_string(),
        properties: {
            let mut props = HashMap::new();
            props.insert("bandwidth".to_string(), "300Mbps".to_string());
            props.insert("encryption".to_string(), "WPA3".to_string());
            props
        },
    };
    
    sdn_builder.connect_nodes(wireless_connection).await?;
    println!("   ✅ Connected workstation-01 → gateway-01 (wireless)\n");

    // 4. Generate nix-topology compliant configuration
    println!("4. 🔧 Generating nix-topology compliant Nix configuration...");
    let nix_config = sdn_builder.generate_nix_topology().await?;
    println!("   ✅ Generated nix-topology compliant configuration");
    
    // Show a preview of the generated configuration
    let preview_lines: Vec<&str> = nix_config.lines().take(15).collect();
    println!("\n📄 Configuration Preview:");
    println!("   ┌─────────────────────────────────────────");
    for line in preview_lines {
        println!("   │ {}", line);
    }
    println!("   │ ... (truncated)");
    println!("   └─────────────────────────────────────────\n");

    // 5. Show SDN state and ContextGraph information
    println!("5. 📊 Current SDN State:");
    let sdn_state = sdn_builder.get_sdn_state();
    println!("   • Nodes: {}", sdn_state.nodes.len());
    println!("   • Connections: {}", sdn_state.connections.len());
    
    let context_graph = sdn_builder.context_graph();
    println!("   • ContextGraph Events: {}", sdn_builder.get_events().len());
    println!("   • Graph Type: {:?}", context_graph.graph_type());

    // 6. Show event audit trail
    println!("\n6. 📋 Event Audit Trail (Domain Event Sourcing):");
    for (i, event) in sdn_builder.get_events().iter().enumerate() {
        println!("   {}. Event ID: {}", i + 1, event.event_id);
        println!("      Correlation: {}", event.correlation_id);
        match &event.payload {
            cim_graph::EventPayload::Context(payload) => {
                println!("      Type: Context Event");
                match payload {
                    cim_graph::events::ContextPayload::BoundedContextCreated { name, .. } => {
                        println!("      Action: Created bounded context '{}'", name);
                    },
                    cim_graph::events::ContextPayload::EntityAdded { entity_type, .. } => {
                        println!("      Action: Added entity of type '{}'", entity_type);
                    },
                    _ => println!("      Action: Other context event"),
                }
            },
            cim_graph::EventPayload::Generic(payload) => {
                println!("      Type: {} Event", payload.event_type);
            },
            _ => println!("      Type: Other event type"),
        }
        println!();
    }

    println!("🎉 SDN Demo completed successfully!");
    println!("\nKey achievements:");
    println!("✅ Domain context → SDN construction");
    println!("✅ ContextGraph-backed state management");
    println!("✅ Event-driven architecture with audit trail");
    println!("✅ nix-topology compliant configuration generation");
    println!("✅ Multi-tier node hierarchy (client/leaf/cluster)");
    println!("✅ Multiple connection types (ethernet/wireless)");
    println!("\nThe SDN is ready for deployment! 🚀");

    Ok(())
}