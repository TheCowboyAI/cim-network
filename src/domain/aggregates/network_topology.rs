//! Network topology aggregate - orchestrates complete network infrastructure

use crate::domain::{NetworkId, ConnectionId, VlanId, IpNetwork, CorrelationId, CausationId, EventId};
use crate::domain::events::{NetworkEvent, EventMetadata};
use crate::domain::errors::NetworkError;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::net::IpAddr;
use serde::{Serialize, Deserialize};

/// Network topology aggregate root - manages complete network infrastructure
pub struct NetworkTopology {
    id: NetworkTopologyId,
    name: String,
    base_ip: IpNetwork,
    topology_type: TopologyType,
    devices: HashMap<DeviceId, NetworkDevice>,
    connections: Vec<NetworkConnection>,
    nix_config: Option<NixTopologyConfig>,
    version: u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Network topology identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkTopologyId(uuid::Uuid);

impl NetworkTopologyId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl std::fmt::Display for NetworkTopologyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<NetworkTopologyId> for crate::domain::AggregateId {
    fn from(id: NetworkTopologyId) -> Self {
        Self(id.0.to_string())
    }
}

/// Types of network topologies that can be automatically generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopologyType {
    /// Single router with multiple interfaces
    SingleRouter {
        interface_count: u8,
    },
    /// Router connected to switch(es)
    RouterSwitch {
        switch_count: u8,
        ports_per_switch: u8,
    },
    /// Three-tier architecture (Core -> Distribution -> Access)
    ThreeTier { 
        core_count: u8, 
        distribution_count: u8, 
        access_count: u8,
        hosts_per_access: u8,
    },
    /// Spine-leaf architecture for data centers
    SpineLeaf { 
        spine_count: u8, 
        leaf_count: u8,
        hosts_per_leaf: u8,
    },
    /// Custom topology defined by user
    Custom(CustomTopologySpec),
}

/// Custom topology specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTopologySpec {
    pub devices: Vec<DeviceSpec>,
    pub connections: Vec<ConnectionSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSpec {
    pub name: String,
    pub device_type: DeviceType,
    pub interface_count: u8,
    pub ip_offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSpec {
    pub source_device: String,
    pub source_interface: String,
    pub target_device: String,
    pub target_interface: String,
    pub vlan: Option<VlanId>,
}

/// Network device in the topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
    pub id: DeviceId,
    pub name: String,
    pub device_type: DeviceType,
    pub ip_address: IpAddr,
    pub interfaces: Vec<InterfaceSpec>,
    pub nix_module: Option<String>,
    pub services: Vec<ServiceSpec>,
}

