//! # Adapter Layer
//!
//! Adapters implement the domain ports for specific external systems.
//!
//! ## Adapter Types
//!
//! ### Vendor Adapters (DeviceControlPort)
//! - `unifi/` - Ubiquiti UniFi Controller
//! - Future: Cisco, Arista, MikroTik
//!
//! ### Inventory Adapters (InventoryPort)
//! - `netbox/` - NetBox DCIM/IPAM
//!
//! ## Kan Extension Integration
//!
//! Each adapter implements both:
//! 1. A domain port (for hexagonal architecture)
//! 2. A `VendorExtension` or `InventoryExtension` (for Kan extension)
//!
//! This allows the same adapter to be used both:
//! - Imperatively through the port interface
//! - Categorically through the Kan extension

pub mod unifi;
pub mod netbox;

pub use unifi::UniFiAdapter;
pub use netbox::NetBoxAdapter;
