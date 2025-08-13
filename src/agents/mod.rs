//! Interactive sub-agents for CIM Network
//!
//! This module contains intelligent sub-agents that can interactively build
//! and manage network topologies using event-driven context graphs.

pub mod network_topology_builder;
pub mod cli;
pub mod subagent;
pub mod sdn_agent;

pub use network_topology_builder::{
    NetworkTopologyBuilderAgent, TopologyCommand, AgentResponse, 
    NetworkLocation, NetworkConnection, ConfigurationFormat,
    InteractionState, TopologySummary, OfficeSize, CloudProvider, VPNProtocol
};
pub use cli::NetworkTopologyCLI;
pub use subagent::{
    NetworkTopologySubAgent, SubAgentRequest, SubAgentResponse, 
    SubAgentTask, SubAgentMetadata, SubAgentStatus, InitialTopologyParams,
    create_subagent_request, create_build_topology_request
};
pub use sdn_agent::{SDNAgent, SDNAgentResponse, NetworkSummary};