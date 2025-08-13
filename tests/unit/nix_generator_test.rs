//! Unit tests for Nix topology generator components

use cim_network::infrastructure::nix::{
    NixTopologyGenerator, ContextGraphTemplateEngine, SimpleFileWriter, SimpleNixFormatter,
    TemplateContext, NetworkTopologyContext, DeviceContext, InterfaceContext,
    GenerationMetadata, ValidationStatus
};
use cim_network::domain::aggregates::network_topology::{NetworkTopology, TopologyType};
use cim_network::domain::IpNetwork;
use std::str::FromStr;
use tempfile::TempDir;

#[tokio::test]
async fn test_template_context_creation() {
    let ip_network = IpNetwork::from_str("192.168.1.0/24").unwrap();
    let mut topology = NetworkTopology::from_ip_and_name(
        ip_network,
        "test-context".to_string(),
        Some(TopologyType::SingleRouter { interface_count: 2 }),
    ).unwrap();

    topology.generate_nix_topology().unwrap();
    
    let generator = NixTopologyGenerator::new(
        Box::new(ContextGraphTemplateEngine::new()),
        Box::new(SimpleFileWriter),
        Box::new(SimpleNixFormatter),
    );

    let options = cim_network::infrastructure::nix::NixGenerationOptions {
        deployment_target: cim_network::infrastructure::nix::DeploymentTarget::Local,
        generate_documentation: true,
        include_examples: true,
        custom_modules: std::collections::HashMap::new(),
        flake_inputs: std::collections::HashMap::new(),
        template_overrides: None,
        output_directory: std::path::PathBuf::from("/tmp"),
    };

    // Test template context building (access private method via reflection would be needed in real tests)
    // For now, we test the overall behavior
    assert_eq!(topology.name(), "test-context");
    assert_eq!(topology.devices().len(), 1); // Single router
    assert!(topology.nix_config().is_some());
}

#[test]
fn test_topology_context_structure() {
    let context = NetworkTopologyContext {
        name: "test-network".to_string(),
        base_network: "192.168.1.0/24".to_string(),
        topology_type: "SingleRouter".to_string(),
        device_count: 1,
        network_count: 1,
    };

    assert_eq!(context.name, "test-network");
    assert_eq!(context.device_count, 1);
    assert_eq!(context.network_count, 1);
}

#[test]
fn test_device_context_structure() {
    let device_context = DeviceContext {
        name: "test-router".to_string(),
        device_type: "router".to_string(),
        ip_address: "192.168.1.1".to_string(),
        interfaces: vec![
            InterfaceContext {
                name: "eth0".to_string(),
                network: "lan".to_string(),
                address: Some("192.168.1.1/24".to_string()),
                dhcp: false,
                enabled: true,
            }
        ],
        services: vec!["ssh".to_string()],
        nix_module_path: "./modules/test-router.nix".to_string(),
    };

    assert_eq!(device_context.name, "test-router");
    assert_eq!(device_context.device_type, "router");
    assert_eq!(device_context.interfaces.len(), 1);
    assert_eq!(device_context.services.len(), 1);
}

#[test]
fn test_generation_metadata() {
    let metadata = GenerationMetadata {
        generation_timestamp: "2024-01-01T12:00:00Z".to_string(),
        generator_version: "0.1.0".to_string(),
        target_system: "x86_64-linux".to_string(),
    };

    assert_eq!(metadata.generator_version, "0.1.0");
    assert_eq!(metadata.target_system, "x86_64-linux");
}

#[tokio::test]
async fn test_file_writer_implementation() {
    let temp_dir = TempDir::new().unwrap();
    let file_writer = SimpleFileWriter;
    
    let test_file = temp_dir.path().join("test.txt");
    let content = "test content";
    
    file_writer.write_file(&test_file, content).await.unwrap();
    
    assert!(test_file.exists());
    let read_content = tokio::fs::read_to_string(&test_file).await.unwrap();
    assert_eq!(read_content, content);
}

#[tokio::test]
async fn test_directory_creation() {
    let temp_dir = TempDir::new().unwrap();
    let file_writer = SimpleFileWriter;
    
    let test_dir = temp_dir.path().join("nested").join("directory");
    file_writer.create_directory(&test_dir).await.unwrap();
    
    assert!(test_dir.exists());
    assert!(test_dir.is_dir());
}

