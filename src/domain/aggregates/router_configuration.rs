//! Router configuration aggregate - the heart of router management

use crate::domain::{RouterId, CorrelationId, CausationId, EventId, IpNetwork};
use crate::domain::events::{RouterVendor, EventMetadata, NetworkEvent, RouterConfigSnapshot, Interface, RoutingProtocol, AccessList};
use crate::domain::errors::NetworkError;
use chrono::{DateTime, Utc};
use std::net::IpAddr;

/// Router configuration aggregate root
pub struct RouterConfiguration {
    id: RouterId,
    name: String,
    vendor: RouterVendor,
    interfaces: Vec<InterfaceConfig>,
    routing_protocols: Vec<RoutingProtocolConfig>,
    access_lists: Vec<AccessListConfig>,
    version: u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

/// Interface configuration
#[derive(Debug, Clone)]
pub struct InterfaceConfig {
    pub name: String,
    pub description: Option<String>,
    pub ip_address: Option<IpNetwork>,
    pub enabled: bool,
    pub vlan: Option<u16>,
}

/// Routing protocol configuration
#[derive(Debug, Clone)]
pub enum RoutingProtocolConfig {
    Ospf(OspfConfig),
    Bgp(BgpConfig),
    Static(StaticRouteConfig),
}

/// OSPF configuration
#[derive(Debug, Clone)]
pub struct OspfConfig {
    pub process_id: u32,
    pub router_id: IpAddr,
    pub areas: Vec<OspfAreaConfig>,
}

/// OSPF area configuration
#[derive(Debug, Clone)]
pub struct OspfAreaConfig {
    pub area_id: u32,
    pub area_type: AreaType,
    pub networks: Vec<IpNetwork>,
}

/// OSPF area type
#[derive(Debug, Clone)]
pub enum AreaType {
    Backbone,
    Standard,
    Stub,
    TotallyStubby,
    Nssa,
}

/// BGP configuration
#[derive(Debug, Clone)]
pub struct BgpConfig {
    pub local_as: u32,
    pub router_id: IpAddr,
    pub neighbors: Vec<BgpNeighborConfig>,
}

/// BGP neighbor configuration
#[derive(Debug, Clone)]
pub struct BgpNeighborConfig {
    pub address: IpAddr,
    pub remote_as: u32,
    pub description: Option<String>,
    pub password: Option<String>,
}

/// Static route configuration
#[derive(Debug, Clone)]
pub struct StaticRouteConfig {
    pub destination: IpNetwork,
    pub next_hop: IpAddr,
    pub metric: Option<u32>,
}

/// Access control list configuration
#[derive(Debug, Clone)]
pub struct AccessListConfig {
    pub number: u16,
    pub name: Option<String>,
    pub entries: Vec<AclEntryConfig>,
}

/// ACL entry configuration
#[derive(Debug, Clone)]
pub struct AclEntryConfig {
    pub sequence: u16,
    pub action: AclAction,
    pub protocol: Protocol,
    pub source: IpNetwork,
    pub destination: IpNetwork,
    pub ports: Option<PortRange>,
}

/// ACL action
#[derive(Debug, Clone)]
pub enum AclAction {
    Permit,
    Deny,
}

/// Protocol
#[derive(Debug, Clone)]
pub enum Protocol {
    Ip,
    Tcp,
    Udp,
    Icmp,
}

/// Port range for TCP/UDP
#[derive(Debug, Clone)]
pub struct PortRange {
    pub start: u16,
    pub end: u16,
}

/// Validation result
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl ValidationResult {
    fn new() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }
    
    fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.is_valid = false;
    }
    
    fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
    
    pub fn is_valid(&self) -> bool {
        self.is_valid
    }
}

/// Router template for common configurations
pub enum RouterTemplate {
    EdgeRouter {
        internal_networks: Vec<IpNetwork>,
        wan_ip: IpAddr,
    },
    CoreRouter {
        ospf_area: u32,
        loopback_ip: IpAddr,
    },
    AccessRouter {
        vlans: Vec<u16>,
        trunk_port: String,
    },
}

impl RouterTemplate {
    pub fn edge_router(internal_networks: Vec<IpNetwork>, wan_ip: IpAddr) -> Self {
        Self::EdgeRouter {
            internal_networks,
            wan_ip,
        }
    }
}

