//! Tests for state machines - WRITTEN FIRST (TDD)

use cim_network::domain::state_machines::router::*;
use cim_network::domain::state_machines::{switch, network};
use cim_network::domain::{RouterId, SwitchId, NetworkId, CorrelationId, CausationId, IpNetwork};
use cim_network::domain::events::{RouterVendor, CiscoOs, RouterConfigSnapshot, MaintenanceWindow};
use std::str::FromStr;

#[test]
fn test_router_state_machine_creation() {
    let router = RouterStateMachine::<Planned>::new(
        RouterId::new(),
        "test-router".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    assert_eq!(router.name(), "test-router");
    // State is encoded in the type, no runtime check needed
}

#[test]
fn test_router_planned_to_provisioning() {
    let router = RouterStateMachine::<Planned>::new(
        RouterId::new(),
        "test-router".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
    let result = router.start_provisioning(correlation_id.clone(), causation_id.clone());
    assert!(result.is_ok());
    
    let (provisioning_router, event) = result.unwrap();
    assert_eq!(provisioning_router.name(), "test-router");
    
    // Verify event was generated
    match event {
        cim_network::NetworkEvent::RouterProvisioningStarted { metadata, .. } => {
            assert_eq!(metadata.correlation_id, correlation_id);
            assert_eq!(metadata.causation_id, causation_id);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_router_provisioning_to_configuring() {
    let router = RouterStateMachine::<Provisioning>::from_parts(
        RouterId::new(),
        "test-router".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
    let result = router.provisioning_complete(correlation_id, causation_id);
    assert!(result.is_ok());
    
    let (configuring_router, _event) = result.unwrap();
    assert_eq!(configuring_router.name(), "test-router");
}

#[test]
fn test_router_configuring_to_active() {
    let router = RouterStateMachine::<Configuring>::from_parts(
        RouterId::new(),
        "test-router".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let config = RouterConfigSnapshot {
        interfaces: vec![],
        routing_protocols: vec![],
        access_lists: vec![],
        timestamp: chrono::Utc::now(),
    };
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
    let result = router.configuration_applied(config, correlation_id, causation_id);
    assert!(result.is_ok());
    
    let (active_router, _event) = result.unwrap();
    assert_eq!(active_router.name(), "test-router");
}

#[test]
fn test_router_configuring_to_failed() {
    let router = RouterStateMachine::<Configuring>::from_parts(
        RouterId::new(),
        "test-router".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
    let result = router.configuration_failed(
        "Invalid configuration syntax".to_string(),
        correlation_id,
        causation_id,
    );
    assert!(result.is_ok());
    
    let (failed_router, _event) = result.unwrap();
    assert_eq!(failed_router.name(), "test-router");
    assert_eq!(failed_router.failure_reason(), "Invalid configuration syntax");
}

#[test]
fn test_router_active_to_maintenance() {
    let router = RouterStateMachine::<Active>::from_parts(
        RouterId::new(),
        "test-router".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
    );
    
    let window = MaintenanceWindow {
        start: chrono::Utc::now(),
        end: chrono::Utc::now() + chrono::Duration::hours(2),
        reason: "Firmware upgrade".to_string(),
    };
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
    let result = router.schedule_maintenance(window.clone(), correlation_id, causation_id);
    assert!(result.is_ok());
    
    let (maintenance_router, _event) = result.unwrap();
    assert_eq!(maintenance_router.name(), "test-router");
    assert_eq!(maintenance_router.maintenance_window().reason, "Firmware upgrade");
}

#[test]
fn test_router_failed_to_configuring_retry() {
    let router = RouterStateMachine::<Failed>::from_parts_with_failure(
        RouterId::new(),
        "test-router".to_string(),
        RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
        "Previous failure".to_string(),
    );
    
    let correlation_id = CorrelationId::new();
    let causation_id = CausationId::new();
    
    let result = router.retry_configuration(correlation_id, causation_id);
    assert!(result.is_ok());
    
    let (configuring_router, _event) = result.unwrap();
    assert_eq!(configuring_router.name(), "test-router");
}

// This should NOT compile - testing type safety
// #[test]
// fn test_invalid_state_transition_does_not_compile() {
//     let router = RouterStateMachine::<Planned>::new(
//         RouterId::new(),
//         "test-router".to_string(),
//         RouterVendor::Cisco { os: CiscoOs::Ios15_7 },
//     );
//     
//     // This should not compile - can't go from Planned to Active
//     // let active = router.configuration_applied(...);
// }

#[test]
fn test_switch_state_machine_creation() {
    let switch = switch::SwitchStateMachine::<switch::Planned>::new(
        SwitchId::new(),
        "core-switch-01".to_string(),
        48, // port count
    );
    
    assert_eq!(switch.name(), "core-switch-01");
    assert_eq!(switch.port_count(), 48);
}

#[test]
fn test_network_state_machine_creation() {
    let network = network::NetworkStateMachine::<network::Planning>::new(
        NetworkId::new(),
        "datacenter-network".to_string(),
        IpNetwork::from_str("10.0.0.0/16").unwrap(),
    );
    
    assert_eq!(network.name(), "datacenter-network");
    assert_eq!(network.cidr().inner().to_string(), "10.0.0.0/16");
}