//! Network Topology Builder CLI
//!
//! Interactive command-line interface for building network topologies
//! using event-driven context graphs.

use cim_network::agents::NetworkTopologyCLI;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = NetworkTopologyCLI::new();
    cli.start().await?;
    Ok(())
}