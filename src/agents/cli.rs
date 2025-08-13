//! CLI interface for the Network Topology Builder sub-agent

use super::network_topology_builder::{
    NetworkTopologyBuilderAgent, TopologyCommand, NetworkLocation, NetworkConnection,
    ConfigurationFormat, OfficeSize, CloudProvider, VPNProtocol
};
use crate::domain::{IpNetwork, NetworkError};
use std::str::FromStr;
use std::io::{self, Write};

/// Interactive CLI for building network topologies
pub struct NetworkTopologyCLI {
    agent: NetworkTopologyBuilderAgent,
}

impl NetworkTopologyCLI {
    /// Create a new CLI instance
    pub fn new() -> Self {
        Self {
            agent: NetworkTopologyBuilderAgent::new(),
        }
    }

    /// Start the interactive CLI session
    pub async fn start(&mut self) -> Result<(), NetworkError> {
        println!("🌐 CIM Network Topology Builder");
        println!("===============================");
        println!();
        println!("Build your network topology interactively using events and context graphs!");
        println!("Type 'help' for available commands or 'quit' to exit.");
        println!();

        loop {
            print!("network-builder> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            if input == "quit" || input == "exit" {
                println!("👋 Goodbye!");
                break;
            }

            match self.process_input(input).await {
                Ok(()) => {}
                Err(e) => {
                    println!("❌ Error: {}", e);
                    println!();
                }
            }
        }

        Ok(())
    }

    /// Process user input and execute commands
    async fn process_input(&mut self, input: &str) -> Result<(), NetworkError> {
        match input {
            "help" | "h" => {
                self.show_help();
                Ok(())
            }
            "list" | "ls" => {
                let response = self.agent.process_command(TopologyCommand::ListTopology).await?;
                self.display_response(&response);
                Ok(())
            }
            "validate" | "check" => {
                let response = self.agent.process_command(TopologyCommand::ValidateTopology).await?;
                self.display_response(&response);
                Ok(())
            }
            "reset" | "clear" => {
                let response = self.agent.process_command(TopologyCommand::Reset).await?;
                self.display_response(&response);
                Ok(())
            }
            "complete" | "done" => {
                let response = self.agent.process_command(TopologyCommand::Complete).await?;
                self.display_response(&response);
                Ok(())
            }
            _ if input.starts_with("add-datacenter ") => {
                self.add_datacenter(&input[15..]).await
            }
            _ if input.starts_with("add-office ") => {
                self.add_office(&input[11..]).await
            }
            _ if input.starts_with("add-cloud ") => {
                self.add_cloud(&input[10..]).await
            }
            _ if input.starts_with("add-segment ") => {
                self.add_segment(&input[12..]).await
            }
            _ if input.starts_with("connect-fiber ") => {
                self.connect_fiber(&input[14..]).await
            }
            _ if input.starts_with("connect-vpn ") => {
                self.connect_vpn(&input[12..]).await
            }
            _ if input.starts_with("connect-internet ") => {
                self.connect_internet(&input[17..]).await
            }
            _ if input.starts_with("generate ") => {
                self.generate_config(&input[9..]).await
            }
            _ if input.starts_with("remove-location ") => {
                self.remove_location(&input[16..]).await
            }
            _ if input.starts_with("remove-connection ") => {
                self.remove_connection(&input[18..]).await
            }
            _ => {
                println!("❓ Unknown command: '{}'", input);
                println!("Type 'help' for available commands.");
                println!();
                Ok(())
            }
        }
    }

    /// Show help information
    fn show_help(&self) {
        println!("📖 Available Commands:");
        println!("======================");
        println!();
        println!("🏢 Adding Locations:");
        println!("  add-datacenter <id> <name> <region> [az]");
        println!("  add-office <id> <name> <address> <size>");
        println!("  add-cloud <id> <provider> <region>");
        println!("  add-segment <id> <name> <subnet> [vlan]");
        println!();
        println!("🔗 Adding Connections:");
        println!("  connect-fiber <from> <to> <bandwidth> [redundant]");
        println!("  connect-vpn <from> <to> <protocol> [encrypted]");
        println!("  connect-internet <from> <to> <bandwidth> <provider>");
        println!();
        println!("📋 Management:");
        println!("  list, ls                    - Show current topology");
        println!("  validate, check             - Validate topology");
        println!("  generate <format>           - Generate config (nixos, terraform, etc.)");
        println!("  remove-location <id>        - Remove a location");
        println!("  remove-connection <from> <to> - Remove a connection");
        println!("  reset, clear                - Start over");
        println!("  complete, done              - Finish and save topology");
        println!("  help, h                     - Show this help");
        println!("  quit, exit                  - Exit the program");
        println!();
        println!("📝 Examples:");
        println!("  add-datacenter dc1 \"Primary DC\" us-west-1");
        println!("  add-office office1 \"HQ\" \"123 Main St\" large");
        println!("  connect-fiber dc1 office1 10Gbps redundant");
        println!("  generate nixos");
        println!();
    }

