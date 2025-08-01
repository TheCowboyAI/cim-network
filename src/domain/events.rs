//! Domain events for network infrastructure
//! 
//! All events MUST have correlation_id and causation_id (NEVER optional)

use super::value_objects::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use ipnetwork::IpNetwork;

/// Event metadata - ALL fields are MANDATORY
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Unique event identifier
    pub event_id: EventId,
    /// Aggregate that this event belongs to
    pub aggregate_id: AggregateId,
    /// Correlation ID for tracking related events (MANDATORY - never Option)
    pub correlation_id: CorrelationId,
    /// Causation ID for tracking what caused this event (MANDATORY - never Option)
    pub causation_id: CausationId,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Version for optimistic concurrency
    pub version: u64,
}

impl EventMetadata {
    /// Create new event metadata with all mandatory fields
    pub fn new(
        aggregate_id: AggregateId,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Self {
        Self {
            event_id: EventId::new(),
            aggregate_id,
            correlation_id,
            causation_id,
            timestamp: Utc::now(),
            version: 1,
        }
    }
}

/// Network infrastructure events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkEvent {
    /// Router was added to the infrastructure
    RouterAdded {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Router identifier
        router_id: RouterId,
        /// Router name
        name: String,
        /// Router vendor and OS
        vendor: RouterVendor,
    },
    
    /// Router configuration was applied
    RouterConfigurationApplied {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Router identifier
        router_id: RouterId,
        /// Snapshot of the configuration
        configuration: RouterConfigSnapshot,
        /// How the configuration was deployed
        deployment_method: DeploymentMethod,
    },
    
    /// VLAN was created on a switch
    VlanCreated {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Switch where VLAN was created
        switch_id: SwitchId,
        /// VLAN details
        vlan: Vlan,
    },
    
    /// Container network was created
    ContainerNetworkCreated {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Container network identifier
        network_id: ContainerNetworkId,
        /// Network name
        name: String,
        /// Optional VLAN ID for physical integration
        vlan_id: Option<VlanId>,
        /// Subnet for the container network
        subnet: IpNetwork,
    },
    
    /// Router provisioning started
    RouterProvisioningStarted {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Router identifier
        router_id: RouterId,
        /// Router vendor and OS
        vendor: RouterVendor,
    },
    
    /// Router provisioning completed
    RouterProvisioningCompleted {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Router identifier
        router_id: RouterId,
    },
    
    /// Router configuration failed
    RouterConfigurationFailed {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Router identifier
        router_id: RouterId,
        /// Failure reason
        reason: String,
    },
    
    /// Router maintenance scheduled
    RouterMaintenanceScheduled {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Router identifier
        router_id: RouterId,
        /// Maintenance window
        window: MaintenanceWindow,
    },
    
    /// Router configuration retry started
    RouterConfigurationRetryStarted {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Router identifier
        router_id: RouterId,
        /// Previous failure reason
        previous_failure: String,
    },
    
    /// Router interface added
    RouterInterfaceAdded {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Router identifier
        router_id: RouterId,
        /// Interface configuration
        interface: Interface,
    },
    
    /// Router OSPF configured
    RouterOspfConfigured {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Router identifier
        router_id: RouterId,
        /// OSPF process ID
        process_id: u32,
        /// Number of areas
        areas: u32,
    },
    
    /// Router BGP configured
    RouterBgpConfigured {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Router identifier
        router_id: RouterId,
        /// Local AS number
        local_as: u32,
        /// Number of neighbors
        neighbors: u32,
    },
    
    /// Router access list added
    RouterAccessListAdded {
        /// Event metadata with mandatory correlation/causation
        metadata: EventMetadata,
        /// Router identifier
        router_id: RouterId,
        /// ACL number
        acl_number: u16,
        /// Number of entries
        entries: u32,
    },
}

