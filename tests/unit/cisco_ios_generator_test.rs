//! Tests for Cisco IOS configuration generator - WRITTEN FIRST (TDD)

use cim_network::domain::configuration::cisco_ios::{CiscoIosGenerator, CiscoGeneratorOptions};
use cim_network::domain::configuration::ConfigurationGenerator;
use cim_network::domain::aggregates::router_configuration::*;
use cim_network::domain::{RouterId, IpNetwork, CorrelationId, CausationId};
use cim_network::domain::events::{RouterVendor, CiscoOs};
use std::str::FromStr;

#[test]
fn test_generate_basic_configuration() {
    let generator = CiscoIosGenerator::new();
    
    let config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let output = generator.generate(&config).unwrap();
    
    // Should have basic configuration elements
    assert!(output.contains("hostname edge-router-01"));
    assert!(output.contains("service timestamps debug datetime msec"));
    assert!(output.contains("service timestamps log datetime msec"));
    assert!(output.contains("no ip domain lookup"));
    assert!(output.contains("ip cef"));
}

#[test]
fn test_generate_interface_configuration() {
    let generator = CiscoIosGenerator::new();
    
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    // Add interfaces
    let interface1 = InterfaceConfig {
        name: "GigabitEthernet0/0".to_string(),
        description: Some("WAN Uplink".to_string()),
        ip_address: Some(IpNetwork::from_str("192.168.1.1/24").unwrap()),
        enabled: true,
        vlan: None,
    };
    
    let interface2 = InterfaceConfig {
        name: "GigabitEthernet0/1".to_string(),
        description: Some("LAN Network".to_string()),
        ip_address: Some(IpNetwork::from_str("10.0.0.1/24").unwrap()),
        enabled: false,
        vlan: None,
    };
    
    config.add_interface(interface1, CorrelationId::new(), CausationId::new()).unwrap();
    config.add_interface(interface2, CorrelationId::new(), CausationId::new()).unwrap();
    
    let output = generator.generate(&config).unwrap();
    
    // Check interface configuration
    assert!(output.contains("interface GigabitEthernet0/0"));
    assert!(output.contains(" description WAN Uplink"));
    assert!(output.contains(" ip address 192.168.1.1 255.255.255.0"));
    assert!(output.contains(" no shutdown"));
    
    assert!(output.contains("interface GigabitEthernet0/1"));
    assert!(output.contains(" description LAN Network"));
    assert!(output.contains(" ip address 10.0.0.1 255.255.255.0"));
    assert!(output.contains(" shutdown")); // Interface is disabled
}

#[test]
fn test_generate_ospf_configuration() {
    let generator = CiscoIosGenerator::new();
    
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "core-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    // Configure OSPF
    let ospf = OspfConfig {
        process_id: 1,
        router_id: "1.1.1.1".parse().unwrap(),
        areas: vec![
            OspfAreaConfig {
                area_id: 0,
                area_type: AreaType::Backbone,
                networks: vec![
                    IpNetwork::from_str("10.0.0.0/24").unwrap(),
                    IpNetwork::from_str("10.0.1.0/24").unwrap(),
                ],
            },
            OspfAreaConfig {
                area_id: 10,
                area_type: AreaType::Standard,
                networks: vec![
                    IpNetwork::from_str("172.16.0.0/24").unwrap(),
                ],
            },
        ],
    };
    
    config.configure_ospf(ospf, CorrelationId::new(), CausationId::new()).unwrap();
    
    let output = generator.generate(&config).unwrap();
    
    // Check OSPF configuration
    assert!(output.contains("router ospf 1"));
    assert!(output.contains(" router-id 1.1.1.1"));
    assert!(output.contains(" network 10.0.0.0 0.0.0.255 area 0"));
    assert!(output.contains(" network 10.0.1.0 0.0.0.255 area 0"));
    assert!(output.contains(" network 172.16.0.0 0.0.0.255 area 10"));
}

#[test]
fn test_generate_bgp_configuration() {
    let generator = CiscoIosGenerator::new();
    
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    // Configure BGP
    let bgp = BgpConfig {
        local_as: 65001,
        router_id: "1.1.1.1".parse().unwrap(),
        neighbors: vec![
            BgpNeighborConfig {
                address: "192.168.1.2".parse().unwrap(),
                remote_as: 65002,
                description: Some("ISP Uplink".to_string()),
                password: Some("secretpass123".to_string()),
            },
            BgpNeighborConfig {
                address: "10.0.0.2".parse().unwrap(),
                remote_as: 65001, // iBGP peer
                description: Some("Internal Peer".to_string()),
                password: None,
            },
        ],
    };
    
    config.configure_bgp(bgp, CorrelationId::new(), CausationId::new()).unwrap();
    
    let output = generator.generate(&config).unwrap();
    
    // Check BGP configuration
    assert!(output.contains("router bgp 65001"));
    assert!(output.contains(" bgp router-id 1.1.1.1"));
    assert!(output.contains(" neighbor 192.168.1.2 remote-as 65002"));
    assert!(output.contains(" neighbor 192.168.1.2 description ISP Uplink"));
    assert!(output.contains(" neighbor 192.168.1.2 password secretpass123"));
    assert!(output.contains(" neighbor 10.0.0.2 remote-as 65001"));
    assert!(output.contains(" neighbor 10.0.0.2 description Internal Peer"));
}