    /// Add a data center
    async fn add_datacenter(&mut self, args: &str) -> Result<(), NetworkError> {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if parts.len() < 3 {
            println!("❌ Usage: add-datacenter <id> <name> <region> [availability_zone]");
            return Ok(());
        }

        let id = parts[0].to_string();
        let name = parts[1].trim_matches('"').to_string();
        let region = parts[2].to_string();
        let availability_zone = parts.get(3).map(|s| s.to_string());

        let location = NetworkLocation::DataCenter {
            name,
            region,
            availability_zone,
        };

        let response = self.agent.process_command(TopologyCommand::AddLocation {
            location_id: id,
            location,
        }).await?;

        self.display_response(&response);
        Ok(())
    }

    /// Add an office
    async fn add_office(&mut self, args: &str) -> Result<(), NetworkError> {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if parts.len() < 4 {
            println!("❌ Usage: add-office <id> <name> <address> <size>");
            return Ok(());
        }

        let id = parts[0].to_string();
        let name = parts[1].trim_matches('"').to_string();
        let address = parts[2].trim_matches('"').to_string();
        let size = match parts[3].to_lowercase().as_str() {
            "small" => OfficeSize::Small,
            "medium" => OfficeSize::Medium,
            "large" => OfficeSize::Large,
            "campus" => OfficeSize::Campus,
            _ => {
                println!("❌ Invalid office size. Use: small, medium, large, campus");
                return Ok(());
            }
        };

        let location = NetworkLocation::Office {
            name,
            address,
            size,
        };

        let response = self.agent.process_command(TopologyCommand::AddLocation {
            location_id: id,
            location,
        }).await?;

        self.display_response(&response);
        Ok(())
    }

    /// Add a cloud region
    async fn add_cloud(&mut self, args: &str) -> Result<(), NetworkError> {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if parts.len() < 3 {
            println!("❌ Usage: add-cloud <id> <provider> <region>");
            return Ok(());
        }

        let id = parts[0].to_string();
        let provider = match parts[1].to_lowercase().as_str() {
            "aws" => CloudProvider::AWS,
            "azure" => CloudProvider::Azure,
            "gcp" => CloudProvider::GCP,
            "digitalocean" => CloudProvider::DigitalOcean,
            _ => CloudProvider::Custom(parts[1].to_string()),
        };
        let region = parts[2].to_string();

        let location = NetworkLocation::CloudRegion { provider, region };

        let response = self.agent.process_command(TopologyCommand::AddLocation {
            location_id: id,
            location,
        }).await?;

        self.display_response(&response);
        Ok(())
    }

    /// Add a virtual segment
    async fn add_segment(&mut self, args: &str) -> Result<(), NetworkError> {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if parts.len() < 3 {
            println!("❌ Usage: add-segment <id> <name> <subnet> [vlan_id]");
            return Ok(());
        }

        let id = parts[0].to_string();
        let name = parts[1].trim_matches('"').to_string();
        let subnet = IpNetwork::from_str(parts[2])
            .map_err(|e| NetworkError::ValidationError(format!("Invalid subnet: {}", e)))?;
        let vlan_id = parts.get(3)
            .and_then(|s| s.parse::<u16>().ok());

        let location = NetworkLocation::VirtualSegment {
            name,
            subnet,
            vlan_id,
        };

        let response = self.agent.process_command(TopologyCommand::AddLocation {
            location_id: id,
            location,
        }).await?;

        self.display_response(&response);
        Ok(())
    }

    /// Connect locations with fiber
    async fn connect_fiber(&mut self, args: &str) -> Result<(), NetworkError> {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if parts.len() < 3 {
            println!("❌ Usage: connect-fiber <from> <to> <bandwidth> [redundant]");
            return Ok(());
        }

        let from = parts[0].to_string();
        let to = parts[1].to_string();
        let bandwidth = parts[2].to_string();
        let redundant = parts.get(3).map(|s| s == "redundant").unwrap_or(false);

        let connection = NetworkConnection::Fiber { bandwidth, redundant };

        let response = self.agent.process_command(TopologyCommand::ConnectLocations {
            from,
            to,
            connection,
        }).await?;

        self.display_response(&response);
        Ok(())
    }