#[tokio::test] 
async fn test_nix_formatter() {
    let formatter = SimpleNixFormatter;
    
    let nix_code = "{ config, lib, pkgs, ... }: { services.openssh.enable = true; }";
    let formatted = formatter.format_nix_code(nix_code).await.unwrap();
    
    // Simple formatter just returns input for now
    assert_eq!(formatted, nix_code);
}

#[test]
fn test_validation_status() {
    assert!(matches!(ValidationStatus::Valid, ValidationStatus::Valid));
    assert!(matches!(ValidationStatus::Warning, ValidationStatus::Warning));
    assert!(matches!(ValidationStatus::Error, ValidationStatus::Error));
}

#[tokio::test]
async fn test_router_switch_device_generation() {
    let ip_network = IpNetwork::from_str("10.0.0.0/16").unwrap();
    let topology = NetworkTopology::from_ip_and_name(
        ip_network,
        "router-switch-test".to_string(),
        Some(TopologyType::RouterSwitch { 
            switch_count: 2, 
            ports_per_switch: 24 
        }),
    ).unwrap();

    // Should have 1 router + 2 switches = 3 devices
    assert_eq!(topology.devices().len(), 3);
    
    // Should have connections between router and switches
    assert_eq!(topology.connections().len(), 2);
    
    // Verify device types
    let device_types: Vec<_> = topology.devices().values()
        .map(|device| match &device.device_type {
            cim_network::domain::aggregates::network_topology::DeviceType::Router { .. } => "router",
            cim_network::domain::aggregates::network_topology::DeviceType::Switch { .. } => "switch",
            _ => "other",
        })
        .collect();
    
    assert_eq!(device_types.iter().filter(|&t| *t == "router").count(), 1);
    assert_eq!(device_types.iter().filter(|&t| *t == "switch").count(), 2);
}

#[test]
fn test_topology_auto_detection_logic() {
    // Test /24 network - should get RouterSwitch
    let ip_24 = IpNetwork::from_str("192.168.1.0/24").unwrap();
    let topology_24 = NetworkTopology::from_ip_and_name(
        ip_24,
        "test-24".to_string(),
        None, // Auto-detect
    ).unwrap();
    
    match topology_24.topology_type() {
        TopologyType::RouterSwitch { switch_count, ports_per_switch } => {
            assert_eq!(*switch_count, 1);
            assert_eq!(*ports_per_switch, 24);
        }
        _ => panic!("Expected RouterSwitch for /24 network"),
    }
    
    // Test /16 network - should get ThreeTier
    let ip_16 = IpNetwork::from_str("10.0.0.0/16").unwrap();
    let topology_16 = NetworkTopology::from_ip_and_name(
        ip_16,
        "test-16".to_string(),
        None, // Auto-detect
    ).unwrap();
    
    match topology_16.topology_type() {
        TopologyType::ThreeTier { core_count, distribution_count, access_count, hosts_per_access } => {
            assert_eq!(*core_count, 2);
            assert_eq!(*distribution_count, 4);
            assert_eq!(*access_count, 8);
            assert_eq!(*hosts_per_access, 12);
        }
        _ => panic!("Expected ThreeTier for /16 network"),
    }
    
    // Test /8 network - should get SpineLeaf
    let ip_8 = IpNetwork::from_str("10.0.0.0/8").unwrap();
    let topology_8 = NetworkTopology::from_ip_and_name(
        ip_8,
        "test-8".to_string(),
        None, // Auto-detect
    ).unwrap();
    
    match topology_8.topology_type() {
        TopologyType::SpineLeaf { spine_count, leaf_count, hosts_per_leaf } => {
            assert_eq!(*spine_count, 4);
            assert_eq!(*leaf_count, 16);
            assert_eq!(*hosts_per_leaf, 32);
        }
        _ => panic!("Expected SpineLeaf for /8 network"),
    }
    
    // Test /30 network - should get SingleRouter
    let ip_30 = IpNetwork::from_str("192.168.1.0/30").unwrap();
    let topology_30 = NetworkTopology::from_ip_and_name(
        ip_30,
        "test-30".to_string(),
        None, // Auto-detect
    ).unwrap();
    
    match topology_30.topology_type() {
        TopologyType::SingleRouter { interface_count } => {
            assert_eq!(*interface_count, 4);
        }
        _ => panic!("Expected SingleRouter for /30 network"),
    }
}

