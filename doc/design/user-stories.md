# CIM Network Module User Stories

## Epic: Network Infrastructure Management

### Story 1: Router Configuration Management
**As a** network administrator  
**I want to** define and deploy router configurations declaratively  
**So that** I can maintain consistent network configurations across all devices

#### Acceptance Criteria
- [ ] Support Cisco IOS configuration generation
- [ ] Support JunOS configuration generation  
- [ ] Validate configurations before deployment
- [ ] Deploy via Nix with rollback capability
- [ ] All changes tracked as events with correlation IDs

#### Test Scenarios
```gherkin
Given a Cisco router "edge-router-01"
When I apply a configuration with 3 interfaces and OSPF
Then the configuration is validated
And deployed via Nix
And a RouterConfigurationApplied event is emitted with correlation_id
```

### Story 2: VLAN Management Across Switches
**As a** network administrator  
**I want to** create VLANs that span multiple switches  
**So that** I can segment network traffic across the infrastructure

#### Acceptance Criteria
- [ ] Create VLAN with same ID on multiple switches
- [ ] Configure trunk ports automatically
- [ ] Validate VLAN ID (1-4094, exclude reserved)
- [ ] Update spanning tree configuration
- [ ] Emit VlanCreated events for each switch

#### Test Scenarios
```gherkin
Given 3 switches in the network
When I create VLAN 100 named "Production"
Then VLAN 100 exists on all 3 switches
And trunk ports are configured between switches
And each switch emits a VlanCreated event
```

### Story 3: Container Network with Physical VLAN
**As a** DevOps engineer  
**I want to** create container networks that integrate with physical VLANs  
**So that** containers can communicate with physical infrastructure

#### Acceptance Criteria
- [ ] Create bridge network with VLAN tagging
- [ ] Allocate IP range from physical subnet
- [ ] Configure gateway for external access
- [ ] Support Docker and Podman
- [ ] Track all containers in the network

#### Test Scenarios
```gherkin
Given VLAN 200 exists on physical switches
And subnet 10.200.0.0/24 is allocated to VLAN 200
When I create container network "app-net" on VLAN 200
Then bridge br-vlan200 is created with VLAN tag
And containers get IPs from 10.200.0.0/24
And containers can reach physical servers
```

### Story 4: Subnet Allocation Management
**As a** network architect  
**I want to** automatically allocate subnets from larger networks  
**So that** IP address space is efficiently managed without conflicts

#### Acceptance Criteria
- [ ] Allocate subnets of specified size
- [ ] Prevent overlapping allocations
- [ ] Track utilization percentage
- [ ] Support both IPv4 and IPv6
- [ ] Emit SubnetAllocated events

#### Test Scenarios
```gherkin
Given supernet 10.0.0.0/8
And existing allocation 10.1.0.0/16
When I request a /24 subnet
Then I receive 10.2.0.0/24
And no overlap with existing allocations
And SubnetAllocated event contains correlation_id
```

### Story 5: Nix Topology Generation
**As a** system administrator  
**I want to** generate Nix configurations from network topology  
**So that** network deployments are reproducible and declarative

#### Acceptance Criteria
- [ ] Generate valid Nix expressions
- [ ] Include all routers and switches
- [ ] Generate systemd services for deployment
- [ ] Support nixos-rebuild
- [ ] Include rollback mechanisms

#### Test Scenarios
```gherkin
Given a network with 2 routers and 3 switches
When I generate Nix topology
Then valid network.nix module is created
And it includes all device configurations
And can be deployed with nixos-rebuild
```

### Story 6: Network Status Visualization
**As a** network operator  
**I want to** see real-time network topology and status  
**So that** I can quickly identify and resolve issues

#### Acceptance Criteria
- [ ] Generate context graph of all devices
- [ ] Show device status (up/down)
- [ ] Display VLAN membership
- [ ] Show container networks
- [ ] Export as Mermaid diagram

#### Test Scenarios
```gherkin
Given a network with mixed physical and virtual components
When I request network visualization
Then a context graph is generated
And includes all routers, switches, and containers
And shows connections between components
And exports valid Mermaid syntax
```

### Story 7: Router State Machine Transitions
**As a** network automation system  
**I want to** manage router lifecycle with type-safe states  
**So that** invalid state transitions are impossible

#### Acceptance Criteria
- [ ] Implement states: Planned, Provisioning, Active, Failed, Maintenance
- [ ] Type-safe transitions using phantom types
- [ ] Emit events for each transition
- [ ] Persist state in event store
- [ ] Support state recovery

#### Test Scenarios
```gherkin
Given a router in Planned state
When I start provisioning
Then router transitions to Provisioning state
And RouterProvisioningStarted event is emitted
And invalid transitions don't compile
```

### Story 8: VM Network Integration
**As a** virtualization administrator  
**I want to** create VM networks that map to physical VLANs  
**So that** VMs can be part of physical network segments

#### Acceptance Criteria
- [ ] Support libvirt, VMware, Hyper-V
- [ ] Create networks with VLAN mapping
- [ ] Configure virtual switches
- [ ] Set bandwidth limits
- [ ] Track VM allocations

#### Test Scenarios
```gherkin
Given a libvirt hypervisor
And physical VLAN 300
When I create VM network "vm-prod" on VLAN 300
Then libvirt network is created with VLAN tag
And VMs can communicate with physical VLAN 300
```

## Non-Functional Requirements

### Performance
- Configuration generation < 100ms
- Event processing < 10ms  
- Graph generation < 1s for 1000 nodes
- Subnet allocation < 5ms

### Reliability
- All events persisted before acknowledgment
- Rollback on deployment failure
- Idempotent operations
- Eventually consistent projections

### Security
- Validate all input configurations
- Sanitize router passwords
- Audit trail via events
- Role-based access control ready

### Observability
- Structured logging with tracing
- Metrics for all operations
- Event correlation tracking
- Performance profiling

## Definition of Done

For each story:
- [ ] Tests written first (TDD)
- [ ] All tests pass
- [ ] Events have correlation/causation IDs
- [ ] No unwrap() in production code
- [ ] Documentation with examples
- [ ] Mermaid diagrams where applicable
- [ ] Code review completed
- [ ] Integration tests pass

## Implementation Priority

1. **Core Domain Model** (Stories 1, 7)
   - Router and Switch aggregates
   - State machines with phantom types
   - Event/Command structures

2. **Configuration Management** (Stories 1, 2)
   - Cisco IOS generator
   - VLAN management
   - Configuration validation

3. **Network Integration** (Stories 3, 8)
   - Container networks
   - VM networks
   - Physical VLAN mapping

4. **Infrastructure** (Stories 4, 5)
   - Subnet allocation
   - Nix topology generation
   - Deployment automation

5. **Visualization** (Story 6)
   - Context graph projection
   - Mermaid export
   - Status dashboard

## Success Metrics

- 100% of events have correlation/causation IDs
- Zero production unwrap() calls
- 90%+ test coverage
- All state transitions compile-time safe
- Deployment success rate > 99%
- Configuration generation < 100ms p99