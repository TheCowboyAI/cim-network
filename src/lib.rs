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
//! use cim_network::*;
//!
//! // Connect to NATS for event persistence
//! let event_store = NatsEventStore::connect("nats://localhost:4222").await?;
//!
//! // Create domain aggregate
//! let mac = MacAddress::parse("00:11:22:33:44:55").unwrap();
//! let mut device = NetworkDeviceAggregate::new_discovered(
//!     mac,
//!     DeviceType::Switch,
//!     Some("192.168.1.100".parse().unwrap()),
//! );
//!
//! // Persist discovery event
//! event_store.append(device.take_pending_events()).await?;
//!
//! // Use UniFi adapter for device control
//! let unifi = UniFiAdapter::new(
//!     "https://unifi.local:8443",
//!     "admin",
//!     "password",
//!     "default",
//! ).await?;
//! unifi.connect().await?;
//!
//! // Adopt device
//! unifi.adopt_device(&device.mac().to_string()).await?;
//!
//! // Sync to NetBox inventory
//! let netbox = NetBoxAdapter::new(
//!     "https://netbox.local",
//!     "api-token",
//! )?;
//! netbox.sync_device(&device).await?;
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
    // Infrastructure bridge
    InfrastructureBridge, BridgeError,
    device_type_to_compute_type, compute_type_to_device_type,
    compute_resource_to_network_device,
    // Infrastructure domain re-export
    infrastructure,
};

pub use adapters::{
    UniFiAdapter, NetBoxAdapter,
    NatsEventStore, NatsEventStoreConfig, NatsEventSubscriber, NatsEventAck,
};

pub mod service;
