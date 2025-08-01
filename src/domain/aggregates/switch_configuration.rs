//! Switch configuration aggregate
//! 
//! Manages switch configuration including VLANs, ports, spanning tree, and MAC addresses

use crate::domain::value_objects::*;
use crate::domain::events::NetworkEvent;
use crate::domain::errors::NetworkError;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Switch configuration aggregate root
pub struct SwitchConfiguration {
    id: SwitchId,
    name: String,
    model: SwitchModel,
    vlans: HashMap<VlanId, VlanConfig>,
    ports: HashMap<PortNumber, PortConfig>,
    mac_address_table: Vec<MacAddressEntry>,
    spanning_tree_config: Option<SpanningTreeConfig>,
    stack_config: Option<StackConfig>,
    version: u64,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl SwitchConfiguration {
    /// Create new switch configuration
    pub fn new(id: SwitchId, name: String, model: SwitchModel) -> Self {
        Self {
            id,
            name,
            model,
            vlans: HashMap::new(),
            ports: HashMap::new(),
            mac_address_table: Vec::new(),
            spanning_tree_config: None,
            stack_config: None,
            version: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Get switch ID
    pub fn id(&self) -> SwitchId {
        self.id
    }
    
    /// Get switch name
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// Get switch model
    pub fn model(&self) -> &SwitchModel {
        &self.model
    }
    
    /// Get version for optimistic locking
    pub fn version(&self) -> u64 {
        self.version
    }
    
    /// Get all VLANs
    pub fn vlans(&self) -> &HashMap<VlanId, VlanConfig> {
        &self.vlans
    }
    
    /// Get specific VLAN
    pub fn get_vlan(&self, vlan_id: &VlanId) -> Option<&VlanConfig> {
        self.vlans.get(vlan_id)
    }
    
    /// Get all ports
    pub fn ports(&self) -> &HashMap<PortNumber, PortConfig> {
        &self.ports
    }
    
    /// Get specific port
    pub fn get_port(&self, port: &PortNumber) -> Option<&PortConfig> {
        self.ports.get(port)
    }
    
    /// Get MAC address table
    pub fn mac_address_table(&self) -> &[MacAddressEntry] {
        &self.mac_address_table
    }
    
    /// Lookup MAC address
    pub fn lookup_mac_address(&self, mac: &MacAddress) -> Option<&MacAddressEntry> {
        self.mac_address_table.iter().find(|e| &e.mac_address == mac)
    }
    
    /// Get spanning tree configuration
    pub fn spanning_tree_config(&self) -> Option<&SpanningTreeConfig> {
        self.spanning_tree_config.as_ref()
    }
    
    /// Get stack configuration
    pub fn stack_config(&self) -> Option<&StackConfig> {
        self.stack_config.as_ref()
    }
    
    /// Create a new VLAN
    pub fn create_vlan(
        &mut self,
        vlan_id: VlanId,
        name: String,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<Vec<NetworkEvent>, NetworkError> {
        // Check if VLAN already exists
        if self.vlans.contains_key(&vlan_id) {
            return Err(NetworkError::General(format!("VLAN {} already exists", vlan_id.value())));
        }
        
        // Create VLAN configuration
        let vlan_config = VlanConfig {
            id: vlan_id,
            name: name.clone(),
            tagged_ports: Vec::new(),
            untagged_ports: Vec::new(),
        };
        
        self.vlans.insert(vlan_id, vlan_config);
        self.version += 1;
        self.updated_at = Utc::now();
        
        // Create event
        let event = NetworkEvent::vlan_created(
            self.id,
            vlan_id,
            name,
            correlation_id,
            causation_id,
        );
        
        Ok(vec![event])
    }
    
    /// Configure a port
    pub fn configure_port(
        &mut self,
        port_config: PortConfig,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<Vec<NetworkEvent>, NetworkError> {
        let port_num = port_config.number;
        
        // Update VLAN assignments
        if port_config.mode == PortMode::Trunk {
            // For trunk ports, add to tagged ports of allowed VLANs
            for vlan_id in &port_config.allowed_vlans {
                if let Some(vlan) = self.vlans.get_mut(vlan_id) {
                    if !vlan.tagged_ports.contains(&port_num) {
                        vlan.tagged_ports.push(port_num);
                    }
                }
            }
        }
        
        self.ports.insert(port_num, port_config);
        self.version += 1;
        self.updated_at = Utc::now();
        
        // Create event
        let event = NetworkEvent::SwitchPortConfigured {
            metadata: crate::domain::events::EventMetadata::new(
                AggregateId::from(self.id),
                correlation_id,
                causation_id,
            ),
            switch_id: self.id,
            port_number: port_num,
        };
        
        Ok(vec![event])
    }
    
    /// Assign VLAN to port
    pub fn assign_vlan_to_port(
        &mut self,
        port_num: PortNumber,
        vlan_id: VlanId,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<Vec<NetworkEvent>, NetworkError> {
        // Check port exists
        let port = self.ports.get_mut(&port_num)
            .ok_or_else(|| NetworkError::General(format!("Port {} not found", port_num.value())))?;
        
        // Check VLAN exists
        let vlan = self.vlans.get_mut(&vlan_id)
            .ok_or_else(|| NetworkError::General(format!("VLAN {} not found", vlan_id.value())))?;
        
        // Update port allowed VLANs
        if !port.allowed_vlans.contains(&vlan_id) {
            port.allowed_vlans.push(vlan_id);
        }
        
        // Update VLAN port assignments
        match port.mode {
            PortMode::Access => {
                if !vlan.untagged_ports.contains(&port_num) {
                    vlan.untagged_ports.push(port_num);
                }
            }
            PortMode::Trunk => {
                if !vlan.tagged_ports.contains(&port_num) {
                    vlan.tagged_ports.push(port_num);
                }
            }
        }
        
        self.version += 1;
        self.updated_at = Utc::now();
        
        // Create event
        let event = NetworkEvent::VlanAssignedToPort {
            metadata: crate::domain::events::EventMetadata::new(
                AggregateId::from(self.id),
                correlation_id,
                causation_id,
            ),
            switch_id: self.id,
            port_number: port_num,
            vlan_id,
        };
        
        Ok(vec![event])
    }
    
    /// Configure spanning tree
    pub fn configure_spanning_tree(
        &mut self,
        config: SpanningTreeConfig,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<Vec<NetworkEvent>, NetworkError> {
        self.spanning_tree_config = Some(config);
        self.version += 1;
        self.updated_at = Utc::now();
        
        // Create event
        let event = NetworkEvent::SpanningTreeConfigured {
            metadata: crate::domain::events::EventMetadata::new(
                AggregateId::from(self.id),
                correlation_id,
                causation_id,
            ),
            switch_id: self.id,
        };
        
        Ok(vec![event])
    }
    
    /// Add MAC address entry
    pub fn add_mac_address_entry(
        &mut self,
        mac_address: MacAddress,
        port: PortNumber,
        vlan: VlanId,
        mac_type: MacAddressType,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<Vec<NetworkEvent>, NetworkError> {
        // Check if MAC already exists
        if self.mac_address_table.iter().any(|e| e.mac_address == mac_address) {
            return Err(NetworkError::General(format!("MAC address {} already exists", mac_address)));
        }
        
        let entry = MacAddressEntry {
            mac_address,
            port,
            vlan,
            mac_type,
            last_seen: Utc::now(),
        };
        
        self.mac_address_table.push(entry);
        self.version += 1;
        self.updated_at = Utc::now();
        
        // Create event
        let event = NetworkEvent::MacAddressLearned {
            metadata: crate::domain::events::EventMetadata::new(
                AggregateId::from(self.id),
                correlation_id,
                causation_id,
            ),
            switch_id: self.id,
            mac_address,
            port,
            vlan,
        };
        
        Ok(vec![event])
    }
    
    /// Configure switch stack
    pub fn configure_stack(
        &mut self,
        config: StackConfig,
        correlation_id: CorrelationId,
        causation_id: CausationId,
    ) -> Result<Vec<NetworkEvent>, NetworkError> {
        self.stack_config = Some(config);
        self.version += 1;
        self.updated_at = Utc::now();
        
        // Create event
        let event = NetworkEvent::SwitchStackConfigured {
            metadata: crate::domain::events::EventMetadata::new(
                AggregateId::from(self.id),
                correlation_id,
                causation_id,
            ),
            switch_id: self.id,
        };
        
        Ok(vec![event])
    }
    
    /// Validate switch configuration
    pub fn validate(&self) -> ConfigurationValidation {
        let mut validation = ConfigurationValidation {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };
        
        // Check for VLANs without ports
        for (vlan_id, vlan) in &self.vlans {
            if vlan.tagged_ports.is_empty() && vlan.untagged_ports.is_empty() {
                validation.warnings.push(format!("VLAN {} has no assigned ports", vlan_id.value()));
            }
        }
        
        // Check for ports without VLANs
        for (port_num, port) in &self.ports {
            if port.allowed_vlans.is_empty() && port.mode == PortMode::Access {
                validation.warnings.push(format!("Access port {} has no VLAN assigned", port_num.value()));
            }
        }
        
        // Check spanning tree configuration
        if let Some(stp) = &self.spanning_tree_config {
            // Verify root guard ports exist
            for port in &stp.root_guard_ports {
                if !self.ports.contains_key(port) {
                    validation.errors.push(format!("Root guard port {} does not exist", port.value()));
                    validation.is_valid = false;
                }
            }
            
            // Verify portfast ports exist
            for port in &stp.portfast_ports {
                if !self.ports.contains_key(port) {
                    validation.errors.push(format!("Portfast port {} does not exist", port.value()));
                    validation.is_valid = false;
                }
            }
        }
        
        validation
    }
}

/// VLAN configuration
#[derive(Debug, Clone, PartialEq)]
pub struct VlanConfig {
    /// VLAN ID
    pub id: VlanId,
    /// VLAN name
    pub name: String,
    /// Tagged ports (trunk)
    pub tagged_ports: Vec<PortNumber>,
    /// Untagged ports (access)
    pub untagged_ports: Vec<PortNumber>,
}

/// Port configuration
#[derive(Debug, Clone, PartialEq)]
pub struct PortConfig {
    /// Port number
    pub number: PortNumber,
    /// Port description
    pub description: Option<String>,
    /// Port mode
    pub mode: PortMode,
    /// Port speed
    pub speed: PortSpeed,
    /// Is port enabled
    pub enabled: bool,
    /// Allowed VLANs
    pub allowed_vlans: Vec<VlanId>,
}

/// Port mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortMode {
    /// Access port (single VLAN)
    Access,
    /// Trunk port (multiple VLANs)
    Trunk,
}

/// MAC address table entry
#[derive(Debug, Clone, PartialEq)]
pub struct MacAddressEntry {
    /// MAC address
    pub mac_address: MacAddress,
    /// Port where MAC was learned
    pub port: PortNumber,
    /// VLAN where MAC was learned
    pub vlan: VlanId,
    /// Entry type
    pub mac_type: MacAddressType,
    /// Last seen timestamp
    pub last_seen: DateTime<Utc>,
}

/// MAC address type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MacAddressType {
    /// Dynamically learned
    Dynamic,
    /// Statically configured
    Static,
}

/// Spanning tree configuration
#[derive(Debug, Clone, PartialEq)]
pub struct SpanningTreeConfig {
    /// STP mode
    pub mode: StpMode,
    /// Bridge priority
    pub priority: Option<u16>,
    /// Ports with root guard
    pub root_guard_ports: Vec<PortNumber>,
    /// Ports with portfast
    pub portfast_ports: Vec<PortNumber>,
}

/// Spanning tree mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StpMode {
    /// Per-VLAN Spanning Tree
    Pvst,
    /// Rapid Per-VLAN Spanning Tree
    RapidPvst,
    /// Multiple Spanning Tree
    Mst,
}

/// Switch stack configuration
#[derive(Debug, Clone, PartialEq)]
pub struct StackConfig {
    /// Stack members
    pub stack_members: Vec<StackMember>,
    /// Stack bandwidth
    pub stack_bandwidth: StackBandwidth,
}

/// Stack member
#[derive(Debug, Clone, PartialEq)]
pub struct StackMember {
    /// Member number
    pub number: u8,
    /// Priority (higher wins master election)
    pub priority: u8,
    /// MAC address
    pub mac_address: MacAddress,
}

/// Stack bandwidth
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackBandwidth {
    /// Half bandwidth
    Half,
    /// Full bandwidth
    Full,
}

/// Switch model
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchModel {
    /// Cisco Catalyst 2960-X
    Cisco2960X,
    /// Cisco Catalyst 3850
    Cisco3850,
    /// Cisco Catalyst 9300
    Cisco9300,
    /// Cisco Nexus 9000
    Nexus9000,
}

/// Configuration validation result
#[derive(Debug, Clone)]
pub struct ConfigurationValidation {
    /// Is configuration valid
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}