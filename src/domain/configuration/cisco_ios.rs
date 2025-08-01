//! Cisco IOS configuration generator

use crate::domain::configuration::ConfigurationGenerator;
use crate::domain::aggregates::router_configuration::*;
use crate::domain::errors::NetworkError;
use crate::domain::events::{CiscoOs, RouterVendor};
use std::fmt::Write;

/// Cisco IOS configuration generator options
pub struct CiscoGeneratorOptions {
    /// Whether to sanitize passwords in output
    pub sanitize_passwords: bool,
    /// Whether to include timestamp comments
    pub include_timestamps: bool,
}

impl Default for CiscoGeneratorOptions {
    fn default() -> Self {
        Self {
            sanitize_passwords: false,
            include_timestamps: true,
        }
    }
}

/// Cisco IOS configuration generator
pub struct CiscoIosGenerator {
    options: CiscoGeneratorOptions,
}

impl CiscoIosGenerator {
    /// Create a new generator with default options
    pub fn new() -> Self {
        Self {
            options: Default::default(),
        }
    }
    
    /// Create a generator with custom options
    pub fn with_options(options: CiscoGeneratorOptions) -> Self {
        Self { options }
    }
    
    /// Convert IP network to Cisco wildcard mask
    fn to_wildcard_mask(prefix_len: u8) -> String {
        let mask = !((1u32 << (32 - prefix_len)) - 1);
        let wildcard = !mask;
        format!(
            "{}.{}.{}.{}",
            (wildcard >> 24) & 0xFF,
            (wildcard >> 16) & 0xFF,
            (wildcard >> 8) & 0xFF,
            wildcard & 0xFF
        )
    }
    
    /// Convert prefix length to subnet mask
    fn to_subnet_mask(prefix_len: u8) -> String {
        let mask = !((1u32 << (32 - prefix_len)) - 1);
        format!(
            "{}.{}.{}.{}",
            (mask >> 24) & 0xFF,
            (mask >> 16) & 0xFF,
            (mask >> 8) & 0xFF,
            mask & 0xFF
        )
    }
    
    /// Generate header section
    fn generate_header(&self, config: &RouterConfiguration, output: &mut String) -> Result<(), NetworkError> {
        writeln!(output, "!")?;
        if self.options.include_timestamps {
            writeln!(output, "! Generated at: {}", chrono::Utc::now())?;
        }
        writeln!(output, "! Router: {}", config.name())?;
        writeln!(output, "!")?;
        writeln!(output, "version 15.7")?;
        writeln!(output, "service timestamps debug datetime msec")?;
        writeln!(output, "service timestamps log datetime msec")?;
        writeln!(output, "no service password-encryption")?;
        writeln!(output, "!")?;
        writeln!(output, "hostname {}", config.name())?;
        writeln!(output, "!")?;
        writeln!(output, "boot-start-marker")?;
        writeln!(output, "boot-end-marker")?;
        writeln!(output, "!")?;
        
        Ok(())
    }
    
