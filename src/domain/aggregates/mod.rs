//! Domain aggregates

pub mod router_configuration;
pub mod switch_configuration;
#[cfg(feature = "workflows")]
pub mod workflow_router_configuration;
pub mod network_topology;