# CIM Network Topology Builder MCP Server

This is a Model Context Protocol (MCP) server that provides Claude Code with interactive tools for building network topologies using event-driven context graphs.

## Features

🌐 **Interactive Network Topology Building**
- Add network locations: data centers, offices, cloud regions, edge locations, virtual segments
- Connect locations with various connection types: fiber, VPN, internet, direct connect, virtual
- Event-driven architecture with full audit trail

⚙️ **Comprehensive Nix Configuration Generation**
- **NixOS**: System configurations for Linux servers and workstations
- **nix-darwin**: System configurations for macOS systems
- **Home Manager**: User environment configurations across all platforms
- **Nix Flakes**: Modern, reproducible, multi-platform configurations
- **JSON**: Debug and inspection format for topology analysis

🔍 **Intelligent Validation & Suggestions**
- Topology completeness validation
- Smart suggestions based on current state
- Real-time status monitoring

## Installation

### Using Nix Flake (Recommended)

1. **Enter the development environment:**
   ```bash
   nix develop
   ```

2. **Or install directly:**
   ```bash
   nix build .#mcp-server
   ```

3. **Configure Claude Code** by adding this to your MCP settings:
   ```json
   {
     "mcpServers": {
       "network-topology-builder": {
         "command": "python3",
         "args": ["-m", "cim_network_mcp"],
         "cwd": "/git/thecowboyai/cim-network",
         "env": {
           "PYTHONPATH": "/git/thecowboyai/cim-network"
         }
       }
     }
   }
   ```

### NixOS System Integration

For system-wide installation on NixOS:

```nix
{
  inputs.cim-network.url = "github:thecowboyai/cim-network";
  
  outputs = { self, nixpkgs, cim-network }: {
    nixosConfigurations.myhost = nixpkgs.lib.nixosSystem {
      modules = [
        cim-network.nixosModules.default
        {
          services.cim-network-mcp.enable = true;
        }
      ];
    };
  };
}
```

### Development

```bash
# Enter development shell
nix develop

# Build Rust components  
cargo build --release --example subagent_demo

# Test MCP server
python3 -m cim_network_mcp
```

## Available Tools

### `build_topology`
Start building a new network topology with optional initial parameters.

**Parameters:**
- `base_network` (optional): Base IP network range (e.g., "10.0.0.0/8")
- `target_environment` (optional): Target deployment environment
- `scale` (optional): Expected scale (small/medium/large/enterprise)
- `use_case` (optional): Primary use case description

### `add_location`
Add a network location to the topology.

**Parameters:**
- `location_id` (required): Unique identifier for the location
- `location_type` (required): One of: `datacenter`, `office`, `cloud`, `edge`, `segment`
- `parameters` (optional): Location-specific parameters

**Location Type Parameters:**
- **datacenter**: `name`, `region`, `az` (availability zone)
- **office**: `name`, `address`, `size` (small/medium/large/campus)
- **cloud**: `provider` (aws/azure/gcp/digitalocean), `region`
- **edge**: `name`, `lat`, `lng` (latitude/longitude)
- **segment**: `name`, `subnet`, `vlan` (VLAN ID)

### `connect_locations`
Connect two locations in the topology.

**Parameters:**
- `from_location` (required): Source location ID
- `to_location` (required): Destination location ID
- `connection_type` (required): One of: `fiber`, `vpn`, `internet`, `directconnect`, `virtual`
- `parameters` (optional): Connection-specific parameters

**Connection Type Parameters:**
- **fiber**: `bandwidth`, `redundant` (true/false)
- **vpn**: `protocol` (ipsec/wireguard/openvpn), `encrypted` (true/false)
- **internet**: `bandwidth`, `provider`
- **directconnect**: `provider`, `bandwidth`
- **virtual**: `protocol`, `bandwidth`

### `generate_configuration`
Generate network configuration in specified format.

**Parameters:**
- `format` (required): One of: `nixos`, `nix-darwin`, `home-manager`, `flake`, `json`

### `validate_topology`
Validate the current network topology for completeness and correctness.

### `get_topology_status`
Get current topology status and summary.

### `reset_topology`
Reset the current topology to start over.

### `complete_topology`
Complete the topology building process and finalize the configuration.

## Usage Examples

### Build a Multi-Region Cloud Topology

```python
# Start building
await call_tool("build_topology", {
    "base_network": "10.0.0.0/8",
    "target_environment": "production",
    "scale": "enterprise"
})

# Add locations
await call_tool("add_location", {
    "location_id": "dc-west",
    "location_type": "datacenter",
    "parameters": {"name": "West Coast DC", "region": "us-west-1"}
})

await call_tool("add_location", {
    "location_id": "aws-east",
    "location_type": "cloud",
    "parameters": {"provider": "aws", "region": "us-east-1"}
})

# Connect them
await call_tool("connect_locations", {
    "from_location": "dc-west",
    "to_location": "aws-east",
    "connection_type": "directconnect",
    "parameters": {"provider": "aws", "bandwidth": "10Gbps"}
})

# Generate configuration
await call_tool("generate_configuration", {"format": "nixos"})
```

## Architecture

The MCP server is built on top of the Rust-based `NetworkTopologySubAgent`, which uses:

- **Event-Driven Architecture**: All topology changes are recorded as events
- **Context Graphs**: Network topology is modeled as semantic graphs using `cim-graph`
- **Domain-Driven Design**: Clean separation of concerns and rich domain modeling
- **Multiple Output Formats**: Flexible configuration generation

## Development

The MCP server bridges between Claude Code's Python environment and the high-performance Rust core, providing the best of both worlds: easy integration and powerful topology modeling.

**Key Components:**
- `cim_network_mcp/server.py`: MCP server implementation
- `src/agents/subagent.rs`: Rust sub-agent core
- `examples/subagent_demo.rs`: Command-line interface for the Rust agent
- `.claude/mcp_settings.json`: Claude Code MCP configuration