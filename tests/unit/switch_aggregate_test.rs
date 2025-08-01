//! Tests for switch configuration aggregate - WRITTEN FIRST (TDD)

use cim_network::domain::aggregates::switch_configuration::*;
use cim_network::domain::{SwitchId, VlanId, PortNumber, MacAddress, CorrelationId, CausationId};
use cim_network::domain::value_objects::PortSpeed;
use std::str::FromStr;

#[test]
fn test_switch_configuration_creation() {
    let switch_id = SwitchId::new();
    let config = SwitchConfiguration::new(
        switch_id,
        "core-switch-01".to_string(),
        SwitchModel::Cisco2960X,
    );
    
    assert_eq!(config.id(), switch_id);
    assert_eq!(config.name(), "core-switch-01");
    assert_eq!(config.model(), &SwitchModel::Cisco2960X);
    assert_eq!(config.version(), 0);
    assert!(config.vlans().is_empty());
    assert!(config.ports().is_empty());
}

#[test]
fn test_create_vlan() {
    let mut config = SwitchConfiguration::new(
        SwitchId::new(),
        "switch-01".to_string(),
        SwitchModel::Cisco3850,
    );
    
    // Create VLAN
    let vlan_id = VlanId::try_new(100).unwrap();
    let events = config.create_vlan(
        vlan_id,
        "Engineering".to_string(),
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    assert_eq!(events.len(), 1);
    assert_eq!(config.vlans().len(), 1);
    
    let vlan = config.get_vlan(&vlan_id).unwrap();
    assert_eq!(vlan.name, "Engineering");
    assert!(vlan.tagged_ports.is_empty());
    assert!(vlan.untagged_ports.is_empty());
}

#[test]
fn test_duplicate_vlan_rejected() {
    let mut config = SwitchConfiguration::new(
        SwitchId::new(),
        "switch-01".to_string(),
        SwitchModel::Cisco2960X,
    );
    
    let vlan_id = VlanId::try_new(100).unwrap();
    
    // Create VLAN
    config.create_vlan(
        vlan_id,
        "VLAN100".to_string(),
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    // Try to create duplicate
    let result = config.create_vlan(
        vlan_id,
        "Duplicate".to_string(),
        CorrelationId::new(),
        CausationId::new(),
    );
    
    assert!(result.is_err());
}

#[test]
fn test_configure_port() {
    let mut config = SwitchConfiguration::new(
        SwitchId::new(),
        "switch-01".to_string(),
        SwitchModel::Cisco3850,
    );
    
    let port_num = PortNumber::try_new(1).unwrap();
    let port_config = PortConfig {
        number: port_num,
        description: Some("Uplink to Router".to_string()),
        mode: PortMode::Trunk,
        speed: PortSpeed::Gigabit,
        enabled: true,
        allowed_vlans: vec![],
    };
    
    let events = config.configure_port(
        port_config,
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    assert_eq!(events.len(), 1);
    assert_eq!(config.ports().len(), 1);
    
    let port = config.get_port(&port_num).unwrap();
    assert_eq!(port.mode, PortMode::Trunk);
    assert_eq!(port.speed, PortSpeed::Gigabit);
}

#[test]
fn test_assign_vlan_to_port() {
    let mut config = SwitchConfiguration::new(
        SwitchId::new(),
        "switch-01".to_string(),
        SwitchModel::Cisco2960X,
    );
    
    // Create VLAN
    let vlan_id = VlanId::try_new(100).unwrap();
    config.create_vlan(
        vlan_id,
        "Users".to_string(),
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    // Configure access port
    let port_num = PortNumber::try_new(5).unwrap();
    config.configure_port(
        PortConfig {
            number: port_num,
            description: Some("User Access".to_string()),
            mode: PortMode::Access,
            speed: PortSpeed::Gigabit,
            enabled: true,
            allowed_vlans: vec![],
        },
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    // Assign VLAN to port
    let events = config.assign_vlan_to_port(
        port_num,
        vlan_id,
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    assert_eq!(events.len(), 1);
    
    // Check port has VLAN
    let port = config.get_port(&port_num).unwrap();
    assert_eq!(port.allowed_vlans, vec![vlan_id]);
    
    // Check VLAN has port
    let vlan = config.get_vlan(&vlan_id).unwrap();
    assert!(vlan.untagged_ports.contains(&port_num));
}

#[test]
fn test_trunk_port_with_multiple_vlans() {
    let mut config = SwitchConfiguration::new(
        SwitchId::new(),
        "switch-01".to_string(),
        SwitchModel::Cisco3850,
    );
    
    // Create multiple VLANs
    let vlan10 = VlanId::try_new(10).unwrap();
    let vlan20 = VlanId::try_new(20).unwrap();
    let vlan30 = VlanId::try_new(30).unwrap();
    
    config.create_vlan(vlan10, "Management".to_string(), CorrelationId::new(), CausationId::new()).unwrap();
    config.create_vlan(vlan20, "Users".to_string(), CorrelationId::new(), CausationId::new()).unwrap();
    config.create_vlan(vlan30, "Servers".to_string(), CorrelationId::new(), CausationId::new()).unwrap();
    
    // Configure trunk port
    let port_num = PortNumber::try_new(24).unwrap();
    config.configure_port(
        PortConfig {
            number: port_num,
            description: Some("Trunk to Core".to_string()),
            mode: PortMode::Trunk,
            speed: PortSpeed::TenGigabit,
            enabled: true,
            allowed_vlans: vec![vlan10, vlan20, vlan30],
        },
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    // Check port configuration
    let port = config.get_port(&port_num).unwrap();
    assert_eq!(port.allowed_vlans.len(), 3);
    assert!(port.allowed_vlans.contains(&vlan10));
    assert!(port.allowed_vlans.contains(&vlan20));
    assert!(port.allowed_vlans.contains(&vlan30));
    
    // Check VLANs have the trunk port
    assert!(config.get_vlan(&vlan10).unwrap().tagged_ports.contains(&port_num));
    assert!(config.get_vlan(&vlan20).unwrap().tagged_ports.contains(&port_num));
    assert!(config.get_vlan(&vlan30).unwrap().tagged_ports.contains(&port_num));
}

#[test]
fn test_spanning_tree_configuration() {
    let mut config = SwitchConfiguration::new(
        SwitchId::new(),
        "switch-01".to_string(),
        SwitchModel::Cisco3850,
    );
    
    // Configure spanning tree
    let stp_config = SpanningTreeConfig {
        mode: StpMode::RapidPvst,
        priority: Some(4096), // Make this switch root
        root_guard_ports: vec![PortNumber::try_new(1).unwrap()],
        portfast_ports: vec![PortNumber::try_new(5).unwrap(), PortNumber::try_new(6).unwrap()],
    };
    
    let events = config.configure_spanning_tree(
        stp_config.clone(),
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    assert_eq!(events.len(), 1);
    assert_eq!(config.spanning_tree_config(), Some(&stp_config));
}

#[test]
fn test_mac_address_table() {
    let mut config = SwitchConfiguration::new(
        SwitchId::new(),
        "switch-01".to_string(),
        SwitchModel::Cisco2960X,
    );
    
    // Add MAC address entries
    let mac1 = MacAddress::from_str("00:11:22:33:44:55").unwrap();
    let mac2 = MacAddress::from_str("AA:BB:CC:DD:EE:FF").unwrap();
    let port1 = PortNumber::try_new(5).unwrap();
    let port2 = PortNumber::try_new(10).unwrap();
    let vlan = VlanId::try_new(100).unwrap();
    
    config.add_mac_address_entry(
        mac1,
        port1,
        vlan,
        MacAddressType::Dynamic,
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    config.add_mac_address_entry(
        mac2,
        port2,
        vlan,
        MacAddressType::Static,
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    // Check MAC address table
    assert_eq!(config.mac_address_table().len(), 2);
    
    let entry1 = config.lookup_mac_address(&mac1).unwrap();
    assert_eq!(entry1.port, port1);
    assert_eq!(entry1.vlan, vlan);
    assert_eq!(entry1.mac_type, MacAddressType::Dynamic);
}

#[test]
fn test_switch_stack_configuration() {
    let mut config = SwitchConfiguration::new(
        SwitchId::new(),
        "switch-stack-01".to_string(),
        SwitchModel::Cisco3850,
    );
    
    // Configure switch stack
    let stack_config = StackConfig {
        stack_members: vec![
            StackMember {
                number: 1,
                priority: 15, // Master
                mac_address: MacAddress::from_str("00:11:22:33:44:55").unwrap(),
            },
            StackMember {
                number: 2,
                priority: 10,
                mac_address: MacAddress::from_str("00:11:22:33:44:66").unwrap(),
            },
        ],
        stack_bandwidth: StackBandwidth::Full,
    };
    
    let events = config.configure_stack(
        stack_config.clone(),
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    assert_eq!(events.len(), 1);
    assert_eq!(config.stack_config(), Some(&stack_config));
}

#[test]
fn test_switch_configuration_validation() {
    let mut config = SwitchConfiguration::new(
        SwitchId::new(),
        "switch-01".to_string(),
        SwitchModel::Cisco2960X,
    );
    
    // Add some configuration
    let vlan = VlanId::try_new(100).unwrap();
    config.create_vlan(vlan, "Test".to_string(), CorrelationId::new(), CausationId::new()).unwrap();
    
    let port = PortNumber::try_new(1).unwrap();
    config.configure_port(
        PortConfig {
            number: port,
            description: None,
            mode: PortMode::Access,
            speed: PortSpeed::Gigabit,
            enabled: true,
            allowed_vlans: vec![vlan],
        },
        CorrelationId::new(),
        CausationId::new(),
    ).unwrap();
    
    // Validate configuration
    let validation = config.validate();
    assert!(validation.is_valid);
    assert!(validation.errors.is_empty());
    // It's ok to have warnings about VLANs without ports
    assert_eq!(validation.warnings.len(), 1);
    assert!(validation.warnings[0].contains("VLAN 100 has no assigned ports"));
}