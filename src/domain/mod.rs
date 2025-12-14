//! # Network Domain Layer
//!
//! This module implements the network infrastructure domain using cim-domain patterns
//! with graph->domain Kan extensions for vendor-agnostic network management.
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
//! │           │                     │                                   │
//! │           │  Ports (Traits)     │                                   │
//! │           ▼                     ▼                                   │
//! │  ┌─────────────────────────────────────────────────────┐           │
//! │  │ NetworkDevicePort │ DiscoveryPort │ ConfigurationPort│           │
//! │  └─────────────────────────────────────────────────────┘           │
//! └─────────────────────────────────────────────────────────────────────┘
//!                              │
//!                              │ KanExtension
//!                              │ Lan_F(G): Domain → Vendor
//!                              ▼
//! ┌─────────────────────────────────────────────────────────────────────┐
//! │                     Extended Category                               │
//! │                    (Vendor Representations)                         │
//! │  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐             │
//! │  │   UniFi     │    │   Cisco     │    │   Arista    │             │
//! │  │  Adapter    │    │  Adapter    │    │  Adapter    │             │
//! │  └─────────────┘    └─────────────┘    └─────────────┘             │
//! └─────────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Kan Extension Pattern
//!
//! The Kan extension allows us to:
//! 1. Define domain logic once (in the Domain category)
//! 2. Extend it to any vendor (via Lan_F: colimit construction)
//! 3. Preserve compositional structure across the extension
//!
//! For network management:
//! - `F: Graph → Domain` maps topology to aggregates
//! - `Lan_F(G)(vendor)` extends domain operations to vendor-specific APIs
//! - Universal property ensures vendor adapters compose correctly

pub mod aggregates;
pub mod events;
pub mod commands;
pub mod ports;
pub mod functor;
pub mod value_objects;

// Re-exports - explicit to avoid ambiguity
pub use aggregates::{
    NetworkDeviceAggregate, DeviceState, AggregateError,
};
pub use events::NetworkEvent;
pub use commands::NetworkCommand;
pub use ports::{
    DeviceControlPort, InventoryPort, DiscoveryPort,
    NetworkManagementPort, EventStorePort, PortError,
    DeviceConfiguration, DiscoveredDevice, DeviceDetails,
    VendorDevice, VendorConfig, DeviceStats, PortStats,
    IpAssignment, IpStatus, EventSubscription,
    ConnectionInfo,
};
pub use functor::{
    NetworkFunctor, NetworkKanExtension, VendorExtension, InventoryExtension,
    NetworkGraphNode, NetworkGraphEdge, NetworkNodeType, NetworkEdgeType,
    DomainObject, DomainMorphism, DomainMorphismType,
    ExtendedRepresentation, VendorRepresentation, InventoryRepresentation,
    FunctorError,
    // Note: functor also has ConnectionInfo and TopologyInfo as domain objects
    TopologyInfo,
};
pub use value_objects::{
    DeviceId, TopologyId, ConnectionId, MacAddress, MacAddressError,
    DeviceType, PortId, InterfaceConfig, VlanConfig, VlanError,
    ConnectionType, LinkSpeed,
};
