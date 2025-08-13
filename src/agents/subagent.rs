//! Network Topology Builder Sub-Agent
//!
//! This is the actual sub-agent that can be invoked by Claude Code to interactively
//! build network topologies using events and context graphs.

use super::network_topology_builder::{
    NetworkTopologyBuilderAgent, TopologyCommand, NetworkLocation, NetworkConnection,
    ConfigurationFormat, OfficeSize, CloudProvider, VPNProtocol, AgentResponse
};
use crate::domain::{IpNetwork, NetworkError};
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use std::collections::HashMap;

/// The actual sub-agent that can be invoked by Claude Code
#[derive(Debug, Clone)]
pub struct NetworkTopologySubAgent {
    /// The core agent implementation
    agent: NetworkTopologyBuilderAgent,
    /// Sub-agent metadata
    metadata: SubAgentMetadata,
}

/// Sub-agent metadata for Claude Code integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentMetadata {
    /// Sub-agent name
    pub name: String,
    /// Sub-agent description
    pub description: String,
    /// Version
    pub version: String,
    /// Capabilities
    pub capabilities: Vec<String>,
    /// Current status
    pub status: SubAgentStatus,
}

/// Sub-agent status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubAgentStatus {
    /// Ready to receive commands
    Ready,
    /// Processing a command
    Processing,
    /// Waiting for user input
    WaitingForInput,
    /// Error state
    Error { message: String },
    /// Completed task
    Completed,
}

/// Sub-agent request from Claude Code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentRequest {
    /// Request ID for tracking
    pub request_id: String,
    /// The task or command to execute
    pub task: SubAgentTask,
    /// Additional context from Claude Code
    pub context: HashMap<String, serde_json::Value>,
}

/// Tasks that the sub-agent can perform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubAgentTask {
    /// Build a network topology interactively
    BuildTopology {
        /// Initial parameters
        initial_params: Option<InitialTopologyParams>,
    },
    /// Add a network location
    AddLocation {
        location_id: String,
        location_type: String,
        parameters: HashMap<String, String>,
    },
    /// Connect two locations
    ConnectLocations {
        from: String,
        to: String,
        connection_type: String,
        parameters: HashMap<String, String>,
    },
    /// Generate configuration in specific format
    GenerateConfiguration {
        format: String,
    },
    /// Validate current topology
    ValidateTopology,
    /// Get current topology status
    GetStatus,
    /// Reset topology
    Reset,
    /// Complete topology building
    Complete,
}

/// Initial parameters for topology building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitialTopologyParams {
    /// Base IP network range
    pub base_network: Option<String>,
    /// Target deployment environment
    pub target_environment: Option<String>,
    /// Expected scale (small/medium/large/enterprise)
    pub scale: Option<String>,
    /// Primary use case
    pub use_case: Option<String>,
}

/// Response from the sub-agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentResponse {
    /// Request ID being responded to
    pub request_id: String,
    /// Whether the task was successful
    pub success: bool,
    /// Response message
    pub message: String,
    /// Structured data response
    pub data: serde_json::Value,
    /// Next suggested actions
    pub suggested_actions: Vec<String>,
    /// Updated sub-agent metadata
    pub metadata: SubAgentMetadata,
    /// Any errors that occurred
    pub errors: Vec<String>,
}

impl NetworkTopologySubAgent {
    /// Create a new network topology sub-agent
    pub fn new() -> Self {
        Self {
            agent: NetworkTopologyBuilderAgent::new(),
            metadata: SubAgentMetadata {
                name: "Network Topology Builder".to_string(),
                description: "Interactive sub-agent for building network topologies using event-driven context graphs".to_string(),
                version: "1.0.0".to_string(),
                capabilities: vec![
                    "Add network locations (data centers, offices, cloud regions, virtual segments)".to_string(),
                    "Connect locations with various connection types (fiber, VPN, internet, direct connect)".to_string(),
                    "Validate network topology completeness and correctness".to_string(),
                    "Generate configurations in multiple formats (NixOS, Terraform, Ansible, JSON, YAML)".to_string(),
                    "Provide intelligent suggestions based on current topology state".to_string(),
                    "Maintain full event audit trail for topology construction".to_string(),
                ],
                status: SubAgentStatus::Ready,
            },
        }
    }

