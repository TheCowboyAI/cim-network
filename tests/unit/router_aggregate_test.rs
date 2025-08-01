//! Tests for router configuration aggregate - WRITTEN FIRST (TDD)

use cim_network::domain::aggregates::router_configuration::*;
use cim_network::domain::{RouterId, CorrelationId, CausationId, IpNetwork};
use cim_network::domain::events::{RouterVendor, CiscoOs};
use std::str::FromStr;

#[test]
fn test_router_configuration_creation() {
    let router_id = RouterId::new();
    let config = RouterConfiguration::new(
        router_id,
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    assert_eq!(config.id(), router_id);
    assert_eq!(config.name(), "edge-router-01");
    assert_eq!(config.version(), 0);
    assert!(config.interfaces().is_empty());
    assert!(config.routing_protocols().is_empty());
}

#[test]
fn test_add_interface_to_router() {
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
    let interface = InterfaceConfig {
        name: "GigabitEthernet0/0".to_string(),
        description: Some("WAN uplink".to_string()),
        ip_address: Some(IpNetwork::from_str("192.168.1.1/24").unwrap()),
        enabled: true,
        vlan: None,
    };
    
    let result = config.add_interface(interface.clone(), correlation_id.clone(), causation_id.clone());
    assert!(result.is_ok());
    
    let event = result.unwrap();
    assert_eq!(config.interfaces().len(), 1);
    assert_eq!(config.interfaces()[0].name, "GigabitEthernet0/0");
    
    // Verify event
    match event {
        cim_network::NetworkEvent::RouterInterfaceAdded { metadata, router_id: _, interface: added_interface } => {
            assert_eq!(metadata.correlation_id, correlation_id);
            assert_eq!(metadata.causation_id, causation_id);
            assert_eq!(added_interface.name, interface.name);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_duplicate_interface_rejected() {
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let interface = InterfaceConfig {
        name: "GigabitEthernet0/0".to_string(),
        description: None,
        ip_address: Some(IpNetwork::from_str("192.168.1.1/24").unwrap()),
        enabled: true,
        vlan: None,
    };
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
    // Add first time - should succeed
    assert!(config.add_interface(interface.clone(), correlation_id.clone(), causation_id.clone()).is_ok());
    
    // Add second time - should fail
    let result = config.add_interface(interface, correlation_id, causation_id);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Network error: Interface GigabitEthernet0/0 already exists");
}

#[test]
fn test_configure_ospf() {
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
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
            }
        ],
    };
    
    let result = config.configure_ospf(ospf, correlation_id.clone(), causation_id.clone());
    assert!(result.is_ok());
    
    let event = result.unwrap();
    assert_eq!(config.routing_protocols().len(), 1);
    
    // Verify event
    match event {
        cim_network::NetworkEvent::RouterOspfConfigured { metadata, .. } => {
            assert_eq!(metadata.correlation_id, correlation_id);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_configure_bgp() {
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
    let bgp = BgpConfig {
        local_as: 65001,
        router_id: "1.1.1.1".parse().unwrap(),
        neighbors: vec![
            BgpNeighborConfig {
                address: "192.168.1.2".parse().unwrap(),
                remote_as: 65002,
                description: Some("ISP uplink".to_string()),
                password: None,
            }
        ],
    };
    
    let result = config.configure_bgp(bgp, correlation_id.clone(), causation_id.clone());
    assert!(result.is_ok());
    
    assert_eq!(config.routing_protocols().len(), 1);
}

#[test]
fn test_add_access_list() {
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
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
                action: AclAction::Deny,
                protocol: Protocol::Ip,
                source: IpNetwork::from_str("172.16.0.0/12").unwrap(),
                destination: IpNetwork::from_str("0.0.0.0/0").unwrap(),
                ports: None,
            },
            AclEntryConfig {
                sequence: 30,
                action: AclAction::Deny,
                protocol: Protocol::Ip,
                source: IpNetwork::from_str("192.168.0.0/16").unwrap(),
                destination: IpNetwork::from_str("0.0.0.0/0").unwrap(),
                ports: None,
            },
            AclEntryConfig {
                sequence: 1000,
                action: AclAction::Permit,
                protocol: Protocol::Ip,
                source: IpNetwork::from_str("0.0.0.0/0").unwrap(),
                destination: IpNetwork::from_str("0.0.0.0/0").unwrap(),
                ports: None,
            },
        ],
    };
    
    let result = config.add_access_list(acl, correlation_id, causation_id);
    assert!(result.is_ok());
    
    assert_eq!(config.access_lists().len(), 1);
    assert_eq!(config.access_lists()[0].entries.len(), 4);
}

#[test]
fn test_router_configuration_snapshot() {
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    // Add some configuration
    let interface = InterfaceConfig {
        name: "GigabitEthernet0/0".to_string(),
        description: Some("WAN".to_string()),
        ip_address: Some(IpNetwork::from_str("192.168.1.1/24").unwrap()),
        enabled: true,
        vlan: None,
    };
    
    config.add_interface(interface, CorrelationId::new(), CausationId::new()).unwrap();
    
    // Get snapshot
    let snapshot = config.snapshot();
    assert_eq!(snapshot.interfaces.len(), 1);
    assert_eq!(snapshot.interfaces[0].name, "GigabitEthernet0/0");
}

#[test]
fn test_ip_address_conflict_detection() {
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let interface1 = InterfaceConfig {
        name: "GigabitEthernet0/0".to_string(),
        description: None,
        ip_address: Some(IpNetwork::from_str("192.168.1.1/24").unwrap()),
        enabled: true,
        vlan: None,
    };
    
    let interface2 = InterfaceConfig {
        name: "GigabitEthernet0/1".to_string(),
        description: None,
        ip_address: Some(IpNetwork::from_str("192.168.1.2/24").unwrap()), // Same subnet!
        enabled: true,
        vlan: None,
    };
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
    // First interface should succeed
    assert!(config.add_interface(interface1, correlation_id.clone(), causation_id.clone()).is_ok());
    
    // Second interface should fail due to IP conflict
    let result = config.add_interface(interface2, correlation_id, causation_id);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("overlaps"));
}

#[test]
fn test_router_configuration_validation() {
    let config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    // Empty configuration should be valid but will warn
    let validation = config.validate();
    assert!(validation.is_valid());
    assert!(!validation.warnings.is_empty());
    assert!(validation.warnings.iter().any(|w| w.contains("No interfaces")));
}

#[test]
fn test_apply_configuration_template() {
    let mut config = RouterConfiguration::new(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let template = RouterTemplate::edge_router(
        vec![
            IpNetwork::from_str("10.0.0.0/24").unwrap(),
            IpNetwork::from_str("10.0.1.0/24").unwrap(),
        ],
        "192.168.1.1".parse().unwrap(),
    );
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
    let events = config.apply_template(template, correlation_id, causation_id);
    assert!(events.is_ok());
    
    let event_list = events.unwrap();
    assert!(event_list.len() > 0);
    
    // Should have interfaces and routing configured
    assert!(!config.interfaces().is_empty());
    assert!(!config.routing_protocols().is_empty());
}