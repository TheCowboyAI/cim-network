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
//! ┌─────────────────────────────────────────────────────────┐
//! │                    cim-network                          │
//! ├─────────────────────────────────────────────────────────┤
//! │  Domain Layer (Ports)                                   │
//! │  ├── NetworkDevice aggregate (state machine)            │
//! │  ├── NetworkTopology aggregate                          │
//! │  └── Events, Commands, Queries                          │
//! ├─────────────────────────────────────────────────────────┤
//! │  Adapter Layer                                          │
//! │  ├── unifi/ - UniFi Controller adapter                  │
//! │  └── (future vendors...)                                │
//! └─────────────────────────────────────────────────────────┘
//! ```

#![warn(missing_docs)]
#![warn(clippy::all)]

pub mod domain;
pub mod adapters;

// Re-exports will be added as modules are implemented
