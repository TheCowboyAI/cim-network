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

pub mod domain;
pub mod infrastructure;
pub mod application;
pub mod projections;

pub use domain::{
    NetworkId, RouterId, SwitchId, VlanId, ContainerNetworkId,
    NetworkEvent, NetworkCommand,
    NetworkError,
};

pub use application::NetworkService;
pub use infrastructure::NixTopologyGenerator;