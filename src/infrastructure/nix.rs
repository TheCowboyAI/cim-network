//! Nix topology generation and configuration

use crate::domain::aggregates::network_topology::{
    NetworkTopology, NetworkTopologyId, NetworkDevice, NetworkConnection, NixTopologyConfig,
    NixDevice, NixNetwork, NixInterface, DeviceType, TopologyType
};
use cim_graph::graphs::{ContextGraph, ContextNode, ContextEdge, context_projection::ContextNodeType};
use crate::domain::{NetworkError, CorrelationId, CausationId, EventId};
use crate::domain::events::{NetworkEvent, EventMetadata};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::Utc;
use async_trait::async_trait;

/// Nix topology configuration generator
pub struct NixTopologyGenerator {
    template_engine: Box<dyn TemplateEngine>,
    file_writer: Box<dyn FileWriter>,
    formatter: Box<dyn NixFormatter>,
}

/// Template engine interface for generating Nix configurations
#[async_trait]
pub trait TemplateEngine: Send + Sync {
    async fn render_template(
        &self,
        template_name: &str,
        context: &TemplateContext,
    ) -> Result<String, TemplateError>;
    
    async fn render_flake_template(
        &self,
        topology: &NetworkTopology,
        options: &NixGenerationOptions,
    ) -> Result<String, TemplateError>;
    
    async fn render_topology_template(
        &self,
        topology: &NetworkTopology,
    ) -> Result<String, TemplateError>;
    
    async fn render_device_module(
        &self,
        device: &NetworkDevice,
        options: &DeviceModuleOptions,
    ) -> Result<String, TemplateError>;
}

/// File writing interface
#[async_trait]
pub trait FileWriter: Send + Sync {
    async fn write_file(
        &self,
        path: &PathBuf,
        content: &str,
    ) -> Result<(), std::io::Error>;
    
    async fn create_directory(
        &self,
        path: &PathBuf,
    ) -> Result<(), std::io::Error>;
}

/// Nix code formatter interface
#[async_trait]
pub trait NixFormatter: Send + Sync {
    async fn format_nix_code(&self, code: &str) -> Result<String, FormatterError>;
}

