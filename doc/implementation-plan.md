# CIM Network Implementation Plan

## Overview

This plan follows all CIM patterns and rules from `./.claude/` to implement network infrastructure management with event sourcing, DDD, and Nix deployment.

## Core Principles (from ./.claude/)

1. **Event-Driven Everything**: No CRUD, only events with mandatory correlation/causation IDs
2. **Living Information**: Information flows as immutable events
3. **Visual Programming**: Mermaid diagrams are mandatory
4. **Test-Driven Development**: Red-Green-Refactor with 100% coverage goal
5. **Type Safety**: Phantom types for state machines
6. **Nix Integration**: All deployments via Nix

## Implementation Phases

### Phase 1: Foundation (Days 1-3)
**Goal**: Establish clean domain model for network infrastructure

#### Day 1: Domain Setup
```rust
// Create these files following DDD patterns

// src/domain/mod.rs
pub mod value_objects;
pub mod aggregates;
pub mod events;
pub mod commands;
pub mod errors;

// src/domain/value_objects.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RouterId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SwitchId(Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VlanId(u16);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MacAddress([u8; 6]);

// src/domain/aggregates/network_infrastructure.rs
pub struct NetworkInfrastructure {
    pub id: NetworkId,
    pub name: String,
    pub devices: HashMap<DeviceId, NetworkDevice>,
    pub topology: NetworkTopology,
    pub version: u64,
}
```

**Tests First (TDD)**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_router_id_generation() {
        let id1 = RouterId::new();
        let id2 = RouterId::new();
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_vlan_id_validation() {
        assert!(VlanId::try_new(4094).is_ok());
        assert!(VlanId::try_new(4095).is_err()); // Reserved
    }
}
```

#### Day 2: Event Model with Mandatory Fields
```rust
// src/domain/events.rs
// CRITICAL: All events MUST have correlation_id and causation_id

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: EventId,
    pub aggregate_id: AggregateId,
    pub correlation_id: CorrelationId,  // MANDATORY
    pub causation_id: CausationId,      // MANDATORY
    pub timestamp: DateTime<Utc>,
    pub version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    RouterAdded {
        metadata: EventMetadata,
        router_id: RouterId,
        name: String,
        vendor: RouterVendor,
    },
    RouterConfigurationApplied {
        metadata: EventMetadata,
        router_id: RouterId,
        configuration: RouterConfigSnapshot,
        deployment_method: DeploymentMethod,
    },
    VlanCreated {
        metadata: EventMetadata,
        vlan_id: VlanId,
        name: String,
        switches: Vec<SwitchId>,
    },
}
```

#### Day 3: State Machines with Phantom Types
```rust
// src/domain/state_machines/router.rs
use std::marker::PhantomData;

// States
pub struct Planned;
pub struct Provisioning;
pub struct Configuring;
pub struct Active;
pub struct Failed { pub reason: String }

// State machine
pub struct RouterStateMachine<S> {
    pub id: RouterId,
    pub name: String,
    pub vendor: RouterVendor,
    _state: PhantomData<S>,
}

// Type-safe transitions
impl RouterStateMachine<Planned> {
    pub fn start_provisioning(self) -> Result<RouterStateMachine<Provisioning>, RouterError> {
        Ok(RouterStateMachine {
            id: self.id,
            name: self.name,
            vendor: self.vendor,
            _state: PhantomData,
        })
    }
}

impl RouterStateMachine<Configuring> {
    pub fn configuration_applied(self) -> RouterStateMachine<Active> {
        RouterStateMachine {
            id: self.id,
            name: self.name,
            vendor: self.vendor,
            _state: PhantomData,
        }
    }
    
    pub fn configuration_failed(self, reason: String) -> RouterStateMachine<Failed> {
        RouterStateMachine {
            id: self.id,
            name: self.name,
            vendor: self.vendor,
            _state: PhantomData,
        }
    }
}
```

### Phase 2: Configuration Management (Days 4-6)
**Goal**: Vendor-specific configuration generation

#### Day 4: Configuration Generator Trait
```rust
// src/domain/configuration/mod.rs
pub trait ConfigurationGenerator {
    type Input;
    type Output;
    type Error;
    
    fn generate(&self, input: &Self::Input) -> Result<Self::Output, Self::Error>;
    fn validate(&self, output: &Self::Output) -> Result<(), Self::Error>;
}

// src/domain/configuration/cisco.rs
pub struct CiscoIosGenerator {
    templates: TemplateEngine,
}

