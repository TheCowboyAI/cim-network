//! Tests for domain events - WRITTEN FIRST (TDD)

use cim_network::domain::events::*;
use cim_network::domain::value_objects::*;

#[test]
fn test_event_metadata_has_mandatory_fields() {
    let metadata = EventMetadata::new(
        AggregateId::new(),
        CorrelationId::new(),
        CausationId::new(),
    );
    
    // All mandatory fields must be present
    assert!(!metadata.event_id.to_string().is_empty());
    assert!(!metadata.correlation_id.to_string().is_empty());
    assert!(!metadata.causation_id.to_string().is_empty());
}

#[test]
fn test_event_metadata_never_has_optional_correlation() {
    // This should not compile if correlation_id is optional
    let metadata = EventMetadata::new(
        AggregateId::new(),
        CorrelationId::new(), // MUST be required, not Option<CorrelationId>
        CausationId::new(),
    );
    
    // Direct access should work (not through Option)
    let _correlation = metadata.correlation_id.clone();
}

#[test]
fn test_router_added_event() {
    let event = NetworkEvent::router_added(
        RouterId::new(),
        "edge-router-01".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
        CorrelationId::new(),
        CausationId::new(),
    );
    
    match event {
        NetworkEvent::RouterAdded { metadata, router_id, name, vendor } => {
            assert!(!metadata.correlation_id.to_string().is_empty());
            assert!(!metadata.causation_id.to_string().is_empty());
            assert_eq!(name, "edge-router-01");
            matches!(vendor, RouterVendor::Cisco { .. });
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_vlan_created_event() {
    let event = NetworkEvent::vlan_created(
        SwitchId::new(),
        VlanId::try_new(100).unwrap(),
        "Production".to_string(),
        CorrelationId::new(),
        CausationId::new(),
    );
    
    match event {
        NetworkEvent::VlanCreated { metadata, switch_id, vlan, .. } => {
            assert!(!metadata.correlation_id.to_string().is_empty());
            assert!(!metadata.causation_id.to_string().is_empty());
            assert_eq!(vlan.name, "Production");
            assert_eq!(vlan.id.value(), 100);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_router_configuration_applied_event() {
    let config = RouterConfigSnapshot {
        interfaces: vec![],
        routing_protocols: vec![],
        access_lists: vec![],
        timestamp: chrono::Utc::now(),
    };
    
    let event = NetworkEvent::router_configuration_applied(
        RouterId::new(),
        config.clone(),
        DeploymentMethod::Nix { flake_ref: "github:org/repo".to_string() },
        CorrelationId::new(),
        CausationId::new(),
    );
    
    match event {
        NetworkEvent::RouterConfigurationApplied { metadata, deployment_method, .. } => {
            assert!(!metadata.correlation_id.to_string().is_empty());
            assert!(!metadata.causation_id.to_string().is_empty());
            matches!(deployment_method, DeploymentMethod::Nix { .. });
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_container_network_created_event() {
    let event = NetworkEvent::container_network_created(
        ContainerNetworkId::new(),
        "app-network".to_string(),
        Some(VlanId::try_new(200).unwrap()),
        "10.200.0.0/24".parse().unwrap(),
        CorrelationId::new(),
        CausationId::new(),
    );
    
    match event {
        NetworkEvent::ContainerNetworkCreated { metadata, vlan_id, subnet, .. } => {
            assert!(!metadata.correlation_id.to_string().is_empty());
            assert!(!metadata.causation_id.to_string().is_empty());
            assert_eq!(vlan_id.unwrap().value(), 200);
            assert_eq!(subnet.to_string(), "10.200.0.0/24");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_event_builder_pattern() {
    let builder = NetworkEventBuilder::new()
        .with_correlation_id(CorrelationId::new())
        .with_causation_id(CausationId::new());
        
    let event = builder.build_router_added(
        RouterId::new(),
        "test-router".to_string(),
        RouterVendor::Vyos { version: "1.3".to_string() },
    );
    
    match event {
        NetworkEvent::RouterAdded { metadata, .. } => {
            assert!(!metadata.correlation_id.to_string().is_empty());
            assert!(!metadata.causation_id.to_string().is_empty());
        }
        _ => panic!("Wrong event type"),
    }
}