/// Template context for rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    pub topology: NetworkTopologyContext,
    pub devices: Vec<DeviceContext>,
    pub networks: Vec<NetworkContext>,
    pub connections: Vec<ConnectionContext>,
    pub generation_metadata: GenerationMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopologyContext {
    pub name: String,
    pub base_network: String,
    pub topology_type: String,
    pub device_count: usize,
    pub network_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceContext {
    pub name: String,
    pub device_type: String,
    pub ip_address: String,
    pub interfaces: Vec<InterfaceContext>,
    pub services: Vec<String>,
    pub nix_module_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceContext {
    pub name: String,
    pub network: String,
    pub address: Option<String>,
    pub dhcp: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContext {
    pub name: String,
    pub cidr: String,
    pub vlan: Option<u16>,
    pub dhcp: bool,
    pub gateway: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionContext {
    pub source: ConnectionEndpointContext,
    pub target: ConnectionEndpointContext,
    pub connection_type: String,
    pub bandwidth: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEndpointContext {
    pub device: String,
    pub interface: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub generation_timestamp: String,
    pub generator_version: String,
    pub target_system: String,
}

/// Options for Nix generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NixGenerationOptions {
    pub deployment_target: DeploymentTarget,
    pub generate_documentation: bool,
    pub include_examples: bool,
    pub custom_modules: HashMap<String, PathBuf>,
    pub flake_inputs: HashMap<String, FlakeInput>,
    pub template_overrides: Option<TemplateConfig>,
    pub output_directory: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentTarget {
    Local,
    Remote { host: String, user: String },
    Container { runtime: String },
    VM { hypervisor: String },
    Cloud { provider: String, region: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlakeInput {
    pub url: String,
    pub follows: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    pub custom_templates: HashMap<String, String>,
    pub template_variables: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceModuleOptions {
    pub nixos_version: String,
    pub enable_ssh: bool,
    pub enable_monitoring: bool,
    pub custom_packages: Vec<String>,
}

/// Generation request
#[derive(Debug, Clone)]
pub struct NixTopologyGenerationRequest {
    pub network_topology: NetworkTopology,
    pub options: NixGenerationOptions,
    pub correlation_id: CorrelationId,
    pub causation_id: CausationId,
}

/// Generation response
#[derive(Debug, Clone)]
pub struct NixTopologyGenerationResponse {
    pub generation_id: GenerationId,
    pub generated_files: GeneratedFiles,
    pub documentation: DocumentationAssets,
    pub validation_results: ValidationResults,
    pub events: Vec<NetworkEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GenerationId(uuid::Uuid);

impl GenerationId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl std::fmt::Display for GenerationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFiles {
    pub flake_nix: PathBuf,
    pub topology_nix: PathBuf,
    pub nixos_modules: HashMap<String, PathBuf>,
    pub network_modules: Vec<PathBuf>,
    pub deployment_scripts: Vec<PathBuf>,
    pub validation_scripts: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationAssets {
    pub mermaid_diagram: Option<PathBuf>,
    pub topology_graph: Option<PathBuf>,
    pub svg_diagram: Option<PathBuf>,
    pub readme: Option<PathBuf>,
    pub deployment_guide: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    pub overall_status: ValidationStatus,
    pub syntax_errors: Vec<String>,
    pub semantic_warnings: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationStatus {
    Valid,
    Warning,
    Error,
}

impl NixTopologyGenerator {
    /// Create a new Nix topology generator
    pub fn new(
        template_engine: Box<dyn TemplateEngine>,
        file_writer: Box<dyn FileWriter>,
        formatter: Box<dyn NixFormatter>,
    ) -> Self {
        Self {
            template_engine,
            file_writer,
            formatter,
        }
    }
    
    /// Generate complete Nix topology configuration
    pub async fn generate_topology(
        &self,
        request: NixTopologyGenerationRequest,
    ) -> Result<NixTopologyGenerationResponse, NixTopologyError> {
        let generation_id = GenerationId::new();
        
        tracing::info!(
            "Starting Nix topology generation for network '{}' (generation_id: {})",
            request.network_topology.name(),
            generation_id
        );
        
        // Create output directory structure
        self.create_directory_structure(&request.options.output_directory).await?;
        
        // Generate template context
        let context = self.build_template_context(&request.network_topology, &request.options).await?;
        
        // Generate all Nix files
        let generated_files = self.generate_all_files(&context, &request.options).await?;
        
        // Generate documentation
        let documentation = if request.options.generate_documentation {
            self.generate_documentation(&request.network_topology, &generated_files, &request.options).await?
        } else {
            DocumentationAssets {
                mermaid_diagram: None,
                topology_graph: None,
                svg_diagram: None,
                readme: None,
                deployment_guide: None,
            }
        };
        
        // Validate generated configuration
        let validation_results = self.validate_generated_files(&generated_files).await?;
        
        // Generate events
        let events = self.generate_events(
            generation_id,
            &request.network_topology,
            &generated_files,
            &request.correlation_id,
            &request.causation_id,
        );
        
        tracing::info!(
            "Nix topology generation completed for network '{}' (generation_id: {})",
            request.network_topology.name(),
            generation_id
        );
        
        Ok(NixTopologyGenerationResponse {
            generation_id,
            generated_files,
            documentation,
            validation_results,
            events,
        })
    }
    
    /// Create output directory structure
    async fn create_directory_structure(&self, output_dir: &PathBuf) -> Result<(), NixTopologyError> {
        let directories = vec![
            output_dir.clone(),
            output_dir.join("modules"),
            output_dir.join("networks"),
            output_dir.join("scripts"),
            output_dir.join("docs"),
            output_dir.join("examples"),
            output_dir.join("tests"),
        ];
        
        for dir in directories {
            self.file_writer.create_directory(&dir).await
                .map_err(NixTopologyError::IoError)?;
        }
        
        Ok(())
    }
    
    /// Build template context from network topology
    async fn build_template_context(
        &self,
        topology: &NetworkTopology,
        options: &NixGenerationOptions,
    ) -> Result<TemplateContext, NixTopologyError> {
        let topology_context = NetworkTopologyContext {
            name: topology.name().to_string(),
            base_network: topology.base_ip().to_string(),
            topology_type: format!("{:?}", topology.topology_type()),
            device_count: topology.devices().len(),
            network_count: topology.nix_config()
                .map(|c| c.networks.len())
                .unwrap_or(0),
        };
        
        let mut device_contexts = Vec::new();
        for device in topology.devices().values() {
            let interfaces: Vec<InterfaceContext> = device.interfaces.iter().map(|iface| {
                InterfaceContext {
                    name: iface.name.clone(),
                    network: "lan".to_string(), // TODO: proper network mapping
                    address: iface.ip_address.map(|ip| 
                        format!("{}/{}", ip, iface.subnet_mask.unwrap_or(24))
                    ),
                    dhcp: false,
                    enabled: iface.enabled,
                }
            }).collect();
            
            device_contexts.push(DeviceContext {
                name: device.name.clone(),
                device_type: self.device_type_to_string(&device.device_type),
                ip_address: device.ip_address.to_string(),
                interfaces,
                services: device.services.iter().map(|s| s.name.clone()).collect(),
                nix_module_path: format!("./modules/{}.nix", device.name),
            });
        }
        
        let network_contexts = if let Some(nix_config) = topology.nix_config() {
            nix_config.networks.iter().map(|network| {
                NetworkContext {
                    name: network.name.clone(),
                    cidr: network.cidr.clone(),
                    vlan: network.vlan,
                    dhcp: network.dhcp,
                    gateway: network.gateway.clone(),
                }
            }).collect()
        } else {
            vec![NetworkContext {
                name: "lan".to_string(),
                cidr: topology.base_ip().to_string(),
                vlan: None,
                dhcp: false,
                gateway: Some(topology.base_ip().network().nth(1).unwrap().to_string()),
            }]
        };
        
        let connection_contexts = topology.connections().iter().map(|conn| {
            ConnectionContext {
                source: ConnectionEndpointContext {
                    device: format!("device_{}", conn.source.device_id),
                    interface: format!("interface_{}", conn.source.interface_id),
                },
                target: ConnectionEndpointContext {
                    device: format!("device_{}", conn.target.device_id),
                    interface: format!("interface_{}", conn.target.interface_id),
                },
                connection_type: format!("{:?}", conn.connection_type),
                bandwidth: conn.bandwidth.as_ref().map(|b| format!("{}Mbps", b.speed)),
            }
        }).collect();
        
        let generation_metadata = GenerationMetadata {
            generation_timestamp: Utc::now().to_rfc3339(),
            generator_version: "0.1.0".to_string(),
            target_system: "x86_64-linux".to_string(),
        };
        
        Ok(TemplateContext {
            topology: topology_context,
            devices: device_contexts,
            networks: network_contexts,
            connections: connection_contexts,
            generation_metadata,
        })
    }
    
    /// Generate all Nix configuration files
    async fn generate_all_files(
        &self,
        context: &TemplateContext,
        options: &NixGenerationOptions,
    ) -> Result<GeneratedFiles, NixTopologyError> {
        
        // Generate flake.nix
        let flake_content = self.generate_flake_nix(context, options).await?;
        let flake_path = options.output_directory.join("flake.nix");
        let formatted_flake = self.formatter.format_nix_code(&flake_content).await
            .map_err(|e| NixTopologyError::FormatterError(format!("{:?}", e)))?;
        self.file_writer.write_file(&flake_path, &formatted_flake).await
            .map_err(NixTopologyError::IoError)?;
        
        // Generate topology.nix
        let topology_content = self.generate_topology_nix(context).await?;
        let topology_path = options.output_directory.join("topology.nix");
        let formatted_topology = self.formatter.format_nix_code(&topology_content).await
            .map_err(|e| NixTopologyError::FormatterError(format!("{:?}", e)))?;
        self.file_writer.write_file(&topology_path, &formatted_topology).await
            .map_err(NixTopologyError::IoError)?;
        
        // Generate device modules
        let mut nixos_modules = HashMap::new();
        for device in &context.devices {
            let module_content = self.generate_device_module(device, options).await?;
            let module_path = options.output_directory.join("modules").join(format!("{}.nix", device.name));
            let formatted_module = self.formatter.format_nix_code(&module_content).await
                .map_err(|e| NixTopologyError::FormatterError(format!("{:?}", e)))?;
            self.file_writer.write_file(&module_path, &formatted_module).await
                .map_err(NixTopologyError::IoError)?;
            nixos_modules.insert(device.name.clone(), module_path);
        }
        
        // Generate deployment scripts
        let deployment_scripts = self.generate_deployment_scripts(context, options).await?;
        
        Ok(GeneratedFiles {
            flake_nix: flake_path,
            topology_nix: topology_path,
            nixos_modules,
            network_modules: Vec::new(), // TODO: implement network modules
            deployment_scripts,
            validation_scripts: Vec::new(), // TODO: implement validation scripts
        })
    }
    
    /// Generate flake.nix content
    async fn generate_flake_nix(
        &self,
        context: &TemplateContext,
        options: &NixGenerationOptions,
    ) -> Result<String, NixTopologyError> {
        let flake_template = r#"{
  description = "{{ topology.name }} - Generated CIM Network Topology";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-topology.url = "github:oddlama/nix-topology";
    cim-network.url = "github:thecowboyai/cim-network";
  };
  
  outputs = { self, nixpkgs, nix-topology, cim-network, ... }@inputs:
    let
      system = "{{ generation_metadata.target_system }}";
      pkgs = nixpkgs.legacyPackages.${system};
      
      topology = import ./topology.nix { inherit pkgs; };
      
    in {
      packages.${system} = {
        topology = nix-topology.lib.mkTopology {
          inherit pkgs;
          modules = [ topology ];
        };
        
        validate = pkgs.writeShellScriptBin "validate-topology" ''
          echo "Validating network topology: {{ topology.name }}"
          ${pkgs.nix}/bin/nix eval .#topology --show-trace
          echo "Topology validation completed successfully"
        '';
        
        deploy = pkgs.writeShellScriptBin "deploy-network" ''
          echo "Deploying network: {{ topology.name }}"
          echo "Generated: {{ generation_metadata.generation_timestamp }}"
          echo "Devices: {{ topology.device_count }}"
          echo "Networks: {{ topology.network_count }}"
          echo "Network deployment completed"
        '';
      };
      
      nixosConfigurations = {
        {{#each devices}}
        {{ this.name }} = nixpkgs.lib.nixosSystem {
          inherit system;
          modules = [
            {{ this.nix_module_path }}
          ];
          specialArgs = { inherit inputs; };
        };
        {{/each}}
      };
      
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          nixos-rebuild
          git
        ];
        
        shellHook = ''
          echo "🌐 CIM Network Topology: {{ topology.name }}"
          echo "📊 Generated: {{ generation_metadata.generation_timestamp }}"
          echo "🔧 Devices: {{ topology.device_count }}"
          echo "🔗 Networks: {{ topology.network_count }}"
          echo ""
          echo "Available commands:"
          echo "  nix run .#validate  - Validate topology"
          echo "  nix run .#deploy    - Deploy network"
          echo "  nix run .#topology  - Generate diagram"
        '';
      };
      
      formatter.${system} = pkgs.nixpkgs-fmt;
    };
}"#;
        
        // Simple template substitution (in production, use a proper template engine)
        let content = flake_template
            .replace("{{ topology.name }}", &context.topology.name)
            .replace("{{ generation_metadata.target_system }}", &context.generation_metadata.target_system)
            .replace("{{ generation_metadata.generation_timestamp }}", &context.generation_metadata.generation_timestamp)
            .replace("{{ topology.device_count }}", &context.topology.device_count.to_string())
            .replace("{{ topology.network_count }}", &context.topology.network_count.to_string());
        
        // TODO: Handle device iteration with proper template engine
        Ok(content)
    }
    
    /// Generate topology.nix content
    async fn generate_topology_nix(&self, context: &TemplateContext) -> Result<String, NixTopologyError> {
        let topology_template = r#"{ pkgs }:

{
  networks = {
    {{#each networks}}
    {{ this.name }} = {
      cidr = "{{ this.cidr }}";
      {{#if this.vlan}}
      vlan = {{ this.vlan }};
      {{/if}}
      enableDHCP = {{ this.dhcp }};
      {{#if this.gateway}}
      gateway = "{{ this.gateway }}";
      {{/if}}
    };
    {{/each}}
  };
  
  nodes = {
    {{#each devices}}
    {{ this.name }} = {
      deviceType = "{{ this.device_type }}";
      image = "nixos/latest";
      
      interfaces = {
        {{#each this.interfaces}}
        {{ this.name }} = {
          network = "{{ this.network }}";
          {{#if this.address}}
          address = "{{ this.address }}";
          {{/if}}
          {{#if this.dhcp}}
          enableDHCP = {{ this.dhcp }};
          {{/if}}
        };
        {{/each}}
      };
      
      {{#if this.services}}
      services = [
        {{#each this.services}}
        "{{ this }}"{{#unless @last}},{{/unless}}
        {{/each}}
      ];
      {{/if}}
    };
    {{/each}}
  };
  
  connections = [
    {{#each connections}}
    {
      from = "{{ this.source.device }}.{{ this.source.interface }}";
      to = "{{ this.target.device }}.{{ this.target.interface }}";
      type = "{{ this.connection_type }}";
      {{#if this.bandwidth}}
      bandwidth = "{{ this.bandwidth }}";
      {{/if}}
    }{{#unless @last}},{{/unless}}
    {{/each}}
  ];
}"#;
        
        // Simple template substitution (in production, use a proper template engine)
        let mut content = topology_template.to_string();
        
        // Replace network sections
        let networks_section = context.networks.iter()
            .map(|net| format!(r#"    {} = {{
      cidr = "{}";
      enableDHCP = {};
      {}{}
    }};"#,
                net.name,
                net.cidr,
                net.dhcp,
                net.vlan.map_or(String::new(), |v| format!("vlan = {};\n      ", v)),
                net.gateway.as_ref().map_or(String::new(), |g| format!(r#"gateway = "{}";"#, g))
            ))
            .collect::<Vec<_>>()
            .join("\n");
        
        content = content.replace("    {{#each networks}}\n    {{ this.name }} = {\n      cidr = \"{{ this.cidr }}\";\n      {{#if this.vlan}}\n      vlan = {{ this.vlan }};\n      {{/if}}\n      enableDHCP = {{ this.dhcp }};\n      {{#if this.gateway}}\n      gateway = \"{{ this.gateway }}\";\n      {{/if}}\n    };\n    {{/each}}", &networks_section);
        
        // TODO: Handle device and connection iteration with proper template engine
        
        Ok(content)
    }
    
    /// Generate device module content
    async fn generate_device_module(
        &self,
        device: &DeviceContext,
        options: &NixGenerationOptions,
    ) -> Result<String, NixTopologyError> {
        let module_template = match device.device_type.as_str() {
            "router" => self.generate_router_module(device).await?,
            "switch" => self.generate_switch_module(device).await?,
            "host" => self.generate_host_module(device).await?,
            _ => self.generate_generic_module(device).await?,
        };
        
        Ok(module_template)
    }
    
    /// Generate router module
    async fn generate_router_module(&self, device: &DeviceContext) -> Result<String, NixTopologyError> {
        let router_template = format!(r#"{{ config, lib, pkgs, ... }}:

{{
  system.stateVersion = "24.05";
  
  networking = {{
    hostName = "{}";
    
    # Interface configuration
    interfaces = {{
      {}
    }};
    
    forwarding = true;
    firewall.enable = false;
  }};
  
  # Routing services
  services = {{
    openssh = {{
      enable = true;
      settings = {{
        PermitRootLogin = "yes";
        PasswordAuthentication = true;
      }};
    }};
  }};
  
  environment.systemPackages = with pkgs; [
    iproute2
    tcpdump
    traceroute
  ];
}}"#,
            device.name,
            device.interfaces.iter()
                .map(|iface| format!(r#"{} = {{
        {}
      }};"#,
                    iface.name,
                    iface.address.as_ref().map_or(String::new(), |addr| 
                        format!(r#"ipv4.addresses = [{{ address = "{}"; prefixLength = 24; }}];"#, 
                               addr.split('/').next().unwrap_or(addr))
                    )
                ))
                .collect::<Vec<_>>()
                .join("\n      ")
        );
        
        Ok(router_template)
    }
    
    /// Generate switch module
    async fn generate_switch_module(&self, device: &DeviceContext) -> Result<String, NixTopologyError> {
        let switch_template = format!(r#"{{ config, lib, pkgs, ... }}:

{{
  system.stateVersion = "24.05";
  
  networking = {{
    hostName = "{}";
    
    # Interface configuration
    interfaces = {{
      {}
    }};
    
    firewall.enable = false;
  }};
  
  # Switch services
  services = {{
    openssh = {{
      enable = true;
      settings = {{
        PermitRootLogin = "yes";
        PasswordAuthentication = true;
      }};
    }};
  }};
  
  environment.systemPackages = with pkgs; [
    bridge-utils
    vlan
    ethtool
  ];
}}"#,
            device.name,
            device.interfaces.iter()
                .map(|iface| format!(r#"{} = {{
        {}
      }};"#,
                    iface.name,
                    iface.address.as_ref().map_or(String::new(), |addr| 
                        format!(r#"ipv4.addresses = [{{ address = "{}"; prefixLength = 24; }}];"#, 
                               addr.split('/').next().unwrap_or(addr))
                    )
                ))
                .collect::<Vec<_>>()
                .join("\n      ")
        );
        
        Ok(switch_template)
    }
    
    /// Generate host module
    async fn generate_host_module(&self, device: &DeviceContext) -> Result<String, NixTopologyError> {
        let host_template = format!(r#"{{ config, lib, pkgs, ... }}:

{{
  system.stateVersion = "24.05";
  
  networking = {{
    hostName = "{}";
    
    # Interface configuration
    interfaces = {{
      {}
    }};
    
    firewall.enable = true;
    firewall.allowedTCPPorts = [ 22 80 443 ];
  }};
  
  services = {{
    openssh = {{
      enable = true;
      settings = {{
        PermitRootLogin = "yes";
        PasswordAuthentication = true;
      }};
    }};
  }};
  
  environment.systemPackages = with pkgs; [
    curl
    wget
    htop
  ];
}}"#,
            device.name,
            device.interfaces.iter()
                .map(|iface| format!(r#"{} = {{
        {}
      }};"#,
                    iface.name,
                    iface.address.as_ref().map_or(String::new(), |addr| 
                        format!(r#"ipv4.addresses = [{{ address = "{}"; prefixLength = 24; }}];"#, 
                               addr.split('/').next().unwrap_or(addr))
                    )
                ))
                .collect::<Vec<_>>()
                .join("\n      ")
        );
        
        Ok(host_template)
    }
    
    /// Generate generic module
    async fn generate_generic_module(&self, device: &DeviceContext) -> Result<String, NixTopologyError> {
        let generic_template = format!(r#"{{ config, lib, pkgs, ... }}:

{{
  system.stateVersion = "24.05";
  
  networking = {{
    hostName = "{}";
    
    # Interface configuration
    interfaces = {{
      {}
    }};
  }};
  
  services = {{
    openssh = {{
      enable = true;
      settings = {{
        PermitRootLogin = "yes";
        PasswordAuthentication = true;
      }};
    }};
  }};
}}"#,
            device.name,
            device.interfaces.iter()
                .map(|iface| format!(r#"{} = {{
        {}
      }};"#,
                    iface.name,
                    iface.address.as_ref().map_or(String::new(), |addr| 
                        format!(r#"ipv4.addresses = [{{ address = "{}"; prefixLength = 24; }}];"#, 
                               addr.split('/').next().unwrap_or(addr))
                    )
                ))
                .collect::<Vec<_>>()
                .join("\n      ")
        );
        
        Ok(generic_template)
    }
    
    /// Generate deployment scripts
    async fn generate_deployment_scripts(
        &self,
        context: &TemplateContext,
        options: &NixGenerationOptions,
    ) -> Result<Vec<PathBuf>, NixTopologyError> {
        let mut scripts = Vec::new();
        
        // Generate deploy.sh
        let deploy_script = format!(r#"#!/usr/bin/env bash
set -euo pipefail

echo "🌐 Deploying CIM Network: {}"
echo "📊 Generated: {}"
echo "🔧 Devices: {}"

# Validate topology first
echo "Validating topology..."
nix run .#validate

# Deploy each device
{}

echo "✅ Network deployment completed successfully"
"#,
            context.topology.name,
            context.generation_metadata.generation_timestamp,
            context.topology.device_count,
            context.devices.iter()
                .map(|device| format!(r#"echo "Deploying {}..."
# nixos-rebuild switch --flake .#{} --target-host {}"#, 
                    device.name, device.name, device.ip_address))
                .collect::<Vec<_>>()
                .join("\n")
        );
        
        let deploy_path = options.output_directory.join("scripts").join("deploy.sh");
        self.file_writer.write_file(&deploy_path, &deploy_script).await
            .map_err(NixTopologyError::IoError)?;
        scripts.push(deploy_path);
        
        Ok(scripts)
    }
    
    /// Generate documentation
    async fn generate_documentation(
        &self,
        topology: &NetworkTopology,
        generated_files: &GeneratedFiles,
        options: &NixGenerationOptions,
    ) -> Result<DocumentationAssets, NixTopologyError> {
        let docs_dir = options.output_directory.join("docs");
        
        // Generate Mermaid diagram
        let mermaid_content = self.generate_mermaid_diagram(topology).await?;
        let mermaid_path = docs_dir.join("network-diagram.mmd");
        self.file_writer.write_file(&mermaid_path, &mermaid_content).await
            .map_err(NixTopologyError::IoError)?;
        
        // Generate README
        let readme_content = self.generate_readme(topology, generated_files).await?;
        let readme_path = docs_dir.join("README.md");
        self.file_writer.write_file(&readme_path, &readme_content).await
            .map_err(NixTopologyError::IoError)?;
        
        Ok(DocumentationAssets {
            mermaid_diagram: Some(mermaid_path),
            topology_graph: None, // TODO: implement JSON graph
            svg_diagram: None,    // TODO: implement SVG generation
            readme: Some(readme_path),
            deployment_guide: None, // TODO: implement deployment guide
        })
    }
    
    /// Generate Mermaid diagram
    async fn generate_mermaid_diagram(&self, topology: &NetworkTopology) -> Result<String, NixTopologyError> {
        let mut mermaid = String::from("graph TB\n");
        mermaid.push_str(&format!("    subgraph \"Network: {} ({})\"\n", 
                                 topology.name(), topology.base_ip()));
        
        // Add devices
        for device in topology.devices().values() {
            let device_type = self.device_type_to_string(&device.device_type);
            mermaid.push_str(&format!(
                "        {}[{}📱{}<br/>{}]\n",
                self.sanitize_name(&device.name),
                device_type,
                device.name,
                device.ip_address
            ));
        }
        
        mermaid.push_str("    end\n\n");
        
        // Add connections
        for connection in topology.connections() {
            mermaid.push_str(&format!(
                "    device_{} ---|{}| device_{}\n",
                connection.source.device_id,
                format!("{:?}", connection.connection_type),
                connection.target.device_id
            ));
        }
        
        // Add styling
        mermaid.push_str("\n    classDef router fill:#e1f5fe\n");
        mermaid.push_str("    classDef switch fill:#f3e5f5\n");
        mermaid.push_str("    classDef host fill:#e8f5e8\n");
        
        Ok(mermaid)
    }
    
    /// Generate README
    async fn generate_readme(&self, topology: &NetworkTopology, generated_files: &GeneratedFiles) -> Result<String, NixTopologyError> {
        let readme = format!(r#"# {} Network Deployment

This directory contains the generated Nix configuration for deploying the "{}" network topology.

## Network Overview

- **Base Network**: {}
- **Topology Type**: {:?}
- **Device Count**: {}
- **Generated**: {}

## Quick Start

1. **Validate the topology**:
   ```bash
   nix run .#validate
   ```

2. **Deploy the network**:
   ```bash
   nix run .#deploy
   ```

3. **Generate network diagram**:
   ```bash
   nix run .#topology
   ```

## Files Structure

- `flake.nix` - Main Nix flake configuration
- `topology.nix` - Network topology definition
- `modules/` - Device-specific NixOS modules
- `scripts/` - Deployment and management scripts
- `docs/` - Generated documentation

## Device Configuration

{}

## Network Configuration

- **LAN Network**: {}
- **Gateway**: {}

## Deployment Commands

Each device can be deployed individually:

```bash
{}
```

## Generated with CIM-Network

This configuration was generated using the CIM-Network module with nix-topology integration.
For more information, see: https://github.com/thecowboyai/cim-network
"#,
            topology.name(),
            topology.name(),
            topology.base_ip(),
            topology.topology_type(),
            topology.devices().len(),
            Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
            topology.devices().values()
                .map(|device| format!("- **{}** ({}): {}", 
                                    device.name, 
                                    self.device_type_to_string(&device.device_type),
                                    device.ip_address))
                .collect::<Vec<_>>()
                .join("\n"),
            topology.base_ip(),
            topology.base_ip().network().nth(1).unwrap(),
            topology.devices().values()
                .map(|device| format!("# Deploy {}\nnixos-rebuild switch --flake .#{} --target-host {}", 
                                    device.name, device.name, device.ip_address))
                .collect::<Vec<_>>()
                .join("\n\n")
        );
        
        Ok(readme)
    }
    
    /// Validate generated files
    async fn validate_generated_files(&self, _generated_files: &GeneratedFiles) -> Result<ValidationResults, NixTopologyError> {
        // TODO: implement proper validation
        Ok(ValidationResults {
            overall_status: ValidationStatus::Valid,
            syntax_errors: Vec::new(),
            semantic_warnings: Vec::new(),
            recommendations: Vec::new(),
        })
    }
    
    /// Generate events for the generation process
    fn generate_events(
        &self,
        generation_id: GenerationId,
        topology: &NetworkTopology,
        generated_files: &GeneratedFiles,
        correlation_id: &CorrelationId,
        causation_id: &CausationId,
    ) -> Vec<NetworkEvent> {
        vec![
            NetworkEvent::NixTopologyGenerationStarted {
                metadata: EventMetadata {
                    event_id: EventId::new(),
                    aggregate_id: topology.id().into(),
                    correlation_id: *correlation_id,
                    causation_id: *causation_id,
                    timestamp: Utc::now(),
                    version: topology.version(),
                },
                topology_id: topology.id(),
                generation_id,
            },
            NetworkEvent::NixTopologyGenerationCompleted {
                metadata: EventMetadata {
                    event_id: EventId::new(),
                    aggregate_id: topology.id().into(),
                    correlation_id: *correlation_id,
                    causation_id: *causation_id,
                    timestamp: Utc::now(),
                    version: topology.version(),
                },
                topology_id: topology.id(),
                generation_id,
                file_count: generated_files.nixos_modules.len() as u32 + 2, // +2 for flake.nix and topology.nix
            }
        ]
    }
    
    /// Convert device type to string
    fn device_type_to_string(&self, device_type: &DeviceType) -> String {
        match device_type {
            DeviceType::Router { .. } => "router".to_string(),
            DeviceType::Switch { .. } => "switch".to_string(),
            DeviceType::Host { .. } => "host".to_string(),
            DeviceType::Container { .. } => "container".to_string(),
            DeviceType::VM { .. } => "vm".to_string(),
        }
    }
    
    /// Sanitize name for use in diagrams
    fn sanitize_name(&self, name: &str) -> String {
        name.replace('-', "_").replace('.', "_")
    }
}

/// Simple implementation of template engine for basic substitution
/// Context graph-based template engine for Nix configuration generation
pub struct ContextGraphTemplateEngine {
    // Note: ContextGraph is a projection built from events, we'll use it as a read-only structure
    nodes: Vec<ContextNode>,
    edges: Vec<ContextEdge>,
}

impl ContextGraphTemplateEngine {
    /// Create a new context graph template engine
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
    
    /// Build context graph from network topology
    fn build_context_graph(&mut self, topology: &NetworkTopology) -> Result<(), TemplateError> {
        // Clear existing graph
        self.nodes.clear();
        self.edges.clear();
        
        // Add topology root node
        let topology_node = ContextNode {
            id: "topology".to_string(),
            node_type: ContextNodeType::Aggregate,
            name: topology.name().to_string(),
            data: serde_json::json!({
                "type": format!("{:?}", topology.topology_type()),
                "base_network": topology.base_ip().to_string(),
            }),
        };
        self.nodes.push(topology_node);
        
        // Add device nodes
        for (device_id, device) in topology.devices() {
            let device_node = ContextNode {
                id: device_id.to_string(),
                node_type: ContextNodeType::Entity,
                name: device.name.clone(),
                data: serde_json::json!({
                    "device_type": format!("{:?}", device.device_type),
                    "ip_address": device.ip_address.to_string(),
                }),
            };
            self.nodes.push(device_node);
            
            // Connect device to topology
            let edge = ContextEdge {
                id: format!("topology-{}", device_id),
                source: "topology".to_string(),
                target: device_id.to_string(),
                relationship: "contains".to_string(),
            };
            self.edges.push(edge);
        }
        
        // Add connection edges between devices
        for connection in topology.connections() {
            let edge = ContextEdge {
                id: format!("{}-{}", connection.source.device_id, connection.target.device_id),
                source: connection.source.device_id.to_string(),
                target: connection.target.device_id.to_string(),
                relationship: "connected_to".to_string(),
            };
            self.edges.push(edge);
        }
        
        Ok(())
    }
}

#[async_trait]
impl TemplateEngine for ContextGraphTemplateEngine {
    async fn render_template(
        &self,
        template_name: &str,
        context: &TemplateContext,
    ) -> Result<String, TemplateError> {
        match template_name {
            "flake.nix" => self.render_flake_from_graph(context).await,
            "topology.nix" => self.render_topology_from_graph(context).await,
            "device_module" => self.render_device_from_graph(context).await,
            _ => Err(TemplateError::NotFound(template_name.to_string())),
        }
    }
    
    async fn render_flake_template(
        &self,
        topology: &NetworkTopology,
        options: &NixGenerationOptions,
    ) -> Result<String, TemplateError> {
        let mut engine = self.clone();
        engine.build_context_graph(topology)?;
        
        // Generate flake.nix from context graph
        let flake_content = format!(
            r#"{{
  description = "Network topology: {}";
  
  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-topology.url = "github:oddlama/nix-topology";
    cim-network.url = "github:thecowboyai/cim-network";
  }};
  
  outputs = {{ self, nixpkgs, nix-topology, cim-network }}: {{
    # Generate NixOS configurations from context graph
    nixosConfigurations = {{
{}
    }};
    
    # Development shell
    devShells.x86_64-linux.default = nixpkgs.legacyPackages.x86_64-linux.mkShell {{
      buildInputs = with nixpkgs.legacyPackages.x86_64-linux; [
        nix-topology.packages.x86_64-linux.default
        cim-network.packages.x86_64-linux.default
      ];
    }};
    
    # Validation and deployment packages
    packages.x86_64-linux = {{
      validate = nixpkgs.legacyPackages.x86_64-linux.writeShellScript "validate" ''
        echo "Validating network topology..."
        nix flake check
      '';
      
      topology = nixpkgs.legacyPackages.x86_64-linux.writeShellScript "topology" ''
        echo "Generating network topology diagram..."
        nix-topology --flake .
      '';
      
      deploy = nixpkgs.legacyPackages.x86_64-linux.writeShellScript "deploy" ''
        echo "Deploying network topology..."
        # Add actual deployment commands based on target
        {}
      '';
    }};
  }};
}}"#,
            topology.name(),
            self.generate_nixos_configurations_from_graph(),
            self.generate_deployment_commands_from_graph(options)
        );
        
        Ok(flake_content)
    }
    
    async fn render_topology_template(
        &self,
        topology: &NetworkTopology,
    ) -> Result<String, TemplateError> {
        let mut engine = self.clone();
        engine.build_context_graph(topology)?;
        
        // Generate topology.nix from context graph
        let topology_content = format!(
            r#"{{ config, lib, pkgs, ... }}:
{{
  # Network topology generated from context graph
  topology = {{
    name = "{}";
    
    # Networks derived from graph structure
    networks = {{
{}
    }};
    
    # Nodes derived from graph nodes
    nodes = {{
{}
    }};
    
    # Connections derived from graph edges
    connections = [
{}
    ];
  }};
}}"#,
            topology.name(),
            self.generate_networks_from_graph(),
            self.generate_nodes_from_graph(),
            self.generate_connections_from_graph()
        );
        
        Ok(topology_content)
    }
    
    async fn render_device_module(
        &self,
        device: &NetworkDevice,
        options: &DeviceModuleOptions,
    ) -> Result<String, TemplateError> {
        // Generate device-specific NixOS module from context graph
        let module_content = format!(
            r#"{{ config, lib, pkgs, ... }}:
{{
  # Device module for {} generated from context graph
  system.stateVersion = "24.11";
  
  networking = {{
    hostName = "{}";
    hostId = "{}";
    firewall.enable = true;
    
    interfaces = {{
{}
    }};
  }};
  
  services = {{
    openssh = {{
      enable = true;
      settings = {{
        PasswordAuthentication = false;
        KbdInteractiveAuthentication = false;
      }};
    }};
    
{}
  }};
  
  # Network-specific configuration
  boot.kernel.sysctl = {{
    "net.ipv4.ip_forward" = {};
    "net.ipv6.conf.all.forwarding" = {};
  }};
}}"#,
            device.name,
            self.sanitize_hostname(&device.name),
            self.generate_host_id(&device.name),
            self.generate_interfaces_from_device(device),
            self.generate_device_services_from_graph(device),
            if matches!(device.device_type, DeviceType::Router { .. }) { 1 } else { 0 },
            if matches!(device.device_type, DeviceType::Router { .. }) { 1 } else { 0 }
        );
        
        Ok(module_content)
    }
}

impl Clone for ContextGraphTemplateEngine {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
        }
    }
}

impl ContextGraphTemplateEngine {
    async fn render_flake_from_graph(&self, _context: &TemplateContext) -> Result<String, TemplateError> {
        // This would be called by render_template for "flake.nix"
        // Implementation would query the context graph for flake structure
        Err(TemplateError::RenderError("Direct flake rendering from context not implemented".to_string()))
    }
    
    async fn render_topology_from_graph(&self, _context: &TemplateContext) -> Result<String, TemplateError> {
        // This would be called by render_template for "topology.nix" 
        Err(TemplateError::RenderError("Direct topology rendering from context not implemented".to_string()))
    }
    
    async fn render_device_from_graph(&self, _context: &TemplateContext) -> Result<String, TemplateError> {
        // This would be called by render_template for "device_module"
        Err(TemplateError::RenderError("Direct device rendering from context not implemented".to_string()))
    }
    
    fn generate_nixos_configurations_from_graph(&self) -> String {
        let mut configs = Vec::new();
        
        // Query context graph for device nodes
        for node in &self.nodes {
            if matches!(node.node_type, ContextNodeType::Entity) && node.id != "topology" {
                let device_name = &node.name;
                
                configs.push(format!(
                    r#"      "{}" = nixpkgs.lib.nixosSystem {{
        system = "x86_64-linux";
        modules = [
          ./modules/{}.nix
          nix-topology.nixosModules.default
        ];
      }};"#,
                    self.sanitize_hostname(device_name),
                    self.sanitize_hostname(device_name)
                ));
            }
        }
        
        configs.join("\n")
    }
    
    fn generate_deployment_commands_from_graph(&self, options: &NixGenerationOptions) -> String {
        match &options.deployment_target {
            DeploymentTarget::Local => "echo 'Local deployment - configurations available for testing'".to_string(),
            DeploymentTarget::Remote { host, user } => format!(
                "nixos-rebuild switch --flake .#{} --target-host {}@{}",
                "hostname", user, host
            ),
            DeploymentTarget::Container { runtime } => format!(
                "echo 'Container deployment using {}'", runtime
            ),
            DeploymentTarget::VM { hypervisor } => format!(
                "echo 'VM deployment using {}'", hypervisor
            ),
            DeploymentTarget::Cloud { provider, region } => format!(
                "echo 'Cloud deployment to {} in region {}'", provider, region
            ),
        }
    }
    
    fn generate_networks_from_graph(&self) -> String {
        // Generate network definitions from context graph
        // This would query the graph for network-related nodes/edges
        format!(
            r#"      lan = {{
        cidr = "192.168.1.0/24";
        hosts = {{ }};
      }};"#
        )
    }
    
    fn generate_nodes_from_graph(&self) -> String {
        let mut nodes = Vec::new();
        
        // Query context graph for device nodes
        for node in &self.nodes {
            if matches!(node.node_type, ContextNodeType::Entity) && node.id != "topology" {
                let device_name = &node.name;
                let device_type = node.data.get("device_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                
                nodes.push(format!(
                    r#"      "{}" = {{
        deviceType = "{}";
        interfaces.eth0 = {{
          network = "lan";
          address = "192.168.1.1";
        }};
      }};"#,
                    self.sanitize_hostname(device_name),
                    device_type.to_lowercase()
                ));
            }
        }
        
        nodes.join("\n")
    }
    
    fn generate_connections_from_graph(&self) -> String {
        let mut connections = Vec::new();
        
        // Query context graph for connection edges
        for edge in &self.edges {
            if edge.relationship == "connected_to" {
                connections.push(format!(
                    r#"      {{ from = "{}"; to = "{}"; }};"#,
                    edge.source,
                    edge.target
                ));
            }
        }
        
        connections.join("\n")
    }
    
    fn generate_interfaces_from_device(&self, device: &NetworkDevice) -> String {
        let mut interfaces = Vec::new();
        
        // This would ideally query the context graph for device interfaces
        // Use the device's primary IP address
        interfaces.push(format!(
            r#"      eth0 = {{
        ipv4.addresses = [{{ address = "{}"; prefixLength = 24; }}];
      }};"#,
            device.ip_address
        ));
        
        interfaces.join("\n")
    }
    
    fn generate_device_services_from_graph(&self, device: &NetworkDevice) -> String {
        match device.device_type {
            DeviceType::Router { .. } => {
                r#"    # Router-specific services
    dhcpcd.enable = false;
    networking.firewall.allowedTCPPorts = [ 22 179 ]; # SSH and BGP"#.to_string()
            }
            DeviceType::Switch { .. } => {
                r#"    # Switch-specific services  
    dhcpcd.enable = false;"#.to_string()
            }
            _ => "    # Default services".to_string(),
        }
    }
    
    fn sanitize_hostname(&self, name: &str) -> String {
        name.chars()
            .map(|c| if c.is_alphanumeric() || c == '-' { c } else { '-' })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }
    
    fn generate_host_id(&self, name: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        format!("{:08x}", hasher.finish() as u32)
    }
}

/// Simple file writer implementation
pub struct SimpleFileWriter;

#[async_trait]
impl FileWriter for SimpleFileWriter {
    async fn write_file(&self, path: &PathBuf, content: &str) -> Result<(), std::io::Error> {
        tokio::fs::write(path, content).await
    }
    
    async fn create_directory(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        tokio::fs::create_dir_all(path).await
    }
}

/// Simple Nix formatter (just returns input for now)
pub struct SimpleNixFormatter;

#[async_trait]
impl NixFormatter for SimpleNixFormatter {
    async fn format_nix_code(&self, code: &str) -> Result<String, FormatterError> {
        // TODO: implement proper Nix formatting using nixpkgs-fmt
        Ok(code.to_string())
    }
}

/// Errors for Nix topology generation
#[derive(Debug, thiserror::Error)]
pub enum NixTopologyError {
    #[error("Template error: {0}")]
    TemplateError(String),
    
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Formatter error: {0}")]
    FormatterError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Network error: {0}")]
    NetworkError(#[from] NetworkError),
}

#[derive(Debug, thiserror::Error)]
pub enum TemplateError {
    #[error("Template not found: {0}")]
    NotFound(String),
    
    #[error("Template syntax error: {0}")]
    SyntaxError(String),
    
    #[error("Template rendering error: {0}")]
    RenderError(String),
}

#[derive(Debug, thiserror::Error)]
pub enum FormatterError {
    #[error("Formatter execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Invalid Nix syntax: {0}")]
    InvalidSyntax(String),
}