//! # CIM Network Infrastructure Module
//! 
//! Network infrastructure management for the Composable Information Machine.
//! 
//! This module provides event-driven management of physical and virtual network
//! infrastructure including routers, switches, VLANs, and container networks.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

pub mod agents;
pub mod api;
pub mod domain;
pub mod infrastructure;
pub mod application;
pub mod projections;
pub mod nix_integration;
pub mod sdn;

pub use domain::{
    NetworkId, RouterId, SwitchId, VlanId, ContainerNetworkId,
    ConnectionId, IpNetwork, EventId,
    NetworkEvent, NetworkCommand,
    NetworkError,
    // Export event-related types for tests
    EventMetadata, CorrelationId, CausationId, AggregateId,
    RouterVendor, CiscoOs, DeploymentMethod,
};

pub use api::deployment::{NetworkDeploymentAPI, deploy_network_simple};
pub use application::NetworkService;
pub use infrastructure::nix::NixTopologyGenerator;