impl ConfigurationGenerator for CiscoIosGenerator {
    type Input = RouterConfiguration;
    type Output = String;
    type Error = ConfigError;
    
    fn generate(&self, config: &RouterConfiguration) -> Result<String, ConfigError> {
        let mut output = String::new();
        
        // Hostname
        writeln!(output, "hostname {}", config.name)?;
        
        // Interfaces
        for interface in &config.interfaces {
            writeln!(output, "interface {}", interface.name)?;
            writeln!(output, " ip address {} {}", 
                interface.ip_address.ip(), 
                interface.ip_address.netmask()
            )?;
            if interface.enabled {
                writeln!(output, " no shutdown")?;
            }
        }
        
        Ok(output)
    }
}
```

#### Day 5: VLAN and Switch Configuration
```rust
// src/domain/aggregates/switch_configuration.rs
pub struct SwitchConfiguration {
    pub id: SwitchId,
    pub name: String,
    pub vendor: SwitchVendor,
    pub ports: Vec<SwitchPort>,
    pub vlans: HashMap<VlanId, Vlan>,
    pub spanning_tree: SpanningTreeConfig,
}

pub struct Vlan {
    pub id: VlanId,
    pub name: String,
    pub tagged_ports: Vec<PortNumber>,
    pub untagged_ports: Vec<PortNumber>,
}

impl SwitchConfiguration {
    pub fn create_vlan(&mut self, id: VlanId, name: String) -> Result<VlanCreated, SwitchError> {
        if self.vlans.contains_key(&id) {
            return Err(SwitchError::VlanAlreadyExists(id));
        }
        
        self.vlans.insert(id, Vlan {
            id,
            name: name.clone(),
            tagged_ports: vec![],
            untagged_ports: vec![],
        });
        
        Ok(VlanCreated {
            switch_id: self.id,
            vlan_id: id,
            name,
        })
    }
}
```

#### Day 6: IP Address Management
```rust
// src/domain/services/ipam.rs
pub struct IpAddressManager {
    allocations: HashMap<IpNetwork, SubnetAllocations>,
}

impl IpAddressManager {
    pub fn allocate_subnet(
        &mut self,
        from: IpNetwork,
        size: u8,
    ) -> Result<IpNetwork, IpamError> {
        let allocations = self.allocations.entry(from).or_default();
        
        // Find next available subnet of requested size
        for subnet in from.subnets_with_prefix(size) {
            if !allocations.is_allocated(&subnet) {
                allocations.mark_allocated(subnet);
                return Ok(subnet);
            }
        }
        
        Err(IpamError::NoAvailableSubnet)
    }
}
```

### Phase 3: Container/VM Networking (Days 7-9)
**Goal**: Integration with virtualization platforms

#### Day 7: Container Network Model
```rust
// src/domain/aggregates/container_network.rs
pub struct ContainerNetwork {
    pub id: ContainerNetworkId,
    pub name: String,
    pub driver: NetworkDriver,
    pub vlan_id: Option<VlanId>,
    pub subnet: IpNetwork,
    pub gateway: Option<IpAddr>,
    pub dns_servers: Vec<IpAddr>,
}

pub enum NetworkDriver {
    Bridge {
        bridge_name: String,
        enable_icc: bool,
    },
    Overlay {
        encryption: bool,
        subnet_pool: Vec<IpNetwork>,
    },
    Macvlan {
        parent_interface: String,
        mode: MacvlanMode,
    },
}

impl ContainerNetwork {
    pub fn attach_container(
        &mut self,
        container_id: ContainerId,
        ip_address: Option<IpAddr>,
    ) -> Result<ContainerAttached, NetworkError> {
        let ip = match ip_address {
            Some(ip) => self.validate_ip(ip)?,
            None => self.allocate_next_ip()?,
        };
        
        Ok(ContainerAttached {
            network_id: self.id,
            container_id,
            ip_address: ip,
        })
    }
}
```

#### Day 8: VM Network Integration
```rust
// src/domain/aggregates/vm_network.rs
pub struct VmNetwork {
    pub id: VmNetworkId,
    pub name: String,
    pub hypervisor: Hypervisor,
    pub network_type: VmNetworkType,
    pub vlan_id: Option<VlanId>,
}

pub enum Hypervisor {
    Libvirt {
        connection_uri: String,
    },
    VMware {
        vcenter: String,
        datacenter: String,
    },
    HyperV {
        host: String,
    },
}