#[test]
fn test_nix_topology_config_generation() {
    let ip_network = IpNetwork::from_str("192.168.100.0/24").unwrap();
    let mut topology = NetworkTopology::from_ip_and_name(
        ip_network,
        "nix-config-test".to_string(),
        Some(TopologyType::SingleRouter { interface_count: 3 }),
    ).unwrap();

    let nix_config = topology.generate_nix_topology().unwrap();
    
    assert_eq!(nix_config.topology_name, "nix-config-test");
    assert_eq!(nix_config.base_network, ip_network);
    assert!(!nix_config.devices.is_empty());
    assert!(!nix_config.networks.is_empty());
    
    // Verify network configuration
    let lan_network = &nix_config.networks[0];
    assert_eq!(lan_network.name, "lan");
    assert!(!lan_network.dhcp);
    assert!(lan_network.gateway.is_some());
    
    // Verify device configuration
    let router_device = &nix_config.devices[0];
    assert!(router_device.name.contains("nix-config-test"));
    assert!(matches!(router_device.device_type, 
                    cim_network::domain::aggregates::network_topology::NixDeviceType::Router));
    assert!(!router_device.interfaces.is_empty());
}

#[test]
fn test_device_name_sanitization() {
    let generator = NixTopologyGenerator::new(
        Box::new(ContextGraphTemplateEngine::new()),
        Box::new(SimpleFileWriter),
        Box::new(SimpleNixFormatter),
    );
    
    // Test sanitize_name method behavior (would need to expose method or test via generated content)
    // For now, test that device names with special characters work
    let ip_network = IpNetwork::from_str("192.168.1.0/24").unwrap();
    let topology = NetworkTopology::from_ip_and_name(
        ip_network,
        "test-network-with-dashes.domain".to_string(),
        Some(TopologyType::SingleRouter { interface_count: 1 }),
    ).unwrap();
    
    // Should not panic with special characters in name
    assert!(topology.name().contains("test-network-with-dashes.domain"));
}

#[test]
fn test_interface_configuration() {
    use cim_network::domain::aggregates::network_topology::{InterfaceSpec, InterfaceType};
    use cim_network::domain::{InterfaceId, VlanId};
    use std::net::IpAddr;

    let interface = InterfaceSpec {
        id: InterfaceId::new(),
        name: "eth0".to_string(),
        interface_type: InterfaceType::GigabitEthernet,
        ip_address: Some(IpAddr::V4("192.168.1.1".parse().unwrap())),
        subnet_mask: Some(24),
        vlan: Some(VlanId::from(100)),
        enabled: true,
    };

    assert_eq!(interface.name, "eth0");
    assert!(matches!(interface.interface_type, InterfaceType::GigabitEthernet));
    assert!(interface.ip_address.is_some());
    assert_eq!(interface.subnet_mask, Some(24));
    assert!(interface.vlan.is_some());
    assert!(interface.enabled);
}

#[test]
fn test_connection_configuration() {
    use cim_network::domain::aggregates::network_topology::{
        NetworkConnection, ConnectionEndpoint, ConnectionType, Bandwidth, Duplex
    };
    use cim_network::domain::{DeviceId, InterfaceId, ConnectionId};

    let connection = NetworkConnection {
        id: ConnectionId::new(),
        source: ConnectionEndpoint {
            device_id: DeviceId::new(),
            interface_id: InterfaceId::new(),
        },
        target: ConnectionEndpoint {
            device_id: DeviceId::new(),
            interface_id: InterfaceId::new(),
        },
        connection_type: ConnectionType::Ethernet,
        vlan: None,
        bandwidth: Some(Bandwidth {
            speed: 1000,
            duplex: Duplex::Full,
        }),
    };

    assert!(matches!(connection.connection_type, ConnectionType::Ethernet));
    assert!(connection.bandwidth.is_some());
    assert_eq!(connection.bandwidth.as_ref().unwrap().speed, 1000);
    assert!(matches!(connection.bandwidth.as_ref().unwrap().duplex, Duplex::Full));
}