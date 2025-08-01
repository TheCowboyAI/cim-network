# CIM Network Test Plan

## Test-Driven Development Order

### Phase 1: Value Objects (Day 1)
```rust
// tests/unit/value_objects_test.rs
#[test]
fn test_router_id_generation()
#[test]  
fn test_vlan_id_validation()
#[test]
fn test_mac_address_parsing()
#[test]
fn test_correlation_id_mandatory()
```

### Phase 2: Events (Day 2)
```rust
// tests/unit/events_test.rs
#[test]
fn test_event_has_correlation_id()
#[test]
fn test_event_has_causation_id()
#[test]
fn test_event_metadata_complete()
```

### Phase 3: State Machines (Day 3)
```rust
// tests/unit/state_machine_test.rs
#[test]
fn test_router_state_transitions()
#[test]
fn test_invalid_transitions_dont_compile()
#[test]
fn test_state_machine_events()
```

### Phase 4: Configuration Generators (Day 4-5)
```rust
// tests/unit/cisco_generator_test.rs
#[test]
fn test_cisco_basic_config()
#[test]
fn test_cisco_interface_config()
#[test]
fn test_cisco_routing_config()
```

### Phase 5: Integration Tests (Day 6+)
```rust
// tests/integration/network_setup_test.rs
#[tokio::test]
async fn test_complete_network_setup()
#[tokio::test]
async fn test_container_vlan_integration()
#[tokio::test]
async fn test_nix_topology_generation()
```

## Property-Based Tests
```rust
// tests/property/subnet_allocation_test.rs
proptest! {
    #[test]
    fn test_subnet_allocation_never_overlaps(...)
    #[test]
    fn test_vlan_id_always_valid(...)
}
```

## Definition of Done for Tests
- [ ] All tests written before implementation
- [ ] All tests pass
- [ ] No warnings
- [ ] Coverage > 90%
- [ ] Property tests for invariants