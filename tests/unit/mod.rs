//! Unit tests module

mod events_test;
mod state_machine_test;

// Tests for value objects - WRITTEN FIRST (TDD)

use cim_network::domain::value_objects::*;

#[test]
fn test_router_id_generation() {
    let id1 = RouterId::new();
    let id2 = RouterId::new();
    
    // IDs should be unique
    assert_ne!(id1, id2);
    
    // IDs should be non-empty when converted to string
    assert!(!id1.to_string().is_empty());
}

#[test]
fn test_vlan_id_validation() {
    // Valid VLAN IDs
    assert!(VlanId::try_new(1).is_ok());
    assert!(VlanId::try_new(100).is_ok());
    assert!(VlanId::try_new(4094).is_ok());
    
    // Invalid VLAN IDs (reserved)
    assert!(VlanId::try_new(0).is_err());
    assert!(VlanId::try_new(4095).is_err());
    assert!(VlanId::try_new(5000).is_err());
}

#[test]
fn test_mac_address_parsing() {
    // Valid MAC addresses
    assert!(MacAddress::from_str("00:11:22:33:44:55").is_ok());
    assert!(MacAddress::from_str("00-11-22-33-44-55").is_ok());
    assert!(MacAddress::from_str("001122334455").is_ok());
    
    // Invalid MAC addresses
    assert!(MacAddress::from_str("00:11:22:33:44").is_err());
    assert!(MacAddress::from_str("00:11:22:33:44:GG").is_err());
    assert!(MacAddress::from_str("").is_err());
}

#[test]
fn test_correlation_id_mandatory() {
    let id = CorrelationId::new();
    
    // Should have a value
    assert!(!id.to_string().is_empty());
    
    // Should be able to create from existing
    let id2 = CorrelationId::from(id.to_string());
    assert_eq!(id, id2);
}

#[test]
fn test_ip_network_validation() {
    use std::str::FromStr;
    
    // Valid networks
    assert!(IpNetwork::from_str("192.168.1.0/24").is_ok());
    assert!(IpNetwork::from_str("10.0.0.0/8").is_ok());
    assert!(IpNetwork::from_str("2001:db8::/32").is_ok());
    
    // Invalid networks
    assert!(IpNetwork::from_str("192.168.1.0/33").is_err()); // Invalid prefix
    assert!(IpNetwork::from_str("256.0.0.0/8").is_err());    // Invalid IP
    assert!(IpNetwork::from_str("not-an-ip").is_err());
}

#[test]
fn test_port_number_validation() {
    // Valid port numbers
    assert!(PortNumber::try_new(1).is_ok());
    assert!(PortNumber::try_new(24).is_ok());
    assert!(PortNumber::try_new(48).is_ok());
    
    // Invalid port numbers
    assert!(PortNumber::try_new(0).is_err());
    assert!(PortNumber::try_new(65536).is_err());
}

#[test]
fn test_switch_id_generation() {
    let id = SwitchId::new();
    assert!(!id.to_string().is_empty());
}

#[test]
fn test_container_network_id_generation() {
    let id = ContainerNetworkId::new();
    assert!(!id.to_string().is_empty());
}