pub enum VmNetworkType {
    Bridged {
        bridge_name: String,
    },
    NAT {
        subnet: IpNetwork,
        dhcp_enabled: bool,
    },
    Internal {
        isolated: bool,
    },
}
```

#### Day 9: Physical Network Integration
```rust
// src/application/services/network_integration.rs
pub struct NetworkIntegrationService {
    infrastructure: Arc<RwLock<NetworkInfrastructure>>,
    event_store: Arc<dyn EventStore>,
}

impl NetworkIntegrationService {
    pub async fn create_container_network_with_vlan(
        &self,
        name: String,
        vlan_id: VlanId,
        subnet: IpNetwork,
    ) -> Result<ContainerNetwork, IntegrationError> {
        // Begin transaction
        let correlation_id = CorrelationId::new();
        
        // Verify VLAN exists on switches
        let infrastructure = self.infrastructure.read().await;
        let switches_with_vlan = infrastructure.find_switches_with_vlan(vlan_id)?;
        
        if switches_with_vlan.is_empty() {
            return Err(IntegrationError::VlanNotFound(vlan_id));
        }
        
        // Create container network
        let network = ContainerNetwork {
            id: ContainerNetworkId::new(),
            name,
            driver: NetworkDriver::Bridge {
                bridge_name: format!("br-vlan{}", vlan_id.0),
                enable_icc: true,
            },
            vlan_id: Some(vlan_id),
            subnet,
            gateway: Some(subnet.nth(1).unwrap()), // .1 as gateway
            dns_servers: vec![],
        };
        
        // Emit event
        self.event_store.append(NetworkEvent::ContainerNetworkCreated {
            metadata: EventMetadata::new(correlation_id, correlation_id),
            network_id: network.id,
            name: network.name.clone(),
            vlan_id: Some(vlan_id),
            subnet,
        }).await?;
        
        Ok(network)
    }
}
```

### Phase 4: Nix Integration (Days 10-12)
**Goal**: Generate and deploy Nix configurations

#### Day 10: Nix Expression Generation
```rust
// src/infrastructure/nix/generator.rs
pub struct NixTopologyGenerator;

impl NixTopologyGenerator {
    pub fn generate_network_module(
        &self,
        infrastructure: &NetworkInfrastructure,
    ) -> Result<String, NixError> {
        let mut nix = String::new();
        
        writeln!(nix, "{{ config, lib, pkgs, ... }}:")?;
        writeln!(nix, "{{")?;
        writeln!(nix, "  networking = {{")?;
        
        // Generate router configurations
        for (id, device) in &infrastructure.devices {
            if let NetworkDevice::Router(router) = device {
                writeln!(nix, "    # Router: {}", router.name)?;
                writeln!(nix, "    routers.\"{}\" = {{", router.name)?;
                writeln!(nix, "      vendor = \"{}\";", router.vendor)?;
                writeln!(nix, "      configFile = pkgs.writeText \"router-config\" ''")?;
                
                let config = self.generate_router_config(router)?;
                for line in config.lines() {
                    writeln!(nix, "        {}", line)?;
                }
                
                writeln!(nix, "      '';")?;
                writeln!(nix, "    }};")?;
            }
        }
        
        writeln!(nix, "  }};")?;
        writeln!(nix, "}}")?;
        
        Ok(nix)
    }
}
```

#### Day 11: Deployment Automation
```rust
// src/infrastructure/deployment/nix_deploy.rs
pub struct NixDeploymentService {
    event_bus: Arc<dyn EventBus>,
}

impl NixDeploymentService {
    pub async fn deploy_configuration(
        &self,
        target: DeploymentTarget,
        config: NixConfiguration,
    ) -> Result<DeploymentResult, DeploymentError> {
        let correlation_id = CorrelationId::new();
        
        // Validate configuration
        self.validate_nix_config(&config)?;
        
        // Create deployment flake
        let flake_path = self.create_deployment_flake(&config)?;
        
        // Deploy
        let result = match target {
            DeploymentTarget::Local => {
                self.deploy_local(flake_path).await?
            }
            DeploymentTarget::Remote { host, user } => {
                self.deploy_remote(flake_path, &host, &user).await?
            }
        };
        
        // Emit deployment event
        self.event_bus.publish(NetworkEvent::ConfigurationDeployed {
            metadata: EventMetadata::new(correlation_id, correlation_id),
            target,
            result: result.clone(),
        }).await?;
        
        Ok(result)
    }
}
```

#### Day 12: Configuration Validation
```rust
// src/infrastructure/validation/config_validator.rs
pub struct ConfigurationValidator {
    rules: Vec<Box<dyn ValidationRule>>,
}