    /// Process a sub-agent request from Claude Code
    pub async fn process_request(&mut self, request: SubAgentRequest) -> SubAgentResponse {
        self.metadata.status = SubAgentStatus::Processing;
        
        let result = self.execute_task(&request.task).await;
        
        match result {
            Ok(agent_response) => {
                self.metadata.status = SubAgentStatus::Ready;
                
                SubAgentResponse {
                    request_id: request.request_id,
                    success: true,
                    message: agent_response.message,
                    data: serde_json::json!({
                        "topology_summary": agent_response.topology_summary,
                        "events_generated": agent_response.events.len(),
                        "interaction_state": agent_response.new_state,
                    }),
                    suggested_actions: agent_response.suggested_actions,
                    metadata: self.metadata.clone(),
                    errors: Vec::new(),
                }
            }
            Err(e) => {
                self.metadata.status = SubAgentStatus::Error { message: e.to_string() };
                
                SubAgentResponse {
                    request_id: request.request_id,
                    success: false,
                    message: format!("Task failed: {}", e),
                    data: serde_json::Value::Null,
                    suggested_actions: vec!["Try a different approach".to_string(), "Check input parameters".to_string()],
                    metadata: self.metadata.clone(),
                    errors: vec![e.to_string()],
                }
            }
        }
    }

    /// Execute a specific task
    async fn execute_task(&mut self, task: &SubAgentTask) -> Result<AgentResponse, NetworkError> {
        match task {
            SubAgentTask::BuildTopology { initial_params } => {
                self.start_topology_building(initial_params.as_ref()).await
            }
            SubAgentTask::AddLocation { location_id, location_type, parameters } => {
                self.add_location_from_params(location_id, location_type, parameters).await
            }
            SubAgentTask::ConnectLocations { from, to, connection_type, parameters } => {
                self.connect_locations_from_params(from, to, connection_type, parameters).await
            }
            SubAgentTask::GenerateConfiguration { format } => {
                self.generate_configuration_from_string(format).await
            }
            SubAgentTask::ValidateTopology => {
                self.agent.process_command(TopologyCommand::ValidateTopology).await
            }
            SubAgentTask::GetStatus => {
                self.agent.process_command(TopologyCommand::ListTopology).await
            }
            SubAgentTask::Reset => {
                self.agent.process_command(TopologyCommand::Reset).await
            }
            SubAgentTask::Complete => {
                self.agent.process_command(TopologyCommand::Complete).await
            }
        }
    }

    /// Start topology building with initial parameters
    async fn start_topology_building(&mut self, params: Option<&InitialTopologyParams>) -> Result<AgentResponse, NetworkError> {
        let mut message = "🌐 Starting interactive network topology building!".to_string();
        
        if let Some(params) = params {
            message.push_str(&format!("\n\nInitial Parameters:"));
            if let Some(network) = &params.base_network {
                message.push_str(&format!("\n• Base Network: {}", network));
            }
            if let Some(env) = &params.target_environment {
                message.push_str(&format!("\n• Target Environment: {}", env));
            }
            if let Some(scale) = &params.scale {
                message.push_str(&format!("\n• Scale: {}", scale));
            }
            if let Some(use_case) = &params.use_case {
                message.push_str(&format!("\n• Use Case: {}", use_case));
            }
        }

        // Return a status response
        self.agent.process_command(TopologyCommand::ListTopology).await
    }

    /// Add a location from string parameters
    async fn add_location_from_params(
        &mut self,
        location_id: &str,
        location_type: &str,
        parameters: &HashMap<String, String>,
    ) -> Result<AgentResponse, NetworkError> {
        let location = match location_type.to_lowercase().as_str() {
            "datacenter" | "dc" => {
                NetworkLocation::DataCenter {
                    name: parameters.get("name").cloned().unwrap_or_else(|| location_id.to_string()),
                    region: parameters.get("region").cloned().unwrap_or_else(|| "us-west-1".to_string()),
                    availability_zone: parameters.get("az").cloned(),
                }
            }
            "office" => {
                let size = match parameters.get("size").map(|s| s.to_lowercase().as_str()).unwrap_or("medium") {
                    "small" => OfficeSize::Small,
                    "medium" => OfficeSize::Medium,
                    "large" => OfficeSize::Large,
                    "campus" => OfficeSize::Campus,
                    _ => OfficeSize::Medium,
                };
                NetworkLocation::Office {
                    name: parameters.get("name").cloned().unwrap_or_else(|| location_id.to_string()),
                    address: parameters.get("address").cloned().unwrap_or_else(|| "Unknown Address".to_string()),
                    size,
                }
            }
            "cloud" | "cloudregion" => {
                let provider = match parameters.get("provider").map(|s| s.to_lowercase().as_str()).unwrap_or("aws") {
                    "aws" => CloudProvider::AWS,
                    "azure" => CloudProvider::Azure,
                    "gcp" | "google" => CloudProvider::GCP,
                    "digitalocean" | "do" => CloudProvider::DigitalOcean,
                    other => CloudProvider::Custom(other.to_string()),
                };
                NetworkLocation::CloudRegion {
                    provider,
                    region: parameters.get("region").cloned().unwrap_or_else(|| "us-east-1".to_string()),
                }
            }
            "edge" | "edgelocation" => {
                let latitude = parameters.get("lat").and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let longitude = parameters.get("lng").and_then(|s| s.parse().ok()).unwrap_or(0.0);
                NetworkLocation::EdgeLocation {
                    name: parameters.get("name").cloned().unwrap_or_else(|| location_id.to_string()),
                    latitude,
                    longitude,
                }
            }
            "segment" | "virtualsegment" => {
                let subnet = parameters.get("subnet")
                    .and_then(|s| IpNetwork::from_str(s).ok())
                    .unwrap_or_else(|| IpNetwork::from_str("192.168.1.0/24").unwrap());
                let vlan_id = parameters.get("vlan").and_then(|s| s.parse().ok());
                NetworkLocation::VirtualSegment {
                    name: parameters.get("name").cloned().unwrap_or_else(|| location_id.to_string()),
                    subnet,
                    vlan_id,
                }
            }
            _ => {
                return Err(NetworkError::ValidationError(
                    format!("Unknown location type: {}. Supported types: datacenter, office, cloud, edge, segment", location_type)
                ));
            }
        };

        self.agent.process_command(TopologyCommand::AddLocation {
            location_id: location_id.to_string(),
            location,
        }).await
    }