/// Device types supported in network topologies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    Router {
        vendor: RouterVendor,
        routing_protocols: Vec<RoutingProtocol>,
    },
    Switch {
        vendor: SwitchVendor,
        layer: SwitchLayer,
        vlan_support: bool,
    },
    Host {
        os: HostOS,
        services: Vec<String>,
    },
    Container {
        runtime: ContainerRuntime,
        image: String,
    },
    VM {
        hypervisor: Hypervisor,
        os: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouterVendor {
    Cisco,
    Juniper,
    VyOS,
    MikroTik,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwitchVendor {
    Cisco,
    Juniper,
    Arista,
    Generic,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwitchLayer {
    Layer2,
    Layer3,
    Layer4Plus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingProtocol {
    OSPF { area: String },
    BGP { asn: u32 },
    Static,
    RIP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HostOS {
    NixOS,
    Ubuntu,
    CentOS,
    Alpine,
    Windows,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerRuntime {
    Docker,
    Podman,
    Kubernetes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Hypervisor {
    KVM,
    VirtualBox,
    VMware,
    HyperV,
}

/// Interface specification for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSpec {
    pub id: InterfaceId,
    pub name: String,
    pub interface_type: InterfaceType,
    pub ip_address: Option<IpAddr>,
    pub subnet_mask: Option<u8>,
    pub vlan: Option<VlanId>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterfaceType {
    Ethernet,
    FastEthernet,
    GigabitEthernet,
    TenGigabitEthernet,
    Serial,
    Loopback,
    Virtual,
}

/// Service specification for devices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub name: String,
    pub service_type: ServiceType,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceType {
    DHCP,
    DNS,
    SSH,
    HTTP,
    HTTPS,
    SNMP,
    Syslog,
    Custom(String),
}

/// Connection between devices in the topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConnection {
    pub id: ConnectionId,
    pub source: ConnectionEndpoint,
    pub target: ConnectionEndpoint,
    pub connection_type: ConnectionType,
    pub vlan: Option<VlanId>,
    pub bandwidth: Option<Bandwidth>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEndpoint {
    pub device_id: DeviceId,
    pub interface_id: InterfaceId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    Ethernet,
    Fiber,
    Serial,
    Wireless,
    Virtual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bandwidth {
    pub speed: u64,  // in Mbps
    pub duplex: Duplex,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Duplex {
    Half,
    Full,
    Auto,
}

/// Nix topology configuration for deployment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixTopologyConfig {
    pub topology_name: String,
    pub base_network: IpNetwork,
    pub devices: Vec<NixDevice>,
    pub networks: Vec<NixNetwork>,
    pub rendered_config: Option<String>,
    pub mermaid_diagram: Option<String>,
    pub graph_json: Option<String>,
}

/// Device configuration for nix-topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixDevice {
    pub name: String,
    pub device_type: NixDeviceType,
    pub image: Option<String>,
    pub interfaces: Vec<NixInterface>,
    pub services: Vec<String>,
    pub nixos_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NixDeviceType {
    Router,
    Switch,
    Host,
    Container,
    VM,
}

/// Interface configuration for nix-topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixInterface {
    pub name: String,
    pub network: String,
    pub address: Option<String>,
    pub dhcp: bool,
}

/// Network definition for nix-topology
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixNetwork {
    pub name: String,
    pub cidr: String,
    pub vlan: Option<u16>,
    pub dhcp: bool,
    pub gateway: Option<String>,
}

impl NetworkTopology {
    /// Create a new network topology from basic parameters
    pub fn from_ip_and_name(
        base_ip: IpNetwork,
        name: String,
        topology_type: Option<TopologyType>,
    ) -> Result<Self, NetworkError> {
        let id = NetworkTopologyId::new();
        let now = Utc::now();
        
        let topology_type = topology_type.unwrap_or_else(|| {
            // Auto-detect topology based on IP range
            match base_ip.prefix() {
                24..=30 => TopologyType::RouterSwitch { 
                    switch_count: 1, 
                    ports_per_switch: 24 
                },
                16..=23 => TopologyType::ThreeTier { 
                    core_count: 2, 
                    distribution_count: 4, 
                    access_count: 8,
                    hosts_per_access: 12,
                },
                8..=15 => TopologyType::SpineLeaf { 
                    spine_count: 4, 
                    leaf_count: 16,
                    hosts_per_leaf: 32,
                },
                _ => TopologyType::SingleRouter { interface_count: 4 }
            }
        });
        
        let mut topology = Self {
            id,
            name,
            base_ip,
            topology_type,
            devices: HashMap::new(),
            connections: Vec::new(),
            nix_config: None,
            version: 1,
            created_at: now,
            updated_at: now,
        };
        
        // Generate topology based on type
        topology.generate_topology()?;
        
        Ok(topology)
    }
    
    /// Generate the network topology based on the topology type
    fn generate_topology(&mut self) -> Result<(), NetworkError> {
        match &self.topology_type {
            TopologyType::SingleRouter { interface_count } => {
                self.generate_single_router_topology(*interface_count)?;
            }
            TopologyType::RouterSwitch { switch_count, ports_per_switch } => {
                self.generate_router_switch_topology(*switch_count, *ports_per_switch)?;
            }
            TopologyType::ThreeTier { core_count, distribution_count, access_count, hosts_per_access } => {
                self.generate_three_tier_topology(*core_count, *distribution_count, *access_count, *hosts_per_access)?;
            }
            TopologyType::SpineLeaf { spine_count, leaf_count, hosts_per_leaf } => {
                self.generate_spine_leaf_topology(*spine_count, *leaf_count, *hosts_per_leaf)?;
            }
            TopologyType::Custom(spec) => {
                self.generate_custom_topology(spec)?;
            }
        }
        
        Ok(())
    }
    
    /// Generate a single router topology
    fn generate_single_router_topology(&mut self, interface_count: u8) -> Result<(), NetworkError> {
        let router_id = DeviceId::new();
        let base_ip = self.base_ip.network();
        
        let mut interfaces = Vec::new();
        for i in 0..interface_count {
            let interface_id = InterfaceId::new();
            let ip_addr = base_ip.nth(i as u32 + 1)
                .ok_or_else(|| NetworkError::InvalidConfiguration("Not enough IP addresses".to_string()))?;
            
            interfaces.push(InterfaceSpec {
                id: interface_id,
                name: format!("eth{}", i),
                interface_type: InterfaceType::GigabitEthernet,
                ip_address: Some(ip_addr),
                subnet_mask: Some(self.base_ip.prefix()),
                vlan: None,
                enabled: true,
            });
        }
        
        let router = NetworkDevice {
            id: router_id,
            name: format!("{}-router", self.name),
            device_type: DeviceType::Router {
                vendor: RouterVendor::Generic,
                routing_protocols: vec![RoutingProtocol::OSPF { area: "0".to_string() }],
            },
            ip_address: base_ip.nth(1).unwrap(),
            interfaces,
            nix_module: Some("router.nix".to_string()),
            services: vec![
                ServiceSpec {
                    name: "ssh".to_string(),
                    service_type: ServiceType::SSH,
                    config: HashMap::new(),
                },
            ],
        };
        
        self.devices.insert(router_id, router);
        Ok(())
    }
    
    /// Generate router-switch topology
    fn generate_router_switch_topology(&mut self, switch_count: u8, ports_per_switch: u8) -> Result<(), NetworkError> {
        let base_ip = self.base_ip.network();
        let router_id = DeviceId::new();
        
        // Create router
        let router = NetworkDevice {
            id: router_id,
            name: format!("{}-router", self.name),
            device_type: DeviceType::Router {
                vendor: RouterVendor::Generic,
                routing_protocols: vec![RoutingProtocol::OSPF { area: "0".to_string() }],
            },
            ip_address: base_ip.nth(1).unwrap(),
            interfaces: vec![
                InterfaceSpec {
                    id: InterfaceId::new(),
                    name: "eth0".to_string(),
                    interface_type: InterfaceType::GigabitEthernet,
                    ip_address: Some(base_ip.nth(1).unwrap()),
                    subnet_mask: Some(self.base_ip.prefix()),
                    vlan: None,
                    enabled: true,
                }
            ],
            nix_module: Some("router.nix".to_string()),
            services: vec![],
        };
        
        self.devices.insert(router_id, router);
        
        // Create switches
        for i in 0..switch_count {
            let switch_id = DeviceId::new();
            let switch_ip = base_ip.nth(i as u32 + 10).unwrap();
            
            let mut interfaces = vec![
                InterfaceSpec {
                    id: InterfaceId::new(),
                    name: "uplink".to_string(),
                    interface_type: InterfaceType::GigabitEthernet,
                    ip_address: Some(switch_ip),
                    subnet_mask: Some(self.base_ip.prefix()),
                    vlan: None,
                    enabled: true,
                }
            ];
            
            // Add access ports
            for port in 1..=ports_per_switch {
                interfaces.push(InterfaceSpec {
                    id: InterfaceId::new(),
                    name: format!("port{}", port),
                    interface_type: InterfaceType::GigabitEthernet,
                    ip_address: None,
                    subnet_mask: None,
                    vlan: Some(VlanId::from(100)),
                    enabled: true,
                });
            }
            
            let switch = NetworkDevice {
                id: switch_id,
                name: format!("{}-switch-{}", self.name, i + 1),
                device_type: DeviceType::Switch {
                    vendor: SwitchVendor::Generic,
                    layer: SwitchLayer::Layer2,
                    vlan_support: true,
                },
                ip_address: switch_ip,
                interfaces,
                nix_module: Some("switch.nix".to_string()),
                services: vec![],
            };
            
            self.devices.insert(switch_id, switch);
            
            // Create connection from router to switch
            self.connections.push(NetworkConnection {
                id: ConnectionId::new(),
                source: ConnectionEndpoint {
                    device_id: router_id,
                    interface_id: InterfaceId::new(), // Would need proper interface tracking
                },
                target: ConnectionEndpoint {
                    device_id: switch_id,
                    interface_id: InterfaceId::new(),
                },
                connection_type: ConnectionType::Ethernet,
                vlan: None,
                bandwidth: Some(Bandwidth {
                    speed: 1000,
                    duplex: Duplex::Full,
                }),
            });
        }
        
        Ok(())
    }
    
    /// Generate three-tier topology
    fn generate_three_tier_topology(&mut self, _core_count: u8, _distribution_count: u8, _access_count: u8, _hosts_per_access: u8) -> Result<(), NetworkError> {
        // Implementation for three-tier topology
        // This would be similar to router-switch but with hierarchical structure
        todo!("Implement three-tier topology generation")
    }
    
    /// Generate spine-leaf topology
    fn generate_spine_leaf_topology(&mut self, _spine_count: u8, _leaf_count: u8, _hosts_per_leaf: u8) -> Result<(), NetworkError> {
        // Implementation for spine-leaf topology
        // This creates a flat, scalable data center topology
        todo!("Implement spine-leaf topology generation")
    }
    
    /// Generate custom topology from specification
    fn generate_custom_topology(&mut self, _spec: &CustomTopologySpec) -> Result<(), NetworkError> {
        // Implementation for custom topology based on user specification
        todo!("Implement custom topology generation")
    }
    
    /// Generate Nix topology configuration
    pub fn generate_nix_topology(&mut self) -> Result<&NixTopologyConfig, NetworkError> {
        let mut nix_devices = Vec::new();
        let mut nix_networks = Vec::new();
        
        // Create network definition
        nix_networks.push(NixNetwork {
            name: "lan".to_string(),
            cidr: self.base_ip.to_string(),
            vlan: None,
            dhcp: false,
            gateway: Some(self.base_ip.network().nth(1).unwrap().to_string()),
        });
        
        // Convert devices to nix format
        for device in self.devices.values() {
            let nix_device_type = match &device.device_type {
                DeviceType::Router { .. } => NixDeviceType::Router,
                DeviceType::Switch { .. } => NixDeviceType::Switch,
                DeviceType::Host { .. } => NixDeviceType::Host,
                DeviceType::Container { .. } => NixDeviceType::Container,
                DeviceType::VM { .. } => NixDeviceType::VM,
            };
            
            let mut nix_interfaces = Vec::new();
            for interface in &device.interfaces {
                nix_interfaces.push(NixInterface {
                    name: interface.name.clone(),
                    network: "lan".to_string(),
                    address: interface.ip_address.map(|ip| 
                        format!("{}/{}", ip, interface.subnet_mask.unwrap_or(24))
                    ),
                    dhcp: false,
                });
            }
            
            nix_devices.push(NixDevice {
                name: device.name.clone(),
                device_type: nix_device_type,
                image: Some("nixos/latest".to_string()),
                interfaces: nix_interfaces,
                services: device.services.iter().map(|s| s.name.clone()).collect(),
                nixos_modules: vec![device.nix_module.clone().unwrap_or_default()],
            });
        }
        
        let nix_config = NixTopologyConfig {
            topology_name: self.name.clone(),
            base_network: self.base_ip,
            devices: nix_devices,
            networks: nix_networks,
            rendered_config: None,
            mermaid_diagram: None,
            graph_json: None,
        };
        
        self.nix_config = Some(nix_config);
        Ok(self.nix_config.as_ref().unwrap())
    }
    
    /// Apply a change to the topology and return domain events
    pub fn apply_change(
        &mut self,
        change: TopologyChange,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<Vec<NetworkEvent>, NetworkError> {
        let mut events = Vec::new();
        
        match change {
            TopologyChange::AddDevice(device) => {
                let device_id = device.id;
                self.devices.insert(device_id, device);
                
                events.push(NetworkEvent::TopologyDeviceAdded {
                    metadata: EventMetadata {
                        event_id: EventId::new(),
                        aggregate_id: self.id.into(),
                        correlation_id,
                        causation_id,
                        timestamp: Utc::now(),
                        version: self.version,
                    },
                    topology_id: self.id,
                    device_id,
                });
            }
            TopologyChange::RemoveDevice(device_id) => {
                if self.devices.remove(&device_id).is_some() {
                    events.push(NetworkEvent::TopologyDeviceRemoved {
                        metadata: EventMetadata {
                            event_id: EventId::new(),
                            aggregate_id: self.id.into(),
                            correlation_id,
                            causation_id,
                            timestamp: Utc::now(),
                            version: self.version,
                        },
                        topology_id: self.id,
                        device_id,
                    });
                }
            }
            TopologyChange::AddConnection(connection) => {
                let connection_id = connection.id;
                self.connections.push(connection);
                
                events.push(NetworkEvent::TopologyConnectionAdded {
                    metadata: EventMetadata {
                        event_id: EventId::new(),
                        aggregate_id: self.id.into(),
                        correlation_id,
                        causation_id,
                        timestamp: Utc::now(),
                        version: self.version,
                    },
                    topology_id: self.id,
                    connection_id,
                });
            }
        }
        
        self.version += 1;
        self.updated_at = Utc::now();
        
        Ok(events)
    }
    
    // Getters
    pub fn id(&self) -> NetworkTopologyId { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn base_ip(&self) -> IpNetwork { self.base_ip }
    pub fn topology_type(&self) -> &TopologyType { &self.topology_type }
    pub fn devices(&self) -> &HashMap<DeviceId, NetworkDevice> { &self.devices }
    pub fn connections(&self) -> &[NetworkConnection] { &self.connections }
    pub fn nix_config(&self) -> Option<&NixTopologyConfig> { self.nix_config.as_ref() }
    pub fn version(&self) -> u64 { self.version }
}

/// Changes that can be applied to a network topology
pub enum TopologyChange {
    AddDevice(NetworkDevice),
    RemoveDevice(DeviceId),
    AddConnection(NetworkConnection),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    
    #[test]
    fn test_create_topology_from_ip_and_name() {
        let ip = IpNetwork::from_str("192.168.1.0/24").unwrap();
        let topology = NetworkTopology::from_ip_and_name(
            ip,
            "test-network".to_string(),
            None,
        ).unwrap();
        
        assert_eq!(topology.name(), "test-network");
        assert_eq!(topology.base_ip(), ip);
        assert!(!topology.devices().is_empty());
    }
    
    #[test]
    fn test_generate_nix_topology() {
        let ip = IpNetwork::from_str("192.168.1.0/24").unwrap();
        let mut topology = NetworkTopology::from_ip_and_name(
            ip,
            "test-network".to_string(),
            Some(TopologyType::SingleRouter { interface_count: 2 }),
        ).unwrap();
        
        let nix_config = topology.generate_nix_topology().unwrap();
        assert_eq!(nix_config.topology_name, "test-network");
        assert!(!nix_config.devices.is_empty());
        assert!(!nix_config.networks.is_empty());
    }
}