    /// Connect locations with VPN
    async fn connect_vpn(&mut self, args: &str) -> Result<(), NetworkError> {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if parts.len() < 3 {
            println!("❌ Usage: connect-vpn <from> <to> <protocol> [encrypted]");
            return Ok(());
        }

        let from = parts[0].to_string();
        let to = parts[1].to_string();
        let protocol = match parts[2].to_lowercase().as_str() {
            "ipsec" => VPNProtocol::IPSec,
            "wireguard" => VPNProtocol::WireGuard,
            "openvpn" => VPNProtocol::OpenVPN,
            _ => VPNProtocol::Custom(parts[2].to_string()),
        };
        let encrypted = parts.get(3).map(|s| s == "encrypted").unwrap_or(true);

        let connection = NetworkConnection::VPN { protocol, encrypted };

        let response = self.agent.process_command(TopologyCommand::ConnectLocations {
            from,
            to,
            connection,
        }).await?;

        self.display_response(&response);
        Ok(())
    }

    /// Connect locations via internet
    async fn connect_internet(&mut self, args: &str) -> Result<(), NetworkError> {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if parts.len() < 4 {
            println!("❌ Usage: connect-internet <from> <to> <bandwidth> <provider>");
            return Ok(());
        }

        let from = parts[0].to_string();
        let to = parts[1].to_string();
        let bandwidth = parts[2].to_string();
        let provider = parts[3].to_string();

        let connection = NetworkConnection::Internet { bandwidth, provider };

        let response = self.agent.process_command(TopologyCommand::ConnectLocations {
            from,
            to,
            connection,
        }).await?;

        self.display_response(&response);
        Ok(())
    }

    /// Generate configuration
    async fn generate_config(&mut self, format: &str) -> Result<(), NetworkError> {
        let config_format = match format.to_lowercase().as_str() {
            "nixos" | "nix" => ConfigurationFormat::NixOS,
            "terraform" | "tf" => ConfigurationFormat::Terraform,
            "ansible" => ConfigurationFormat::Ansible,
            "json" => ConfigurationFormat::JSON,
            "yaml" | "yml" => ConfigurationFormat::YAML,
            _ => {
                println!("❌ Unknown format: {}. Use: nixos, terraform, ansible, json, yaml", format);
                return Ok(());
            }
        };

        let response = self.agent.process_command(TopologyCommand::GenerateConfiguration {
            format: config_format,
        }).await?;

        self.display_response(&response);
        Ok(())
    }

    /// Remove a location
    async fn remove_location(&mut self, location_id: &str) -> Result<(), NetworkError> {
        let response = self.agent.process_command(TopologyCommand::RemoveLocation {
            location_id: location_id.to_string(),
        }).await?;

        self.display_response(&response);
        Ok(())
    }

    /// Remove a connection
    async fn remove_connection(&mut self, args: &str) -> Result<(), NetworkError> {
        let parts: Vec<&str> = args.split_whitespace().collect();
        if parts.len() != 2 {
            println!("❌ Usage: remove-connection <from> <to>");
            return Ok(());
        }

        let from = parts[0].to_string();
        let to = parts[1].to_string();

        let response = self.agent.process_command(TopologyCommand::RemoveConnection {
            from,
            to,
        }).await?;

        self.display_response(&response);
        Ok(())
    }

    /// Display agent response
    fn display_response(&self, response: &super::network_topology_builder::AgentResponse) {
        println!("{}", response.message);
        
        if response.topology_summary.location_count > 0 || response.topology_summary.connection_count > 0 {
            println!();
            println!("📊 Topology Summary:");
            println!("  Locations: {}", response.topology_summary.location_count);
            println!("  Connections: {}", response.topology_summary.connection_count);
            
            if !response.topology_summary.locations.is_empty() {
                println!("  📍 Locations:");
                for (id, desc) in &response.topology_summary.locations {
                    println!("    - {}: {}", id, desc);
                }
            }
            
            if !response.topology_summary.connections.is_empty() {
                println!("  🔗 Connections:");
                for conn in &response.topology_summary.connections {
                    println!("    - {} → {} ({}): {}", conn.from, conn.to, conn.connection_type, conn.details);
                }
            }
        }

        if !response.suggested_actions.is_empty() {
            println!();
            println!("💡 Suggested actions:");
            for action in &response.suggested_actions {
                println!("  • {}", action);
            }
        }

        if !response.events.is_empty() {
            println!();
            println!("📝 Generated {} event(s)", response.events.len());
        }

        println!();
    }
}

impl Default for NetworkTopologyCLI {
    fn default() -> Self {
        Self::new()
    }
}