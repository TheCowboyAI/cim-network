//! SDN Agent - Simplified network topology builder using SDN approach
//!
//! This agent focuses on the core goal:
//! 1. Start from a domain established in cim-start
//! 2. Build Software Defined Network using cim-graph ContextGraph
//! 3. Generate nix-topology compliant Nix files as projections

use crate::sdn::{SDNBuilder, SDNNode, SDNConnection, SDNInterface};
use crate::domain::NetworkError;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Simplified SDN Agent that wraps SDNBuilder for MCP integration
#[derive(Debug)]
pub struct SDNAgent {
    /// The core SDN builder
    sdn_builder: SDNBuilder,
}

/// Agent response for MCP integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SDNAgentResponse {
    /// Success status
    pub success: bool,
    /// Response message
    pub message: String,
    /// Generated Nix configuration (if applicable)
    pub nix_config: Option<String>,
    /// Current network summary
    pub network_summary: NetworkSummary,
}

/// Summary of the current network state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummary {
    /// Number of nodes in the network
    pub node_count: usize,
    /// Number of connections in the network
    pub connection_count: usize,
    /// List of node IDs
    pub node_ids: Vec<String>,
}

impl SDNAgent {
    /// Create a new SDN agent
    pub fn new() -> Self {
        Self {
            sdn_builder: SDNBuilder::new(),
        }
    }

    /// Initialize from a domain context (from cim-start)
    pub async fn from_domain(domain_context: serde_json::Value) -> Result<Self, NetworkError> {
        Ok(Self {
            sdn_builder: SDNBuilder::from_domain(domain_context).await?,
        })
    }

    /// Add a location as an SDN node
    pub async fn add_location(
        &mut self,
        location_id: &str,
        location_type: &str,
        parameters: &HashMap<String, String>,
    ) -> Result<SDNAgentResponse, NetworkError> {
        // Convert location to SDN node
        let sdn_node = self.convert_location_to_sdn_node(location_id, location_type, parameters)?;
        
        // Add to SDN
        self.sdn_builder.add_node(sdn_node).await?;

        Ok(SDNAgentResponse {
            success: true,
            message: format!("✅ Added {} '{}' to SDN", location_type, location_id),
            nix_config: None,
            network_summary: self.get_network_summary(),
        })
    }

    /// Connect two locations in the SDN
    pub async fn connect_locations(
        &mut self,
        from_location: &str,
        to_location: &str,
        connection_type: &str,
        parameters: &HashMap<String, String>,
    ) -> Result<SDNAgentResponse, NetworkError> {
        // Create SDN connection
        let connection = SDNConnection {
            id: format!("{}-to-{}", from_location, to_location),
            from_node: from_location.to_string(),
            to_node: to_location.to_string(),
            connection_type: connection_type.to_string(),
            properties: parameters.clone(),
        };

        // Add to SDN
        self.sdn_builder.connect_nodes(connection).await?;

        Ok(SDNAgentResponse {
            success: true,
            message: format!("🔗 Connected {} → {} via {}", from_location, to_location, connection_type),
            nix_config: None,
            network_summary: self.get_network_summary(),
        })
    }

    /// Generate Nix configuration from the current SDN state
    pub async fn generate_nix_config(&self) -> Result<SDNAgentResponse, NetworkError> {
        let nix_config = self.sdn_builder.generate_nix_topology().await?;

        Ok(SDNAgentResponse {
            success: true,
            message: "🔧 Generated Nix topology configuration".to_string(),
            nix_config: Some(nix_config),
            network_summary: self.get_network_summary(),
        })
    }

    /// Get the current network state
    pub fn get_network_summary(&self) -> NetworkSummary {
        let sdn_state = self.sdn_builder.get_sdn_state();
        
        NetworkSummary {
            node_count: sdn_state.nodes.len(),
            connection_count: sdn_state.connections.len(),
            node_ids: sdn_state.nodes.keys().cloned().collect(),
        }
    }

    /// Convert location parameters to SDN node
    fn convert_location_to_sdn_node(
        &self,
        location_id: &str,
        location_type: &str,
        parameters: &HashMap<String, String>,
    ) -> Result<SDNNode, NetworkError> {
        let (node_type, tier, interfaces, services) = match location_type {
            "datacenter" | "dc" => (
                "server".to_string(),
                "super-cluster".to_string(),
                vec![SDNInterface {
                    name: "eth0".to_string(),
                    interface_type: "ethernet".to_string(),
                    addresses: vec!["dhcp".to_string()],
                    mtu: Some(1500),
                    vlan_id: None,
                }],
                vec!["networking".to_string(), "firewall".to_string()],
            ),
            "office" => (
                "workstation".to_string(),
                "leaf".to_string(),
                vec![SDNInterface {
                    name: "eth0".to_string(),
                    interface_type: "ethernet".to_string(),
                    addresses: vec!["dhcp".to_string()],
                    mtu: Some(1500),
                    vlan_id: None,
                }],
                vec!["openssh".to_string()],
            ),
            "cloud" => (
                "server".to_string(),
                "cluster".to_string(),
                vec![SDNInterface {
                    name: "eth0".to_string(),
                    interface_type: "ethernet".to_string(),
                    addresses: vec!["dhcp".to_string()],
                    mtu: Some(1500),
                    vlan_id: None,
                }],
                vec!["cloud-init".to_string(), "docker".to_string()],
            ),
            "edge" => (
                "gateway".to_string(),
                "client".to_string(),
                vec![SDNInterface {
                    name: "wlan0".to_string(),
                    interface_type: "wireless".to_string(),
                    addresses: vec!["dhcp".to_string()],
                    mtu: Some(1500),
                    vlan_id: None,
                }],
                vec!["hostapd".to_string(), "nat".to_string()],
            ),
            "segment" => (
                "switch".to_string(),
                "leaf".to_string(),
                vec![SDNInterface {
                    name: "br0".to_string(),
                    interface_type: "bridge".to_string(),
                    addresses: vec!["192.168.1.1".to_string()],
                    mtu: Some(1500),
                    vlan_id: parameters.get("vlan").and_then(|v| v.parse().ok()),
                }],
                vec!["bridge".to_string(), "vlan".to_string()],
            ),
            _ => {
                return Err(NetworkError::ValidationError(
                    format!("Unknown location type: {}", location_type)
                ));
            }
        };

        Ok(SDNNode {
            id: location_id.to_string(),
            node_type,
            tier,
            interfaces,
            services,
            metadata: parameters.clone(),
        })
    }
}

impl Default for SDNAgent {
    fn default() -> Self {
        Self::new()
    }
}