impl ConfigurationValidator {
    pub fn validate_router_config(
        &self,
        config: &RouterConfiguration,
    ) -> Result<(), ValidationError> {
        // Check for IP conflicts
        let mut seen_ips = HashSet::new();
        for interface in &config.interfaces {
            if !seen_ips.insert(interface.ip_address) {
                return Err(ValidationError::DuplicateIpAddress(interface.ip_address));
            }
        }
        
        // Validate routing protocols
        for protocol in &config.routing_protocols {
            self.validate_routing_protocol(protocol)?;
        }
        
        // Run custom rules
        for rule in &self.rules {
            rule.validate(config)?;
        }
        
        Ok(())
    }
}
```

### Phase 5: Context Graph and Visualization (Days 13-15)
**Goal**: Network topology visualization

#### Day 13: Context Graph Projection
```rust
// src/projections/context_graph.rs
pub struct NetworkContextGraph {
    graph: Graph<NetworkNode, NetworkEdge>,
    node_index: HashMap<String, NodeIndex>,
}

#[derive(Debug, Clone)]
pub enum NetworkNode {
    Router {
        id: RouterId,
        name: String,
        vendor: RouterVendor,
        interfaces: Vec<Interface>,
    },
    Switch {
        id: SwitchId,
        name: String,
        vlans: Vec<VlanId>,
    },
    Container {
        id: ContainerId,
        name: String,
        network: ContainerNetworkId,
        ip_address: IpAddr,
    },
}

impl NetworkContextGraph {
    pub fn from_infrastructure(
        infrastructure: &NetworkInfrastructure,
    ) -> Result<Self, GraphError> {
        let mut graph = Graph::new();
        let mut node_index = HashMap::new();
        
        // Add all devices as nodes
        for (id, device) in &infrastructure.devices {
            let node = match device {
                NetworkDevice::Router(r) => NetworkNode::Router {
                    id: r.id,
                    name: r.name.clone(),
                    vendor: r.vendor.clone(),
                    interfaces: r.interfaces.clone(),
                },
                NetworkDevice::Switch(s) => NetworkNode::Switch {
                    id: s.id,
                    name: s.name.clone(),
                    vlans: s.vlans.keys().cloned().collect(),
                },
            };
            
            let idx = graph.add_node(node);
            node_index.insert(id.to_string(), idx);
        }
        
        // Add edges based on connections
        // ... connection logic
        
        Ok(Self { graph, node_index })
    }
}
```

#### Day 14: Mermaid Diagram Generation
```rust
// src/projections/mermaid.rs
pub trait MermaidDiagram {
    fn to_mermaid(&self) -> String;
}

impl MermaidDiagram for NetworkContextGraph {
    fn to_mermaid(&self) -> String {
        let mut mermaid = String::from("graph TB\n");
        
        // Add nodes
        for node in self.graph.node_indices() {
            let node_data = &self.graph[node];
            match node_data {
                NetworkNode::Router { name, vendor, .. } => {
                    writeln!(mermaid, "    {}[Router: {} - {:?}]", 
                        node.index(), name, vendor
                    ).unwrap();
                }
                NetworkNode::Switch { name, vlans, .. } => {
                    writeln!(mermaid, "    {}[Switch: {} - {} VLANs]", 
                        node.index(), name, vlans.len()
                    ).unwrap();
                }
                NetworkNode::Container { name, ip_address, .. } => {
                    writeln!(mermaid, "    {}[Container: {} - {}]", 
                        node.index(), name, ip_address
                    ).unwrap();
                }
            }
        }
        
        // Add edges
        for edge in self.graph.edge_indices() {
            let (source, target) = self.graph.edge_endpoints(edge).unwrap();
            writeln!(mermaid, "    {} --> {}", source.index(), target.index()).unwrap();
        }
        
        mermaid
    }
}
```

#### Day 15: Status Dashboard
```rust
// src/projections/status_dashboard.rs
pub struct NetworkStatusDashboard {
    device_status: HashMap<DeviceId, DeviceStatus>,
    port_statistics: HashMap<(DeviceId, PortNumber), PortStats>,
    recent_events: VecDeque<NetworkEvent>,
}

pub struct DeviceStatus {
    pub state: DeviceState,
    pub last_seen: DateTime<Utc>,
    pub uptime: Duration,
    pub cpu_usage: f32,
    pub memory_usage: f32,
}