impl NetworkEvent {
    /// Create a RouterAdded event
    pub fn router_added(
        router_id: RouterId,
        name: String,
        vendor: RouterVendor,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Self {
        Self::RouterAdded {
            metadata: EventMetadata::new(
                AggregateId::from(router_id),
                correlation_id,
                causation_id,
            ),
            router_id,
            name,
            vendor,
        }
    }
    
    /// Create a RouterConfigurationApplied event
    pub fn router_configuration_applied(
        router_id: RouterId,
        configuration: RouterConfigSnapshot,
        deployment_method: DeploymentMethod,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Self {
        Self::RouterConfigurationApplied {
            metadata: EventMetadata::new(
                AggregateId::from(router_id),
                correlation_id,
                causation_id,
            ),
            router_id,
            configuration,
            deployment_method,
        }
    }
    
    /// Create a VlanCreated event
    pub fn vlan_created(
        switch_id: SwitchId,
        vlan_id: VlanId,
        name: String,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Self {
        Self::VlanCreated {
            metadata: EventMetadata::new(
                AggregateId::from(switch_id),
                correlation_id,
                causation_id,
            ),
            switch_id,
            vlan: Vlan {
                id: vlan_id,
                name,
                tagged_ports: vec![],
                untagged_ports: vec![],
            },
        }
    }
    
    /// Create a ContainerNetworkCreated event
    pub fn container_network_created(
        network_id: ContainerNetworkId,
        name: String,
        vlan_id: Option<VlanId>,
        subnet: IpNetwork,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Self {
        Self::ContainerNetworkCreated {
            metadata: EventMetadata::new(
                AggregateId::from(network_id),
                correlation_id,
                causation_id,
            ),
            network_id,
            name,
            vlan_id,
            subnet,
        }
    }
}

/// Router vendor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouterVendor {
    /// Cisco routers
    Cisco {
        /// Operating system version
        os: CiscoOs,
    },
    /// Juniper routers
    Juniper {
        /// JunOS version
        os: JunosVersion,
    },
    /// VyOS routers
    Vyos {
        /// VyOS version
        version: String,
    },
    /// Mikrotik routers
    Mikrotik {
        /// RouterOS version
        os: RouterOsVersion,
    },
}

/// Cisco operating systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CiscoOs {
    /// IOS 15.7
    Ios15_7,
    /// IOS XE
    IosXe(String),
    /// NX-OS
    NxOs(String),
}

/// Juniper OS versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JunosVersion(pub String);

/// Mikrotik RouterOS versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterOsVersion(pub String);

/// Router configuration snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfigSnapshot {
    /// Configured interfaces
    pub interfaces: Vec<Interface>,
    /// Routing protocols
    pub routing_protocols: Vec<RoutingProtocol>,
    /// Access control lists
    pub access_lists: Vec<AccessList>,
    /// When configuration was captured
    pub timestamp: DateTime<Utc>,
}

/// Router interface
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interface {
    /// Interface name (e.g., "GigabitEthernet0/0")
    pub name: String,
    /// IP address if configured
    pub ip_address: Option<IpAddr>,
    /// Interface description
    pub description: Option<String>,
    /// Whether interface is enabled
    pub enabled: bool,
}

/// Routing protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingProtocol {
    /// OSPF configuration
    Ospf {
        /// Process ID
        process_id: u32,
        /// Areas
        areas: Vec<OspfArea>,
    },
    /// BGP configuration
    Bgp {
        /// AS number
        as_number: u32,
        /// Neighbors
        neighbors: Vec<BgpNeighbor>,
    },
}

/// OSPF area
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OspfArea {
    /// Area ID
    pub id: u32,
    /// Networks in this area
    pub networks: Vec<IpNetwork>,
}

/// BGP neighbor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BgpNeighbor {
    /// Neighbor IP
    pub address: IpAddr,
    /// Remote AS
    pub remote_as: u32,
}

/// Access control list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessList {
    /// ACL number or name
    pub id: String,
    /// ACL entries
    pub entries: Vec<AclEntry>,
}

/// ACL entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AclEntry {
    /// Permit or deny
    pub action: AclAction,
    /// Protocol
    pub protocol: String,
    /// Source
    pub source: IpNetwork,
    /// Destination
    pub destination: IpNetwork,
}

/// ACL action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AclAction {
    /// Permit traffic
    Permit,
    /// Deny traffic
    Deny,
}

/// How configuration was deployed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentMethod {
    /// Deployed via Nix
    Nix {
        /// Flake reference
        flake_ref: String,
    },
    /// Deployed via Ansible
    Ansible {
        /// Playbook used
        playbook: String,
    },
    /// Direct configuration
    Direct {
        /// Protocol used
        protocol: ManagementProtocol,
    },
}

/// Management protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManagementProtocol {
    /// SSH
    Ssh,
    /// NETCONF
    Netconf,
    /// REST API
    RestApi,
}

/// Maintenance window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaintenanceWindow {
    /// Start time
    pub start: DateTime<Utc>,
    /// End time
    pub end: DateTime<Utc>,
    /// Reason for maintenance
    pub reason: String,
}

/// VLAN configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vlan {
    /// VLAN ID
    pub id: VlanId,
    /// VLAN name
    pub name: String,
    /// Tagged ports
    pub tagged_ports: Vec<PortNumber>,
    /// Untagged ports
    pub untagged_ports: Vec<PortNumber>,
}

/// Event builder for consistent event creation
pub struct NetworkEventBuilder {
    correlation_id: Option<CorrelationId>,
    causation_id: Option<CausationId>,
}

impl NetworkEventBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            correlation_id: None,
            causation_id: None,
        }
    }
    
    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: CorrelationId) -> Self {
        self.correlation_id = Some(id);
        self
    }
    
    /// Set causation ID
    pub fn with_causation_id(mut self, id: CausationId) -> Self {
        self.causation_id = Some(id);
        self
    }
    
    /// Build RouterAdded event
    pub fn build_router_added(
        self,
        router_id: RouterId,
        name: String,
        vendor: RouterVendor,
    ) -> NetworkEvent {
        NetworkEvent::router_added(
            router_id,
            name,
            vendor,
            self.correlation_id.expect("correlation_id is mandatory"),
            self.causation_id.expect("causation_id is mandatory"),
        )
    }
}