impl RouterConfiguration {
    /// Create a new router configuration
    pub fn new(id: RouterId, name: String, vendor: RouterVendor) -> Self {
        let now = Utc::now();
        Self {
            id,
            name,
            vendor,
            interfaces: Vec::new(),
            routing_protocols: Vec::new(),
            access_lists: Vec::new(),
            version: 0,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Get router ID
    pub fn id(&self) -> RouterId {
        self.id
    }
    
    /// Get router name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get current version
    pub fn version(&self) -> u64 {
        self.version
    }
    
    /// Get interfaces
    pub fn interfaces(&self) -> &[InterfaceConfig] {
        &self.interfaces
    }
    
    /// Get routing protocols
    pub fn routing_protocols(&self) -> &[RoutingProtocolConfig] {
        &self.routing_protocols
    }
    
    /// Get access lists
    pub fn access_lists(&self) -> &[AccessListConfig] {
        &self.access_lists
    }
    
    /// Add an interface
    pub fn add_interface(
        &mut self,
        interface: InterfaceConfig,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<NetworkEvent, NetworkError> {
        // Check for duplicate interface
        if self.interfaces.iter().any(|i| i.name == interface.name) {
            return Err(NetworkError::General(format!(
                "Interface {} already exists",
                interface.name
            )));
        }
        
        // Check for IP address conflicts
        if let Some(new_ip) = &interface.ip_address {
            for existing in &self.interfaces {
                if let Some(existing_ip) = &existing.ip_address {
                    // Check if networks overlap by comparing network addresses
                    let new_net = new_ip.inner();
                    let existing_net = existing_ip.inner();
                    
                    // Networks overlap if one contains the other
                    if new_net.network() == existing_net.network() || 
                       new_net.contains(existing_net.ip()) ||
                       existing_net.contains(new_net.ip()) {
                        return Err(NetworkError::General(format!(
                            "IP address {} overlaps with existing interface {}",
                            new_ip.inner(), existing.name
                        )));
                    }
                }
            }
        }
        
        // Add the interface
        self.interfaces.push(interface.clone());
        self.version += 1;
        self.updated_at = Utc::now();
        
        // Create event
        Ok(NetworkEvent::RouterInterfaceAdded {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: self.version,
            },
            router_id: self.id,
            interface: Interface {
                name: interface.name,
                ip_address: interface.ip_address.map(|ip| ip.inner().ip()),
                description: interface.description,
                enabled: interface.enabled,
            },
        })
    }
    
    /// Configure OSPF
    pub fn configure_ospf(
        &mut self,
        ospf: OspfConfig,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<NetworkEvent, NetworkError> {
        // Remove any existing OSPF configuration
        self.routing_protocols.retain(|p| !matches!(p, RoutingProtocolConfig::Ospf(_)));
        
        // Add new OSPF configuration
        self.routing_protocols.push(RoutingProtocolConfig::Ospf(ospf.clone()));
        self.version += 1;
        self.updated_at = Utc::now();
        
        // Create event
        Ok(NetworkEvent::RouterOspfConfigured {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: self.version,
            },
            router_id: self.id,
            process_id: ospf.process_id,
            areas: ospf.areas.len() as u32,
        })
    }
    
    /// Configure BGP
    pub fn configure_bgp(
        &mut self,
        bgp: BgpConfig,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<NetworkEvent, NetworkError> {
        // Remove any existing BGP configuration
        self.routing_protocols.retain(|p| !matches!(p, RoutingProtocolConfig::Bgp(_)));
        
        // Add new BGP configuration
        self.routing_protocols.push(RoutingProtocolConfig::Bgp(bgp.clone()));
        self.version += 1;
        self.updated_at = Utc::now();
        
        // Create event
        Ok(NetworkEvent::RouterBgpConfigured {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: self.version,
            },
            router_id: self.id,
            local_as: bgp.local_as,
            neighbors: bgp.neighbors.len() as u32,
        })
    }
    
    /// Add access list
    pub fn add_access_list(
        &mut self,
        acl: AccessListConfig,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<NetworkEvent, NetworkError> {
        // Check for duplicate ACL number
        if self.access_lists.iter().any(|a| a.number == acl.number) {
            return Err(NetworkError::General(format!(
                "Access list {} already exists",
                acl.number
            )));
        }
        
        // Add the ACL
        self.access_lists.push(acl.clone());
        self.version += 1;
        self.updated_at = Utc::now();
        
        // Create event
        Ok(NetworkEvent::RouterAccessListAdded {
            metadata: EventMetadata {
                event_id: EventId::new(),
                aggregate_id: self.id.into(),
                correlation_id,
                causation_id,
                timestamp: Utc::now(),
                version: self.version,
            },
            router_id: self.id,
            acl_number: acl.number,
            entries: acl.entries.len() as u32,
        })
    }
    
    /// Get configuration snapshot
    pub fn snapshot(&self) -> RouterConfigSnapshot {
        RouterConfigSnapshot {
            interfaces: self.interfaces.iter().map(|i| Interface {
                name: i.name.clone(),
                ip_address: i.ip_address.as_ref().map(|ip| ip.inner().ip()),
                description: i.description.clone(),
                enabled: i.enabled,
            }).collect(),
            routing_protocols: self.routing_protocols.iter().map(|p| {
                match p {
                    RoutingProtocolConfig::Ospf(ospf) => RoutingProtocol::Ospf {
                        process_id: ospf.process_id,
                        areas: ospf.areas.iter().map(|a| crate::domain::events::OspfArea {
                            id: a.area_id,
                            networks: a.networks.iter().map(|n| n.inner().clone()).collect(),
                        }).collect(),
                    },
                    RoutingProtocolConfig::Bgp(bgp) => RoutingProtocol::Bgp {
                        as_number: bgp.local_as,
                        neighbors: bgp.neighbors.iter().map(|n| crate::domain::events::BgpNeighbor {
                            address: n.address,
                            remote_as: n.remote_as,
                        }).collect(),
                    },
                    RoutingProtocolConfig::Static(_) => {
                        // TODO: Add static route support
                        RoutingProtocol::Ospf { process_id: 0, areas: vec![] }
                    }
                }
            }).collect(),
            access_lists: self.access_lists.iter().map(|acl| AccessList {
                id: acl.number.to_string(),
                entries: acl.entries.iter().map(|e| crate::domain::events::AclEntry {
                    action: match e.action {
                        AclAction::Permit => crate::domain::events::AclAction::Permit,
                        AclAction::Deny => crate::domain::events::AclAction::Deny,
                    },
                    protocol: match e.protocol {
                        Protocol::Ip => "ip",
                        Protocol::Tcp => "tcp",
                        Protocol::Udp => "udp",
                        Protocol::Icmp => "icmp",
                    }.to_string(),
                    source: e.source.inner().clone(),
                    destination: e.destination.inner().clone(),
                }).collect(),
            }).collect(),
            timestamp: Utc::now(),
        }
    }
    
    /// Validate configuration
    pub fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult::new();
        
        // Check for interfaces
        if self.interfaces.is_empty() {
            result.add_warning("No interfaces configured".to_string());
        }
        
        // Check for routing
        if self.routing_protocols.is_empty() {
            result.add_warning("No routing protocols configured".to_string());
        }
        
        // Validate IP addresses don't overlap
        for i in 0..self.interfaces.len() {
            for j in (i + 1)..self.interfaces.len() {
                if let (Some(ip1), Some(ip2)) = (&self.interfaces[i].ip_address, &self.interfaces[j].ip_address) {
                    let net1 = ip1.inner();
                    let net2 = ip2.inner();
                    
                    if net1.network() == net2.network() || 
                       net1.contains(net2.ip()) ||
                       net2.contains(net1.ip()) {
                        result.add_error(format!(
                            "IP address conflict between {} and {}",
                            self.interfaces[i].name,
                            self.interfaces[j].name
                        ));
                    }
                }
            }
        }
        
        // Validate OSPF areas
        if let Some(RoutingProtocolConfig::Ospf(ospf)) = self.routing_protocols.iter().find(|p| matches!(p, RoutingProtocolConfig::Ospf(_))) {
            if !ospf.areas.iter().any(|a| a.area_id == 0) {
                result.add_error("OSPF configuration missing area 0 (backbone)".to_string());
            }
        }
        
        result
    }
    
    /// Apply a template
    pub fn apply_template(
        &mut self,
        template: RouterTemplate,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<Vec<NetworkEvent>, NetworkError> {
        let mut events = Vec::new();
        
        match template {
            RouterTemplate::EdgeRouter { internal_networks, wan_ip } => {
                // Add WAN interface
                let wan_interface = InterfaceConfig {
                    name: "GigabitEthernet0/0".to_string(),
                    description: Some("WAN Uplink".to_string()),
                    ip_address: Some(IpNetwork::from_str(&format!("{}/30", wan_ip)).unwrap()),
                    enabled: true,
                    vlan: None,
                };
                events.push(self.add_interface(wan_interface, correlation_id.clone(), causation_id.clone())?);
                
                // Add LAN interfaces
                for (i, network) in internal_networks.iter().enumerate() {
                    let lan_interface = InterfaceConfig {
                        name: format!("GigabitEthernet0/{}", i + 1),
                        description: Some(format!("LAN Network {}", i + 1)),
                        ip_address: Some(network.clone()),
                        enabled: true,
                        vlan: None,
                    };
                    events.push(self.add_interface(lan_interface, correlation_id.clone(), causation_id.clone())?);
                }
                
                // Configure OSPF for internal networks
                let ospf = OspfConfig {
                    process_id: 1,
                    router_id: wan_ip,
                    areas: vec![
                        OspfAreaConfig {
                            area_id: 0,
                            area_type: AreaType::Backbone,
                            networks: internal_networks,
                        }
                    ],
                };
                events.push(self.configure_ospf(ospf, correlation_id.clone(), causation_id.clone())?);
                
                // Add basic security ACL
                let acl = AccessListConfig {
                    number: 100,
                    name: Some("DENY_RFC1918_INBOUND".to_string()),
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
                events.push(self.add_access_list(acl, correlation_id, causation_id)?);
            }
            _ => {
                return Err(NetworkError::General("Template not implemented yet".to_string()));
            }
        }
        
        Ok(events)
    }
}

// Re-export needed types
pub use std::str::FromStr;