#[test]
fn test_generate_access_list_configuration() {
    let generator = CiscoIosGenerator::new();
    
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    // Add access list
    let acl = AccessListConfig {
        number: 100,
        name: Some("DENY_RFC1918".to_string()),
        entries: vec![
            AclEntryConfig {
                sequence: 10,
                action: AclAction::Deny,
                protocol: Protocol::Ip,
                source: IpNetwork::from_str("10.0.0.0/8").unwrap(),
                destination: IpNetwork::from_str("0.0.0.0/0").unwrap(),
                ports: None,
            },
            AclEntryConfig {
                sequence: 20,
                action: AclAction::Permit,
                protocol: Protocol::Tcp,
                source: IpNetwork::from_str("192.168.1.0/24").unwrap(),
                destination: IpNetwork::from_str("0.0.0.0/0").unwrap(),
                ports: Some(PortRange { start: 80, end: 80 }),
            },
        ],
    };
    
    config.add_access_list(acl, CorrelationId::new(), CausationId::new()).unwrap();
    
    let output = generator.generate(&config).unwrap();
    
    // Check access list configuration
    assert!(output.contains("ip access-list extended 100"));
    assert!(output.contains(" remark DENY_RFC1918"));
    assert!(output.contains(" 10 deny ip 10.0.0.0 0.255.255.255 any"));
    assert!(output.contains(" 20 permit tcp 192.168.1.0 0.0.0.255 any eq 80"));
}

#[test]
fn test_generate_vlan_interface_configuration() {
    let generator = CiscoIosGenerator::new();
    
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "core-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    // Add VLAN interface
    let vlan_interface = InterfaceConfig {
        name: "Vlan100".to_string(),
        description: Some("Management VLAN".to_string()),
        ip_address: Some(IpNetwork::from_str("10.100.0.1/24").unwrap()),
        enabled: true,
        vlan: Some(100),
    };
    
    config.add_interface(vlan_interface, CorrelationId::new(), CausationId::new()).unwrap();
    
    let output = generator.generate(&config).unwrap();
    
    // Check VLAN interface configuration
    assert!(output.contains("interface Vlan100"));
    assert!(output.contains(" description Management VLAN"));
    assert!(output.contains(" ip address 10.100.0.1 255.255.255.0"));
    assert!(output.contains(" no shutdown"));
}

#[test]
fn test_configuration_sections_order() {
    let generator = CiscoIosGenerator::new();
    
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "test-router".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    // Add various configurations
    config.add_interface(InterfaceConfig {
        name: "GigabitEthernet0/0".to_string(),
        description: None,
        ip_address: Some(IpNetwork::from_str("192.168.1.1/24").unwrap()),
        enabled: true,
        vlan: None,
    }, CorrelationId::new(), CausationId::new()).unwrap();
    
    config.configure_ospf(OspfConfig {
        process_id: 1,
        router_id: "1.1.1.1".parse().unwrap(),
        areas: vec![],
    }, CorrelationId::new(), CausationId::new()).unwrap();
    
    let output = generator.generate(&config).unwrap();
    
    // Check that sections appear in correct order
    let hostname_pos = output.find("hostname").unwrap();
    let interface_pos = output.find("interface").unwrap();
    let ospf_pos = output.find("router ospf").unwrap();
    
    assert!(hostname_pos < interface_pos);
    assert!(interface_pos < ospf_pos);
}

#[test]
fn test_validate_generated_configuration() {
    let generator = CiscoIosGenerator::new();
    
    let config = RouterConfiguration::new(
        RouterId::new(),
        "test-router".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let output = generator.generate(&config).unwrap();
    
    // Validate the configuration
    let validation_result = generator.validate(&output);
    assert!(validation_result.is_ok());
    
    // Check for basic syntax
    assert!(!output.contains("!!")); // No double exclamation marks
    assert!(output.ends_with("end\n")); // Proper ending
}

#[test]
fn test_generate_with_ios_version_differences() {
    let generator = CiscoIosGenerator::new();
    
    let config_ios15 = RouterConfiguration::new(
        RouterId::new(),
        "router-ios15".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let config_iosxe = RouterConfiguration::new(
        RouterId::new(),
        "router-iosxe".to_string(),
        RouterVendor::Cisco { os: CiscoOs::IosXe17_3 },
    );
    
    let output_ios15 = generator.generate(&config_ios15).unwrap();
    let output_iosxe = generator.generate(&config_iosxe).unwrap();
    
    // IOS XE should have some additional features
    assert!(output_iosxe.contains("ip nbar protocol-discovery"));
}

#[test]
fn test_sanitize_sensitive_information() {
    let generator = CiscoIosGenerator::with_options(CiscoGeneratorOptions {
        sanitize_passwords: true,
        include_timestamps: false,
    });
    
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "secure-router".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    // Add BGP with password
    let bgp = BgpConfig {
        local_as: 65001,
        router_id: "1.1.1.1".parse().unwrap(),
        neighbors: vec![
            BgpNeighborConfig {
                address: "192.168.1.2".parse().unwrap(),
                remote_as: 65002,
                description: None,
                password: Some("supersecret123".to_string()),
            },
        ],
    };
    
    config.configure_bgp(bgp, CorrelationId::new(), CausationId::new()).unwrap();
    
    let output = generator.generate(&config).unwrap();
    
    // Password should be sanitized
    assert!(!output.contains("supersecret123"));
    assert!(output.contains("neighbor 192.168.1.2 password <REDACTED>"));
}