    /// Connect locations from string parameters
    async fn connect_locations_from_params(
        &mut self,
        from: &str,
        to: &str,
        connection_type: &str,
        parameters: &HashMap<String, String>,
    ) -> Result<AgentResponse, NetworkError> {
        let connection = match connection_type.to_lowercase().as_str() {
            "fiber" => {
                NetworkConnection::Fiber {
                    bandwidth: parameters.get("bandwidth").cloned().unwrap_or_else(|| "1Gbps".to_string()),
                    redundant: parameters.get("redundant").map(|s| s == "true").unwrap_or(false),
                }
            }
            "vpn" => {
                let protocol = match parameters.get("protocol").map(|s| s.to_lowercase().as_str()).unwrap_or("ipsec") {
                    "ipsec" => VPNProtocol::IPSec,
                    "wireguard" | "wg" => VPNProtocol::WireGuard,
                    "openvpn" => VPNProtocol::OpenVPN,
                    other => VPNProtocol::Custom(other.to_string()),
                };
                NetworkConnection::VPN {
                    protocol,
                    encrypted: parameters.get("encrypted").map(|s| s == "true").unwrap_or(true),
                }
            }
            "internet" => {
                NetworkConnection::Internet {
                    bandwidth: parameters.get("bandwidth").cloned().unwrap_or_else(|| "100Mbps".to_string()),
                    provider: parameters.get("provider").cloned().unwrap_or_else(|| "ISP".to_string()),
                }
            }
            "directconnect" | "direct" => {
                let provider = match parameters.get("provider").map(|s| s.to_lowercase().as_str()).unwrap_or("aws") {
                    "aws" => CloudProvider::AWS,
                    "azure" => CloudProvider::Azure,
                    "gcp" => CloudProvider::GCP,
                    "digitalocean" => CloudProvider::DigitalOcean,
                    other => CloudProvider::Custom(other.to_string()),
                };
                NetworkConnection::DirectConnect {
                    provider,
                    bandwidth: parameters.get("bandwidth").cloned().unwrap_or_else(|| "1Gbps".to_string()),
                }
            }
            "virtual" => {
                NetworkConnection::Virtual {
                    protocol: parameters.get("protocol").cloned().unwrap_or_else(|| "VLAN".to_string()),
                    bandwidth: parameters.get("bandwidth").cloned(),
                }
            }
            _ => {
                return Err(NetworkError::ValidationError(
                    format!("Unknown connection type: {}. Supported types: fiber, vpn, internet, directconnect, virtual", connection_type)
                ));
            }
        };

        self.agent.process_command(TopologyCommand::ConnectLocations {
            from: from.to_string(),
            to: to.to_string(),
            connection,
        }).await
    }

    /// Generate configuration from format string
    async fn generate_configuration_from_string(&mut self, format: &str) -> Result<AgentResponse, NetworkError> {
        let config_format = match format.to_lowercase().as_str() {
            "nixos" | "nix" => ConfigurationFormat::NixOS,
            "nix-darwin" | "darwin" => ConfigurationFormat::NixDarwin,
            "home-manager" | "hm" => ConfigurationFormat::HomeManager,
            "flake" | "nix-flake" => ConfigurationFormat::NixFlake,
            "json" => ConfigurationFormat::JSON,
            _ => {
                return Err(NetworkError::ValidationError(
                    format!("Unknown configuration format: {}. Supported formats: nixos, nix-darwin, home-manager, flake, json", format)
                ));
            }
        };

        self.agent.process_command(TopologyCommand::GenerateConfiguration {
            format: config_format,
        }).await
    }

