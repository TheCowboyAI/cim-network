//! Nix Integration Module
//!
//! This module provides integration between the CIM Network MCP server
//! and the cim-domain-nix crate for generating production-ready NixOS configurations.

use cim_domain_nix::domains::network::{
    NetworkTopologyService, NetworkTopologyId, NodeTier, NodeType,
    NetworkInterface, IpAddress, InterfaceType
};
use cim_domain_nix::value_objects::MessageIdentity;
use crate::agents::{
    NetworkLocation, NetworkConnection, OfficeSize, CloudProvider, VPNProtocol
};
use crate::sdn::{SDNNode, SDNConnection, SDNInterface};
use crate::domain::{IpNetwork, NetworkError};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::str::FromStr;

/// Nix topology generator using cim-domain-nix
pub struct NixTopologyGenerator {
    /// The underlying network topology service
    service: NetworkTopologyService,
    /// Current topology ID being worked on
    current_topology_id: Option<NetworkTopologyId>,
}

/// Generated NixOS configuration output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedNixConfig {
    /// Configuration content in Nix language
    pub content: String,
    /// Metadata about the generation
    pub metadata: NixConfigMetadata,
}

/// Metadata for generated Nix configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixConfigMetadata {
    /// Number of nodes in the topology
    pub node_count: usize,
    /// Generation timestamp
    pub generated_at: chrono::DateTime<chrono::Utc>,
    /// Configuration format version
    pub version: String,
    /// List of generated hostnames
    pub hostnames: Vec<String>,
}

impl NixTopologyGenerator {
    /// Create a new Nix topology generator
    pub fn new() -> Self {
        Self {
            service: NetworkTopologyService::new(),
            current_topology_id: None,
        }
    }

    /// Start a new network topology
    pub async fn start_topology(&mut self, name: String) -> Result<NetworkTopologyId, NetworkError> {
        let topology_view = self.service
            .create_starlink_topology(
                name,
                "192.168.100".to_string(), // Default WAN subnet
                "192.168.1".to_string(),   // Default LAN subnet
            )
            .await
            .map_err(|e| NetworkError::General(format!("Failed to create topology: {}", e)))?;
        
        self.current_topology_id = Some(topology_view.id);
        Ok(topology_view.id)
    }

