# CIM Network Base Topologies

This document describes the two base network configurations available in the CIM Network SDN system:

## Dev Mode Configuration
**Single machine, single ISP, 1 public IP**

### Overview
Dev mode is designed for development environments, small offices, or home labs where simplicity and cost-effectiveness are priorities.

### Architecture Components
- **1 Router/Gateway**: Edge device handling NAT, firewall, and DHCP
- **1 Switch**: 8-port access switch for local connectivity  
- **1 Development Machine**: Primary workstation
- **1 ISP Connection**: Single WAN interface with 1 public IP

### Network Layout
```
Internet (ISP) → Router (NAT/Firewall) → Switch → Dev Machine
                      │
                      └─ DHCP Server (192.168.1.0/24)
```

### Key Features
- **Network Range**: 192.168.1.0/24 (home/small office standard)
- **WAN Interface**: wan0 (DHCP from ISP)  
- **LAN Interface**: lan0 (192.168.1.1/24)
- **Services**: NAT, firewall, DHCP server, SSH
- **Port Count**: 8 ports on access switch
- **Public IPs**: 1 (shared via NAT)

### Use Cases
- Development environments
- Home offices
- Small business networks
- Testing and experimentation
- Learning network administration

## Leaf Mode Configuration
**Dual ISPs with failover, 16 public IPs**

### Overview
Leaf mode is designed for production environments, branch offices, or enterprises requiring high availability and multiple public IP addresses.

### Architecture Components
- **1 High-Availability Router**: Enterprise-grade edge device with dual WAN
- **1 Distribution Switch**: 24-port enterprise switch with VLAN support
- **2 ISP Connections**: Primary and failover WAN interfaces
- **16 Public IP Addresses**: 8 per ISP for load balancing

### Network Layout
```
Primary ISP   → Router (HA/Load Balancer) → 24-Port Switch → Enterprise Network
Failover ISP  →        │                           │
                       │                           └─ VLAN Support
                       └─ Keepalived + BGP (10.0.1.0/24)
```

### Key Features
- **Network Range**: 10.0.1.0/24 (enterprise standard)
- **WAN Interfaces**: wan0 (primary) + wan1 (failover)
- **LAN Interface**: lan0 (10.0.1.1/24)
- **High Availability**: Active-passive failover (30s intervals)
- **Services**: NAT, firewall, DHCP, keepalived, bird2 (BGP)
- **Port Count**: 24 ports on distribution switch
- **Public IPs**: 16 total (8 per ISP)
- **Advanced Features**: VLAN support, LACP, load balancing

### Failover Configuration
- **Mode**: Active-passive
- **Health Check**: 30-second intervals
- **Failover Timeout**: 60 seconds
- **Primary Weight**: 100
- **Failover Weight**: 10

### Use Cases
- Branch office networks
- Production environments
- Enterprise deployments
- High-availability requirements
- Multiple public services hosting

## Comparison Table

| Feature | Dev Mode | Leaf Mode |
|---------|----------|-----------|
| ISP Connections | 1 | 2 (with failover) |
| Public IP Addresses | 1 | 16 |
| Network Range | 192.168.1.0/24 | 10.0.1.0/24 |
| Switch Ports | 8 | 24 |
| High Availability | No | Yes (keepalived + BGP) |
| VLAN Support | No | Yes |
| Target Environment | Development | Production |
| Complexity | Low | High |
| Cost | Low | High |

## Generated Nix Configurations

Both base topologies generate nix-topology compliant NixOS configurations that include:

### Dev Mode Nix Features
- Single WAN interface (wan0) with DHCP
- LAN interface with static IP (192.168.1.1/24)
- DHCP server for 192.168.1.100-200 range
- Basic NAT and firewall rules
- Development tools (git, vim, docker, etc.)
- SSH access enabled

### Leaf Mode Nix Features
- Dual WAN interfaces (wan0, wan1) with DHCP
- Enterprise LAN interface (10.0.1.1/24)
- Advanced DHCP server for 10.0.1.100-200 range
- High-availability services (keepalived, bird2)
- Advanced firewall rules with load balancing
- Enterprise-grade routing and failover logic

## Usage with CIM Network MCP Server

### Creating a Dev Mode Topology
```json
{
  "name": "create_base_topology",
  "arguments": {
    "mode": "dev",
    "name": "my-dev-network",
    "primary_isp": "comcast"
  }
}
```

### Creating a Leaf Mode Topology
```json
{
  "name": "create_base_topology", 
  "arguments": {
    "mode": "leaf",
    "name": "my-leaf-network",
    "primary_isp": "verizon",
    "failover_isp": "att"
  }
}
```

### Generating Mode-Specific Nix Configuration
```json
{
  "name": "generate_nix_topology",
  "arguments": {
    "format": "nixos",
    "mode": "dev" // or "leaf"
  }
}
```

## Integration with Claude Code

Add this MCP server configuration to your Claude Code settings:

```json
{
  "mcpServers": {
    "network-topology-builder": {
      "command": "python3",
      "args": ["-m", "cim_network_mcp"],
      "cwd": "/path/to/cim-network",
      "env": {
        "PYTHONPATH": "/path/to/cim-network"
      }
    }
  }
}
```

Then use Claude Code to interactively build network topologies using the base configurations as starting points.

## Architecture Benefits

### Clean Separation of Concerns
- **Dev Mode**: Optimized for simplicity and development workflow
- **Leaf Mode**: Optimized for production reliability and scalability

### Standardized Deployment Patterns
- Consistent network layouts across environments
- Predictable IP addressing schemes
- Standard service configurations

### Scalability Path
- Start with dev mode for development
- Graduate to leaf mode for production
- Maintain configuration compatibility

### Infrastructure as Code
- Fully declarative NixOS configurations
- Version-controlled network infrastructure
- Reproducible deployments across environments

The base topology system provides a solid foundation for building more complex network architectures while maintaining best practices and operational simplicity.