    /// Get sub-agent metadata
    pub fn metadata(&self) -> &SubAgentMetadata {
        &self.metadata
    }

    /// Get current session information
    pub fn session_info(&self) -> serde_json::Value {
        serde_json::json!({
            "session_id": self.agent.session_id(),
            "interaction_state": self.agent.interaction_state(),
            "events_generated": self.agent.get_events().len(),
            "agent_status": self.metadata.status,
        })
    }
}

impl Default for NetworkTopologySubAgent {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a sub-agent request
pub fn create_subagent_request(task: SubAgentTask) -> SubAgentRequest {
    SubAgentRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        task,
        context: HashMap::new(),
    }
}

/// Helper function to create a build topology request
pub fn create_build_topology_request(
    base_network: Option<String>,
    target_environment: Option<String>,
    scale: Option<String>,
    use_case: Option<String>,
) -> SubAgentRequest {
    let initial_params = if base_network.is_some() || target_environment.is_some() || scale.is_some() || use_case.is_some() {
        Some(InitialTopologyParams {
            base_network,
            target_environment,
            scale,
            use_case,
        })
    } else {
        None
    };

    create_subagent_request(SubAgentTask::BuildTopology { initial_params })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_subagent_creation() {
        let subagent = NetworkTopologySubAgent::new();
        assert_eq!(subagent.metadata.name, "Network Topology Builder");
        assert!(matches!(subagent.metadata.status, SubAgentStatus::Ready));
    }

    #[tokio::test]
    async fn test_build_topology_request() {
        let mut subagent = NetworkTopologySubAgent::new();
        
        let request = create_build_topology_request(
            Some("10.0.0.0/8".to_string()),
            Some("production".to_string()),
            Some("enterprise".to_string()),
            Some("multi-region-cloud".to_string()),
        );

        let response = subagent.process_request(request).await;
        assert!(response.success);
        assert!(response.message.contains("Starting interactive network topology building"));
    }

    #[tokio::test]
    async fn test_add_datacenter() {
        let mut subagent = NetworkTopologySubAgent::new();
        
        let mut params = HashMap::new();
        params.insert("name".to_string(), "Primary DC".to_string());
        params.insert("region".to_string(), "us-west-1".to_string());
        params.insert("az".to_string(), "us-west-1a".to_string());

        let request = create_subagent_request(SubAgentTask::AddLocation {
            location_id: "dc1".to_string(),
            location_type: "datacenter".to_string(),
            parameters: params,
        });

        let response = subagent.process_request(request).await;
        assert!(response.success);
        assert!(response.message.contains("Added location"));
    }

    #[tokio::test]
    async fn test_connect_locations() {
        let mut subagent = NetworkTopologySubAgent::new();
        
        // First add two locations
        let mut dc_params = HashMap::new();
        dc_params.insert("name".to_string(), "DC1".to_string());
        dc_params.insert("region".to_string(), "us-west-1".to_string());

        let add_dc_request = create_subagent_request(SubAgentTask::AddLocation {
            location_id: "dc1".to_string(),
            location_type: "datacenter".to_string(),
            parameters: dc_params,
        });

        let mut office_params = HashMap::new();
        office_params.insert("name".to_string(), "HQ".to_string());
        office_params.insert("address".to_string(), "123 Main St".to_string());
        office_params.insert("size".to_string(), "large".to_string());

        let add_office_request = create_subagent_request(SubAgentTask::AddLocation {
            location_id: "hq".to_string(),
            location_type: "office".to_string(),
            parameters: office_params,
        });

        // Add locations
        subagent.process_request(add_dc_request).await;
        subagent.process_request(add_office_request).await;

        // Now connect them
        let mut conn_params = HashMap::new();
        conn_params.insert("bandwidth".to_string(), "10Gbps".to_string());
        conn_params.insert("redundant".to_string(), "true".to_string());

        let connect_request = create_subagent_request(SubAgentTask::ConnectLocations {
            from: "dc1".to_string(),
            to: "hq".to_string(),
            connection_type: "fiber".to_string(),
            parameters: conn_params,
        });

        let response = subagent.process_request(connect_request).await;
        assert!(response.success);
        assert!(response.message.contains("Connected"));
    }
}