impl NetworkStatusDashboard {
    pub fn update_from_event(&mut self, event: &NetworkEvent) {
        // Update status based on event type
        match event {
            NetworkEvent::RouterStatusChanged { router_id, status, .. } => {
                self.device_status.insert(
                    DeviceId::Router(*router_id),
                    DeviceStatus {
                        state: status.clone(),
                        last_seen: Utc::now(),
                        // ... other fields
                    }
                );
            }
            // ... handle other events
        }
        
        // Keep last 100 events
        self.recent_events.push_back(event.clone());
        if self.recent_events.len() > 100 {
            self.recent_events.pop_front();
        }
    }
}
```

## Testing Strategy

### Unit Tests (Every Day)
```rust
// Example test structure for each component
#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    
    #[test]
    fn test_event_has_correlation_id() {
        let event = NetworkEvent::router_added(
            RouterId::new(),
            "test-router".to_string(),
        );
        
        assert!(event.metadata().correlation_id.is_some());
        assert!(event.metadata().causation_id.is_some());
    }
    
    proptest! {
        #[test]
        fn test_subnet_allocation_never_overlaps(
            subnet_size in 24u8..=30u8,
            allocation_count in 1usize..=10usize,
        ) {
            let mut ipam = IpAddressManager::new();
            let base = "10.0.0.0/16".parse().unwrap();
            
            let mut allocations = vec![];
            for _ in 0..allocation_count {
                if let Ok(subnet) = ipam.allocate_subnet(base, subnet_size) {
                    allocations.push(subnet);
                }
            }
            
            // Check no overlaps
            for i in 0..allocations.len() {
                for j in (i + 1)..allocations.len() {
                    assert!(!allocations[i].overlaps(&allocations[j]));
                }
            }
        }
    }
}
```

### Integration Tests (End of Each Phase)
```rust
#[tokio::test]
async fn test_complete_network_setup() {
    let event_store = InMemoryEventStore::new();
    let service = NetworkService::new(event_store);
    
    // Create infrastructure
    let network = service.create_network("datacenter", "10.0.0.0/16").await?;
    
    // Add devices
    let router = service.add_router(network.id, "edge-router", RouterVendor::Cisco).await?;
    let switch = service.add_switch(network.id, "core-switch", 48).await?;
    
    // Create VLAN
    service.create_vlan(switch.id, 100, "production").await?;
    
    // Create container network with VLAN
    let container_net = service.create_container_network(
        "app-network",
        Some(VlanId(100)),
        "10.0.100.0/24"
    ).await?;
    
    // Verify graph generation
    let graph = service.generate_context_graph().await?;
    assert!(graph.node_count() >= 2);
    
    // Verify Nix generation
    let nix_config = service.generate_nix_topology().await?;
    assert!(nix_config.contains("networking"));
}
```

## Validation Checklist

Daily validation:
- [ ] `cargo build` - Zero errors
- [ ] `cargo test` - All tests pass
- [ ] `cargo clippy` - No warnings
- [ ] All events have correlation_id and causation_id
- [ ] State machines use phantom types
- [ ] No `unwrap()` in production code

Phase validation:
- [ ] Integration tests pass
- [ ] Documentation updated
- [ ] Mermaid diagrams created
- [ ] Performance benchmarks run

## Success Metrics

1. **Code Quality**
   - 100% of events have correlation/causation IDs
   - Zero `unwrap()` calls in src/
   - All state transitions type-safe

2. **Test Coverage**
   - Unit test coverage > 90%
   - Integration test coverage > 80%
   - Property tests for critical algorithms

3. **Performance**
   - Event processing < 10ms
   - Configuration generation < 100ms
   - Graph visualization < 1s for 1000 nodes

4. **Functionality**
   - Can deploy router config via Nix
   - Container networks integrate with VLANs
   - Context graphs accurately represent topology

## Risk Mitigation

1. **Complexity Risk**
   - Start with single vendor (Cisco)
   - Add others incrementally
   - Keep abstractions minimal

2. **Integration Risk**
   - Mock external systems first
   - Test with real systems in isolated environment
   - Have rollback strategy

3. **Performance Risk**
   - Design for streaming from start
   - Use pagination for large datasets
   - Profile early and often

## Next Steps After Implementation

1. Add more vendor support
2. Implement BGP and OSPF configuration
3. Add monitoring integration (Prometheus)
4. Create web UI for visualization
5. Build Terraform provider