    /// Add a network location to the current topology
    pub async fn add_location(
        &mut self,
        location_id: &str,
        location: &NetworkLocation,
    ) -> Result<(), NetworkError> {
        let topology_id = self.current_topology_id
            .ok_or_else(|| NetworkError::ValidationError("No active topology".to_string()))?;

        let (node_type, tier, interfaces, services) = match location {
            NetworkLocation::DataCenter { name, region, availability_zone } => {
                let interfaces = vec![
                    NetworkInterface {
                        name: "eth0".to_string(),
                        mac_address: None,
                        interface_type: InterfaceType::Ethernet,
                        addresses: vec![IpAddress::new_dhcp()],
                        mtu: Some(1500),
                        vlan_id: None,
                        bridge_members: vec![],
                    }
                ];
                
                (
                    NodeType::Server,
                    NodeTier::SuperCluster,
                    interfaces,
                    vec!["networking".to_string(), "firewall".to_string()]
                )
            },
            
            NetworkLocation::Office { name, address, size } => {
                let tier = match size {
                    OfficeSize::Small => NodeTier::Client,
                    OfficeSize::Medium => NodeTier::Leaf,
                    OfficeSize::Large | OfficeSize::Campus => NodeTier::Cluster,
                };
                
                let interfaces = vec![
                    NetworkInterface {
                        name: "eth0".to_string(),
                        mac_address: None,
                        interface_type: InterfaceType::Ethernet,
                        addresses: vec![IpAddress::new_dhcp()],
                        mtu: Some(1500),
                        vlan_id: None,
                        bridge_members: vec![],
                    }
                ];
                
                (
                    NodeType::Workstation,
                    tier,
                    interfaces,
                    vec!["openssh".to_string()]
                )
            },
            
            NetworkLocation::CloudRegion { provider, region } => {
                let interfaces = vec![
                    NetworkInterface {
                        name: "eth0".to_string(),
                        mac_address: None,
                        interface_type: InterfaceType::Ethernet,
                        addresses: vec![IpAddress::new_dhcp()],
                        mtu: Some(1500),
                        vlan_id: None,
                        bridge_members: vec![],
                    }
                ];
                
                (
                    NodeType::Server,
                    NodeTier::Cluster,
                    interfaces,
                    vec!["cloud-init".to_string(), "docker".to_string()]
                )
            },
            
            NetworkLocation::EdgeLocation { name, latitude, longitude } => {
                let interfaces = vec![
                    NetworkInterface {
                        name: "wlan0".to_string(),
                        mac_address: None,
                        interface_type: InterfaceType::Wireless,
                        addresses: vec![IpAddress::new_dhcp()],
                        mtu: Some(1500),
                        vlan_id: None,
                        bridge_members: vec![],
                    }
                ];
                
                (
                    NodeType::Gateway,
                    NodeTier::Client,
                    interfaces,
                    vec!["hostapd".to_string(), "nat".to_string()]
                )
            },
            
            NetworkLocation::VirtualSegment { name, subnet, vlan_id } => {
                let ip_str = format!("{}", subnet);
                let parts: Vec<&str> = ip_str.split('/').collect();
                let base_ip = if parts.len() >= 1 {
                    format!("{}.1", parts[0].rsplit('.').skip(1).collect::<Vec<_>>().iter().rev().collect::<Vec<_>>().join("."))
                } else {
                    "192.168.1.1".to_string()
                };
                
                let mut interfaces = vec![
                    NetworkInterface {
                        name: "br0".to_string(),
                        mac_address: None,
                        interface_type: InterfaceType::Bridge,
                        addresses: vec![IpAddress::new_static(base_ip, 24)],
                        mtu: Some(1500),
                        vlan_id: *vlan_id,
                        bridge_members: vec!["eth0".to_string()],
                    }
                ];
                
                (
                    NodeType::Switch,
                    NodeTier::Leaf,
                    interfaces,
                    vec!["bridge".to_string(), "vlan".to_string()]
                )
            },
        };

        // Use the existing service to add a node
        let add_node_cmd = cim_domain_nix::domains::network::AddNodeToTopology {
            identity: MessageIdentity::new_root(),
            topology_id,
            name: location_id.to_string(),
            node_type,
            tier,
            interfaces,
            services,
            metadata: HashMap::new(),
        };

        self.service.command_handler
            .handle_add_node(add_node_cmd)
            .await
            .map_err(|e| NetworkError::General(format!("Failed to add node: {}", e)))?;

        Ok(())
    }

    /// Generate NixOS configurations for the current topology
    pub async fn generate_nixos_config(&self) -> Result<GeneratedNixConfig, NetworkError> {
        let topology_id = self.current_topology_id
            .ok_or_else(|| NetworkError::ValidationError("No active topology".to_string()))?;

        let configs = self.service
            .generate_nixos_configs(topology_id)
            .await
            .map_err(|e| NetworkError::General(format!("Failed to generate configs: {}", e)))?;

        // Convert the configs to a single Nix flake
        let mut content = String::new();
        
        content.push_str("{\n");
        content.push_str("  description = \"CIM Network Topology\";\n\n");
        
        content.push_str("  inputs = {\n");
        content.push_str("    nixpkgs.url = \"github:NixOS/nixpkgs/nixos-unstable\";\n");
        content.push_str("  };\n\n");
        
        content.push_str("  outputs = { self, nixpkgs }: {\n");
        content.push_str("    nixosConfigurations = {\n");

        let hostnames: Vec<String> = configs.iter().map(|c| c.hostname.clone()).collect();
        
        for config in &configs {
            content.push_str(&format!("      {} = nixpkgs.lib.nixosSystem {{\n", config.hostname));
            content.push_str(&format!("        system = \"{}\";\n", config.system));
            content.push_str("        modules = [\n");
            content.push_str("          ({ config, pkgs, ... }: {\n");
            
            // System configuration
            content.push_str(&format!("            networking.hostName = \"{}\";\n", config.hostname));
            
            for (key, value) in &config.networking {
                if key != "hostName" {
                    content.push_str(&format!("            networking.{} = \"{}\";\n", key, value));
                }
            }
            
            // Services
            for (service, service_config) in &config.services {
                content.push_str(&format!("            services.{} = {};\n", service, service_config));
            }
            
            // Packages
            if !config.packages.is_empty() {
                content.push_str("            environment.systemPackages = with pkgs; [\n");
                for package in &config.packages {
                    content.push_str(&format!("              {}\n", package));
                }
                content.push_str("            ];\n");
            }
            
            // Extra config
            if !config.extra_config.is_empty() {
                content.push_str(&format!("            {}\n", config.extra_config));
            }
            
            content.push_str("          })\n");
            content.push_str("        ];\n");
            content.push_str("      };\n");
        }
        
        content.push_str("    };\n");
        content.push_str("  };\n");
        content.push_str("}\n");

        let metadata = NixConfigMetadata {
            node_count: configs.len(),
            generated_at: chrono::Utc::now(),
            version: "1.0.0".to_string(),
            hostnames,
        };

        Ok(GeneratedNixConfig {
            content,
            metadata,
        })
    }

