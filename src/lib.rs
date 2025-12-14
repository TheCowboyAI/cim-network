//! # cim-network
//!
//! Network infrastructure domain module for the Composable Information Machine.
//!
//! This module provides event-driven management of network infrastructure using
//! cim-domain aggregate state machines and a port/adapter pattern for vendor
//! implementations.
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        Source Category                              │
//! │                     (cim-graph::ContextGraph)                       │
//! │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐             │
//! │  │ NetworkNode │───▶│ NetworkEdge │───▶│ NetworkNode │             │
//! │  │  (Device)   │    │(Connection) │    │  (Device)   │             │
//! │  └─────────────┘    └─────────────┘    └─────────────┘             │
//! └─────────────────────────────────────────────────────────────────────┘
//!                              │
//!                              │ DomainFunctor (NetworkFunctor)
//!                              │ F: Graph → Domain
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                        Target Category                              │
//! │                      (Domain Aggregates)                            │
//! │  ┌─────────────────┐    ┌─────────────────┐                        │
//! │  │ NetworkDevice   │    │ NetworkTopology │                        │
//! │  │ (StateMachine)  │───▶│   (Aggregate)   │                        │
//! │  └─────────────────┘    └─────────────────┘                        │
//! └─────────────────────────────────────────────────────────────────────┘
//!                              │
//!                              │ KanExtension
//!                              │ Lan_F(G): Domain → Vendor/Inventory
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                     Extended Category                               │
//! │  ┌─────────────┐    ┌─────────────┐                                │
//! │  │   UniFi     │    │   NetBox    │                                │
//! │  │  Adapter    │    │  Adapter    │                                │
//! │  └─────────────┘    └─────────────┘                                │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use cim_network::domain::*;
//! use cim_network::adapters::UniFiAdapter;
//!
//! // Create domain aggregate
//! let mac = MacAddress::parse("00:11:22:33:44:55").unwrap();
//! let device = NetworkDeviceAggregate::new_discovered(
//!     mac,
//!     DeviceType::Switch,
//!     Some("192.168.1.100".parse().unwrap()),
//! );
//!
//! // Use UniFi adapter
//! let adapter = UniFiAdapter::new(
//!     "https://unifi.local:8443",
//!     "admin",
//!     "password",
//!     "default",
//! ).await?;
//!
//! // Adopt device through port
//! adapter.adopt_device(&device.vendor_id().unwrap()).await?;
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod domain;
pub mod adapters;

// Re-export key types
pub use domain::{
    // Value objects
    DeviceId, TopologyId, ConnectionId, MacAddress, DeviceType,
    PortId, InterfaceConfig, VlanConfig, ConnectionType, LinkSpeed,
    // Aggregates
    NetworkDeviceAggregate, DeviceState, AggregateError,
    // Events and commands
    NetworkEvent, NetworkCommand,
    // Ports
    DeviceControlPort, InventoryPort, DiscoveryPort,
    NetworkManagementPort, EventStorePort, PortError,
    // Functor types
    NetworkFunctor, NetworkKanExtension, VendorExtension, InventoryExtension,
    DomainObject, ExtendedRepresentation, FunctorError,
};

pub use adapters::{UniFiAdapter, NetBoxAdapter};