    /// Generate global configuration
    fn generate_global(&self, config: &RouterConfiguration, output: &mut String) -> Result<(), NetworkError> {
        writeln!(output, "!")?;
        writeln!(output, "no aaa new-model")?;
        writeln!(output, "!")?;
        writeln!(output, "ip cef")?;
        writeln!(output, "no ip domain lookup")?;
        writeln!(output, "!")?;
        
        // Add IP NBAR for IOS XE
        match config.vendor() {
            RouterVendor::Cisco { os: CiscoOs::IosXe17_3 } => {
                writeln!(output, "ip nbar protocol-discovery")?;
                writeln!(output, "!")?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Generate interface configuration
    fn generate_interfaces(&self, config: &RouterConfiguration, output: &mut String) -> Result<(), NetworkError> {
        for interface in config.interfaces() {
            writeln!(output, "!")?;
            writeln!(output, "interface {}", interface.name)?;
            
            if let Some(desc) = &interface.description {
                writeln!(output, " description {}", desc)?;
            }
            
            if let Some(ip_net) = &interface.ip_address {
                let ip = ip_net.inner();
                let mask = Self::to_subnet_mask(ip.prefix());
                writeln!(output, " ip address {} {}", ip.ip(), mask)?;
            }
            
            if interface.enabled {
                writeln!(output, " no shutdown")?;
            } else {
                writeln!(output, " shutdown")?;
            }
        }
        
        Ok(())
    }
    
    /// Generate OSPF configuration
    fn generate_ospf(&self, ospf: &OspfConfig, output: &mut String) -> Result<(), NetworkError> {
        writeln!(output, "!")?;
        writeln!(output, "router ospf {}", ospf.process_id)?;
        writeln!(output, " router-id {}", ospf.router_id)?;
        
        for area in &ospf.areas {
            for network in &area.networks {
                let net = network.inner();
                let wildcard = Self::to_wildcard_mask(net.prefix());
                writeln!(output, " network {} {} area {}", 
                    net.network(), wildcard, area.area_id)?;
            }
        }
        
        Ok(())
    }
    
    /// Generate BGP configuration
    fn generate_bgp(&self, bgp: &BgpConfig, output: &mut String) -> Result<(), NetworkError> {
        writeln!(output, "!")?;
        writeln!(output, "router bgp {}", bgp.local_as)?;
        writeln!(output, " bgp router-id {}", bgp.router_id)?;
        writeln!(output, " bgp log-neighbor-changes")?;
        
        for neighbor in &bgp.neighbors {
            writeln!(output, " neighbor {} remote-as {}", 
                neighbor.address, neighbor.remote_as)?;
            
            if let Some(desc) = &neighbor.description {
                writeln!(output, " neighbor {} description {}", 
                    neighbor.address, desc)?;
            }
            
            if let Some(password) = &neighbor.password {
                if self.options.sanitize_passwords {
                    writeln!(output, " neighbor {} password <REDACTED>", 
                        neighbor.address)?;
                } else {
                    writeln!(output, " neighbor {} password {}", 
                        neighbor.address, password)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Generate access list configuration
    fn generate_access_lists(&self, config: &RouterConfiguration, output: &mut String) -> Result<(), NetworkError> {
        for acl in config.access_lists() {
            writeln!(output, "!")?;
            writeln!(output, "ip access-list extended {}", acl.number)?;
            
            if let Some(name) = &acl.name {
                writeln!(output, " remark {}", name)?;
            }
            
            for entry in &acl.entries {
                let action = match entry.action {
                    AclAction::Permit => "permit",
                    AclAction::Deny => "deny",
                };
                
                let protocol = match entry.protocol {
                    Protocol::Ip => "ip",
                    Protocol::Tcp => "tcp",
                    Protocol::Udp => "udp",
                    Protocol::Icmp => "icmp",
                };
                
                let src_net = entry.source.inner();
                let src_wildcard = Self::to_wildcard_mask(src_net.prefix());
                
                let dst = if entry.destination.inner().prefix() == 0 {
                    "any".to_string()
                } else {
                    let dst_net = entry.destination.inner();
                    let dst_wildcard = Self::to_wildcard_mask(dst_net.prefix());
                    format!("{} {}", dst_net.network(), dst_wildcard)
                };
                
                let mut line = format!(" {} {} {} {} {} {}", 
                    entry.sequence, action, protocol, 
                    src_net.network(), src_wildcard, dst
                );
                
                if let Some(ports) = &entry.ports {
                    if ports.start == ports.end {
                        line.push_str(&format!(" eq {}", ports.start));
                    } else {
                        line.push_str(&format!(" range {} {}", ports.start, ports.end));
                    }
                }
                
                writeln!(output, "{}", line)?;
            }
        }
        
        Ok(())
    }
    
    /// Generate routing protocol configuration
    fn generate_routing_protocols(&self, config: &RouterConfiguration, output: &mut String) -> Result<(), NetworkError> {
        for protocol in config.routing_protocols() {
            match protocol {
                RoutingProtocolConfig::Ospf(ospf) => self.generate_ospf(ospf, output)?,
                RoutingProtocolConfig::Bgp(bgp) => self.generate_bgp(bgp, output)?,
                RoutingProtocolConfig::Static(_) => {
                    // TODO: Implement static routes
                }
            }
        }
        
        Ok(())
    }
    
    /// Generate footer
    fn generate_footer(&self, output: &mut String) -> Result<(), NetworkError> {
        writeln!(output, "!")?;
        writeln!(output, "line con 0")?;
        writeln!(output, " logging synchronous")?;
        writeln!(output, "line aux 0")?;
        writeln!(output, "line vty 0 4")?;
        writeln!(output, " login")?;
        writeln!(output, " transport input ssh")?;
        writeln!(output, "!")?;
        writeln!(output, "end")?;
        
        Ok(())
    }
}

impl ConfigurationGenerator for CiscoIosGenerator {
    type Input = RouterConfiguration;
    type Output = String;
    type Error = NetworkError;
    
    fn generate(&self, config: &Self::Input) -> Result<Self::Output, Self::Error> {
        let mut output = String::new();
        
        // Generate sections in order
        self.generate_header(config, &mut output)?;
        self.generate_global(config, &mut output)?;
        self.generate_interfaces(config, &mut output)?;
        self.generate_routing_protocols(config, &mut output)?;
        self.generate_access_lists(config, &mut output)?;
        self.generate_footer(&mut output)?;
        
        Ok(output)
    }
    
    fn validate(&self, output: &Self::Output) -> Result<(), Self::Error> {
        // Basic validation
        if !output.contains("hostname") {
            return Err(NetworkError::General("Missing hostname configuration".to_string()));
        }
        
        if !output.ends_with("end\n") {
            return Err(NetworkError::General("Configuration does not end properly".to_string()));
        }
        
        // Check for double exclamation marks
        if output.contains("!!") {
            return Err(NetworkError::General("Invalid syntax: double exclamation marks".to_string()));
        }
        
        Ok(())
    }
}

impl Default for CiscoIosGenerator {
    fn default() -> Self {
        Self::new()
    }
}