    /// Reset the current topology
    pub fn reset(&mut self) {
        self.service = NetworkTopologyService::new();
        self.current_topology_id = None;
    }

    /// Get the current topology ID
    pub fn current_topology_id(&self) -> Option<NetworkTopologyId> {
        self.current_topology_id
    }

    /// Add an SDN node to the topology
    pub async fn add_sdn_node(&mut self, node: &SDNNode) -> Result<(), NetworkError> {
        let topology_id = self.current_topology_id
            .unwrap_or_else(|| {
                // Start a new topology if none exists
                let new_id = NetworkTopologyId::new();
                self.current_topology_id = Some(new_id);
                new_id
            });

        // Convert SDNNode to cim-domain-nix types
        let node_type = match node.node_type.as_str() {
            "server" => NodeType::Server,
            "workstation" => NodeType::Workstation,
            "gateway" => NodeType::Gateway,
            "switch" => NodeType::Switch,
            _ => NodeType::Server, // Default fallback
        };

        let tier = match node.tier.as_str() {
            "client" => NodeTier::Client,
            "leaf" => NodeTier::Leaf,
            "cluster" => NodeTier::Cluster,
            "super-cluster" => NodeTier::SuperCluster,
            _ => NodeTier::Leaf, // Default fallback
        };

        let interfaces: Vec<NetworkInterface> = node.interfaces.iter().map(|iface| {
            let interface_type = match iface.interface_type.as_str() {
                "ethernet" => InterfaceType::Ethernet,
                "wireless" => InterfaceType::Wireless,
                "bridge" => InterfaceType::Bridge,
                _ => InterfaceType::Ethernet, // Default fallback
            };

            let addresses: Vec<IpAddress> = iface.addresses.iter().map(|addr| {
                if addr == "dhcp" {
                    IpAddress::new_dhcp()
                } else {
                    IpAddress::new_static(addr.clone(), 24) // Default /24 subnet
                }
            }).collect();

            NetworkInterface {
                name: iface.name.clone(),
                mac_address: None, // TODO: Could be extracted from metadata
                interface_type,
                addresses,
                mtu: iface.mtu,
                vlan_id: iface.vlan_id,
                bridge_members: vec![], // TODO: Could be extracted from metadata
            }
        }).collect();

        // Use the existing service to add a node
        let add_node_cmd = cim_domain_nix::domains::network::AddNodeToTopology {
            identity: MessageIdentity::new_root(),
            topology_id,
            name: node.id.clone(),
            node_type,
            tier,
            interfaces,
            services: node.services.clone(),
            metadata: node.metadata.clone(),
        };

        self.service.command_handler
            .handle_add_node(add_node_cmd)
            .await
            .map_err(|e| NetworkError::General(format!("Failed to add SDN node: {}", e)))?;

        Ok(())
    }

    /// Add an SDN connection to the topology
    pub async fn add_sdn_connection(&mut self, connection: &SDNConnection) -> Result<(), NetworkError> {
        // TODO: Implement connection handling in cim-domain-nix
        // For now, this is a placeholder since cim-domain-nix focuses on nodes
        // Connections might be represented as services or network configurations
        
        println!("SDN Connection added: {} -> {} ({})", 
                connection.from_node, connection.to_node, connection.connection_type);
        
        Ok(())
    }
}

impl Default for NixTopologyGenerator {
    fn default() -> Self {
        Self::new()
    }
}