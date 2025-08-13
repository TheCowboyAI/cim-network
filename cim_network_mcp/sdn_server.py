#!/usr/bin/env python3
"""
CIM Network SDN MCP Server

Simplified MCP server focused on the core goal:
1. Start from a domain established in cim-start
2. Build Software Defined Network using cim-graph ContextGraph  
3. Generate nix-topology compliant Nix files as projections

This version removes the complex workflow dependencies and focuses on the SDN approach.
"""

import json
import sys
import asyncio
import subprocess
import tempfile
import uuid
from pathlib import Path
from typing import Dict, Any, List, Optional
try:
    from .advanced_nix_generator import AdvancedNixGenerator, NetworkConfig, NetworkMode, SecurityLevel
    ADVANCED_GENERATOR_AVAILABLE = True
except ImportError:
    ADVANCED_GENERATOR_AVAILABLE = False


class SDNMCPServer:
    """Simplified SDN-focused MCP server"""
    
    def __init__(self):
        self.project_root = Path(__file__).parent.parent
        self.tools = self._define_tools()
    
    def _define_tools(self) -> List[Dict[str, Any]]:
        """Define available SDN tools"""
        return [
            {
                "name": "initialize_sdn",
                "description": "Initialize SDN from a domain context (from cim-start)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "domain_context": {"type": "object", "description": "Domain context from cim-start"},
                        "base_config": {"type": "string", "enum": ["dev", "leaf"], "default": "dev", "description": "Base network configuration"}
                    }
                }
            },
            {
                "name": "create_base_topology",
                "description": "Create a base network topology (dev or leaf mode)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "mode": {"type": "string", "enum": ["dev", "leaf"], "description": "Network topology mode"},
                        "name": {"type": "string", "description": "Network name"},
                        "primary_isp": {"type": "string", "description": "Primary ISP name"},
                        "failover_isp": {"type": "string", "description": "Failover ISP name (leaf mode only)"}
                    },
                    "required": ["mode", "name", "primary_isp"]
                }
            },
            {
                "name": "add_sdn_node",
                "description": "Add a node to the Software Defined Network",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "node_id": {"type": "string", "description": "Unique node identifier"},
                        "node_type": {"type": "string", "enum": ["server", "workstation", "gateway", "switch"]},
                        "tier": {"type": "string", "enum": ["client", "leaf", "cluster", "super-cluster"]},
                        "interfaces": {"type": "array", "description": "Network interfaces"},
                        "services": {"type": "array", "description": "Services running on this node"},
                        "metadata": {"type": "object", "additionalProperties": {"type": "string"}}
                    },
                    "required": ["node_id", "node_type", "tier"]
                }
            },
            {
                "name": "connect_sdn_nodes",
                "description": "Connect two nodes in the SDN",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "from_node": {"type": "string", "description": "Source node ID"},
                        "to_node": {"type": "string", "description": "Destination node ID"},
                        "connection_type": {"type": "string", "description": "Type of connection"},
                        "properties": {"type": "object", "additionalProperties": {"type": "string"}}
                    },
                    "required": ["from_node", "to_node", "connection_type"]
                }
            },
            {
                "name": "generate_nix_topology",
                "description": "Generate nix-topology compliant Nix configuration from SDN state",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "format": {"type": "string", "enum": ["nixos", "nix-darwin", "home-manager", "flake"], "default": "nixos"},
                        "mode": {"type": "string", "enum": ["dev", "leaf"], "default": "dev", "description": "Base topology mode for configuration"}
                    }
                }
            },
            {
                "name": "get_sdn_state",
                "description": "Get the current state of the Software Defined Network",
                "inputSchema": {"type": "object", "properties": {}}
            },
            {
                "name": "export_context_graph",
                "description": "Export the cim-graph ContextGraph representing the SDN",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "format": {"type": "string", "enum": ["json", "dot", "cypher"], "default": "json"}
                    }
                }
            },
            {
                "name": "visualize_topology",
                "description": "Generate visual representation of the network topology",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "format": {"type": "string", "enum": ["ascii", "mermaid", "dot", "svg"], "default": "ascii"},
                        "layout": {"type": "string", "enum": ["hierarchical", "tier-based", "force-directed"], "default": "tier-based"},
                        "color_scheme": {"type": "string", "enum": ["default", "dark", "blue", "enterprise"], "default": "default"},
                        "show_details": {"type": "boolean", "default": True}
                    }
                }
            },
            {
                "name": "generate_advanced_nix",
                "description": "Generate advanced nix-topology configurations with security, monitoring, and enterprise features",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "mode": {"type": "string", "enum": ["dev", "leaf", "enterprise", "secure"], "default": "dev"},
                        "security_level": {"type": "string", "enum": ["basic", "hardened", "compliance"], "default": "basic"},
                        "enable_monitoring": {"type": "boolean", "default": True},
                        "enable_vpn": {"type": "boolean", "default": False},
                        "enable_vlan": {"type": "boolean", "default": False},
                        "container_support": {"type": "boolean", "default": True},
                        "high_availability": {"type": "boolean", "default": False},
                        "network_cidr": {"type": "string", "default": "192.168.1.0/24"},
                        "domain_name": {"type": "string", "default": "local.network"},
                        "format": {"type": "string", "enum": ["nixos", "nix-darwin", "home-manager"], "default": "nixos"}
                    }
                }
            }
        ]
    
    async def execute_sdn_command(self, command: str, args: Dict[str, Any]) -> Dict[str, Any]:
        """Execute an SDN command via a simplified interface"""
        try:
            # For now, create mock responses that demonstrate the SDN approach
            # In a real implementation, this would call the Rust SDN components
            
            if command == "initialize_sdn":
                base_config = args.get("base_config", "dev")
                return {
                    "success": True,
                    "message": f"🌐 SDN initialized from domain context ({base_config} mode)",
                    "data": {
                        "sdn_id": str(uuid.uuid4()),
                        "context_graph_id": str(uuid.uuid4()),
                        "initialized_from": "cim-start domain",
                        "base_config": base_config
                    }
                }
            
            elif command == "create_base_topology":
                mode = args.get("mode", "dev")
                name = args.get("name", "network")
                primary_isp = args.get("primary_isp", "isp1")
                failover_isp = args.get("failover_isp")
                
                if mode == "dev":
                    topology_data = self._create_dev_topology(name, primary_isp)
                elif mode == "leaf":
                    topology_data = self._create_leaf_topology(name, primary_isp, failover_isp)
                else:
                    raise ValueError(f"Unknown topology mode: {mode}")
                
                return {
                    "success": True,
                    "message": f"🏗️ Created {mode} mode base topology '{name}'",
                    "data": topology_data
                }
            
            elif command == "add_sdn_node":
                node_id = args.get("node_id", "unknown")
                node_type = args.get("node_type", "server")
                tier = args.get("tier", "leaf")
                
                return {
                    "success": True,
                    "message": f"✅ Added SDN node '{node_id}' ({node_type}/{tier})",
                    "data": {
                        "node_id": node_id,
                        "node_type": node_type,
                        "tier": tier,
                        "added_to_context_graph": True,
                        "nix_integration": "ready"
                    }
                }
            
            elif command == "connect_sdn_nodes":
                from_node = args.get("from_node")
                to_node = args.get("to_node")
                connection_type = args.get("connection_type")
                
                return {
                    "success": True,
                    "message": f"🔗 Connected {from_node} → {to_node} ({connection_type})",
                    "data": {
                        "connection_id": str(uuid.uuid4()),
                        "from_node": from_node,
                        "to_node": to_node,
                        "connection_type": connection_type,
                        "added_to_context_graph": True
                    }
                }
            
            elif command == "generate_nix_topology":
                format_type = args.get("format", "nixos")
                mode = args.get("mode", "dev")  # Support mode-specific generation
                
                # Generate a nix-topology compliant configuration
                nix_config = self._generate_sample_nix_config(format_type, mode)
                
                return {
                    "success": True,
                    "message": f"🔧 Generated {format_type} topology configuration ({mode} mode)",
                    "data": {
                        "format": format_type,
                        "mode": mode,
                        "nix_topology_compliant": True,
                        "configuration": nix_config,
                        "projected_from": "cim-graph ContextGraph"
                    }
                }
            
            elif command == "get_sdn_state":
                return {
                    "success": True,
                    "message": "📊 Current SDN state",
                    "data": {
                        "nodes": {
                            "server-01": {"type": "server", "tier": "cluster", "status": "active"},
                            "gateway-01": {"type": "gateway", "tier": "leaf", "status": "active"},
                        },
                        "connections": {
                            "conn-1": {"from": "server-01", "to": "gateway-01", "type": "ethernet"}
                        },
                        "context_graph_nodes": 2,
                        "context_graph_edges": 1
                    }
                }
            
            elif command == "export_context_graph":
                format_type = args.get("format", "json")
                
                # Generate a sample context graph export
                graph_export = self._generate_sample_context_graph(format_type)
                
                return {
                    "success": True,
                    "message": f"📈 Exported ContextGraph as {format_type}",
                    "data": {
                        "format": format_type,
                        "context_graph": graph_export,
                        "cim_graph_compliant": True
                    }
                }
            
            elif command == "visualize_topology":
                format_type = args.get("format", "ascii")
                layout = args.get("layout", "tier-based")
                color_scheme = args.get("color_scheme", "default")
                show_details = args.get("show_details", True)
                
                # Generate topology visualization
                visualization = self._generate_topology_visualization(
                    format_type, layout, color_scheme, show_details
                )
                
                return {
                    "success": True,
                    "message": f"📊 Generated {format_type} topology visualization ({layout} layout)",
                    "data": {
                        "format": format_type,
                        "layout": layout,
                        "color_scheme": color_scheme,
                        "visualization": visualization,
                        "interactive": format_type in ["svg", "html"]
                    }
                }
            
            elif command == "generate_advanced_nix":
                if not ADVANCED_GENERATOR_AVAILABLE:
                    return {
                        "success": False,
                        "message": "❌ Advanced Nix generator not available. Please check installation.",
                        "data": {}
                    }
                
                try:
                    # Parse parameters
                    mode = NetworkMode(args.get("mode", "dev"))
                    security_level = SecurityLevel(args.get("security_level", "basic"))
                    
                    config = NetworkConfig(
                        mode=mode,
                        security_level=security_level,
                        enable_monitoring=args.get("enable_monitoring", True),
                        enable_vpn=args.get("enable_vpn", False),
                        enable_vlan=args.get("enable_vlan", False),
                        container_support=args.get("container_support", True),
                        high_availability=args.get("high_availability", False),
                        network_cidr=args.get("network_cidr", "192.168.1.0/24"),
                        domain_name=args.get("domain_name", "local.network")
                    )
                    
                    format_type = args.get("format", "nixos")
                    generator = AdvancedNixGenerator()
                    flake_content = generator.generate_flake(config, format_type)
                    
                    # Generate feature summary
                    features = []
                    if config.enable_monitoring:
                        features.append("monitoring")
                    if config.enable_vpn:
                        features.append("VPN")
                    if config.enable_vlan:
                        features.append("VLAN")
                    if config.container_support:
                        features.append("containers")
                    if config.high_availability:
                        features.append("high-availability")
                    if config.security_level != SecurityLevel.BASIC:
                        features.append(f"{config.security_level.value}-security")
                    
                    return {
                        "success": True,
                        "message": f"🚀 Generated advanced {format_type} configuration ({config.mode.value} mode)",
                        "data": {
                            "mode": config.mode.value,
                            "security_level": config.security_level.value,
                            "format": format_type,
                            "features": features,
                            "network_cidr": config.network_cidr,
                            "domain_name": config.domain_name,
                            "configuration": flake_content,
                            "deployment_ready": True,
                            "nix_topology_compliant": True
                        }
                    }
                    
                except ValueError as e:
                    return {
                        "success": False,
                        "message": f"❌ Invalid configuration parameter: {e}",
                        "data": {}
                    }
                except Exception as e:
                    return {
                        "success": False,
                        "message": f"❌ Error generating advanced configuration: {e}",
                        "data": {}
                    }
            
            else:
                return {
                    "success": False,
                    "message": f"Unknown SDN command: {command}",
                    "data": {}
                }
                
        except Exception as e:
            return {
                "success": False,
                "message": f"SDN command failed: {str(e)}",
                "data": {},
                "error": str(e)
            }
    
    def _generate_sample_nix_config(self, format_type: str, mode: str = "dev") -> str:
        """Generate a sample nix-topology compliant configuration"""
        if format_type == "nixos" and mode == "dev":
            return '''# Generated NixOS Network Topology (nix-topology compliant)
# Base Topology: Development Mode
{
  description = "CIM SDN Network Topology - Dev Mode";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-topology.url = "github:oddlama/nix-topology";
  };
  
  outputs = { self, nixpkgs, nix-topology }: {
    nixosConfigurations = {
      # Development Router/Gateway
      dev-network-router = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          {
            networking.hostName = "dev-network-router";
            networking.firewall.enable = true;
            networking.nat = {
              enable = true;
              externalInterface = "wan0";
              internalIPs = [ "192.168.1.0/24" ];
            };
            
            # WAN interface (single ISP)
            networking.interfaces.wan0 = {
              useDHCP = true; # Get IP from ISP
            };
            
            # LAN interface
            networking.interfaces.lan0.ipv4.addresses = [{
              address = "192.168.1.1";
              prefixLength = 24;
            }];
            
            # DHCP server for LAN
            services.dhcpd4 = {
              enable = true;
              interfaces = [ "lan0" ];
              extraConfig = ''
                subnet 192.168.1.0 netmask 255.255.255.0 {
                  range 192.168.1.100 192.168.1.200;
                  option routers 192.168.1.1;
                  option domain-name-servers 8.8.8.8, 1.1.1.1;
                }
              '';
            };
            
            services.openssh.enable = true;
          }
        ];
      };
      
      # Development Workstation
      dev-network-dev-machine = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          {
            networking.hostName = "dev-network-dev-machine";
            networking.interfaces.eth0.useDHCP = true;
            
            # Development environment
            environment.systemPackages = with nixpkgs; [
              git vim curl wget docker
            ];
            
            services.openssh.enable = true;
            virtualisation.docker.enable = true;
          }
        ];
      };
    };
    
    # nix-topology integration for dev mode
    topology = nix-topology.lib.mkTopology {
      nodes = {
        dev-network-router = { 
          deviceType = "gateway";
          interfaces.wan0 = {
            addresses = [ "dhcp" ];
            network = "wan";
          };
          interfaces.lan0 = {
            addresses = [ "192.168.1.1/24" ];
            network = "lan";
          };
        };
        dev-network-switch = { 
          deviceType = "switch";
          interfaces.uplink0 = {
            addresses = [ "192.168.1.2/24" ];
            network = "lan";
          };
        };
        dev-network-dev-machine = { 
          deviceType = "workstation";
          interfaces.eth0 = {
            addresses = [ "dhcp" ];
            network = "lan";
          };
        };
      };
      
      networks = {
        wan = { cidr = "0.0.0.0/0"; };
        lan = { cidr = "192.168.1.0/24"; };
      };
      
      connections = {
        router-to-switch = {
          from = "dev-network-router";
          to = "dev-network-switch";
          type = "ethernet";
        };
        switch-to-machine = {
          from = "dev-network-switch";
          to = "dev-network-dev-machine";
          type = "ethernet";
        };
      };
    };
  };
}'''
        elif format_type == "nixos" and mode == "leaf":
            return '''# Generated NixOS Network Topology (nix-topology compliant)
# Base Topology: Leaf Mode - Dual ISPs with Failover
{
  description = "CIM SDN Network Topology - Leaf Mode";
  
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    nix-topology.url = "github:oddlama/nix-topology";
  };
  
  outputs = { self, nixpkgs, nix-topology }: {
    nixosConfigurations = {
      # High-Availability Router/Gateway with Dual ISPs
      leaf-network-router = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";
        modules = [
          {
            networking.hostName = "leaf-network-router";
            networking.firewall.enable = true;
            
            # Dual WAN configuration with failover
            networking.interfaces.wan0 = {
              useDHCP = true; # Primary ISP
            };
            networking.interfaces.wan1 = {
              useDHCP = true; # Failover ISP  
            };
            
            # LAN interface for enterprise network
            networking.interfaces.lan0.ipv4.addresses = [{
              address = "10.0.1.1";
              prefixLength = 24;
            }];
            
            # Advanced NAT with load balancing
            networking.nat = {
              enable = true;
              externalInterface = "wan0";
              internalIPs = [ "10.0.1.0/24" ];
            };
            
            # DHCP server for enterprise LAN
            services.dhcpd4 = {
              enable = true;
              interfaces = [ "lan0" ];
              extraConfig = ''
                subnet 10.0.1.0 netmask 255.255.255.0 {
                  range 10.0.1.100 10.0.1.200;
                  option routers 10.0.1.1;
                  option domain-name-servers 8.8.8.8, 1.1.1.1;
                }
              '';
            };
            
            # High availability services
            services.keepalived.enable = true;
            services.bird2.enable = true; # BGP for advanced routing
            
            services.openssh.enable = true;
            
            # Enterprise firewall rules
            networking.firewall.extraCommands = ''
              # Allow failover traffic
              iptables -A INPUT -i wan1 -j ACCEPT
              # Load balancing rules for 16 public IPs
              for i in {2..17}; do
                iptables -t nat -A PREROUTING -d 10.0.1.$i -j DNAT --to-destination 10.0.1.$i
              done
            '';
          }
        ];
      };
    };
    
    # nix-topology integration for leaf mode
    topology = nix-topology.lib.mkTopology {
      nodes = {
        leaf-network-router = { 
          deviceType = "gateway";
          interfaces.wan0 = {
            addresses = [ "dhcp" ];
            network = "wan_primary";
          };
          interfaces.wan1 = {
            addresses = [ "dhcp" ];
            network = "wan_failover";
          };
          interfaces.lan0 = {
            addresses = [ "10.0.1.1/24" ];
            network = "lan";
          };
        };
        leaf-network-switch = { 
          deviceType = "switch";
          interfaces.uplink0 = {
            addresses = [ "10.0.1.2/24" ];
            network = "lan";
          };
        };
      };
      
      networks = {
        wan_primary = { cidr = "0.0.0.0/0"; };
        wan_failover = { cidr = "0.0.0.0/0"; };
        lan = { cidr = "10.0.1.0/24"; };
      };
      
      connections = {
        router-to-primary-isp = {
          from = "leaf-network-router";
          to = "external-primary-isp";
          type = "wan";
          properties = { priority = "primary"; };
        };
        router-to-failover-isp = {
          from = "leaf-network-router";
          to = "external-failover-isp";
          type = "wan";
          properties = { priority = "failover"; };
        };
        router-to-switch = {
          from = "leaf-network-router";
          to = "leaf-network-switch";
          type = "ethernet";
          properties = { bandwidth = "10Gbps"; };
        };
      };
    };
  };
}'''
        else:
            return f'''# Generated {format_type} configuration (nix-topology compliant)
# Projected from cim-graph ContextGraph representing SDN state
{{ config, pkgs, ... }}: {{
  # SDN node configuration
  networking.hostName = "sdn-node";
  services.openssh.enable = true;
}}'''
    
    def _generate_sample_context_graph(self, format_type: str) -> str:
        """Generate a sample context graph export"""
        if format_type == "json":
            return json.dumps({
                "graph_type": "ContextGraph",
                "nodes": [
                    {
                        "id": "server-01",
                        "type": "SDNNode",
                        "properties": {
                            "node_type": "server",
                            "tier": "cluster",
                            "interfaces": ["eth0"],
                            "services": ["openssh", "networking"]
                        }
                    },
                    {
                        "id": "gateway-01", 
                        "type": "SDNNode",
                        "properties": {
                            "node_type": "gateway",
                            "tier": "leaf",
                            "interfaces": ["eth0"],
                            "services": ["nat", "firewall"]
                        }
                    }
                ],
                "edges": [
                    {
                        "id": "conn-1",
                        "from": "server-01",
                        "to": "gateway-01",
                        "type": "SDNConnection",
                        "properties": {
                            "connection_type": "ethernet",
                            "bandwidth": "1Gbps"
                        }
                    }
                ],
                "metadata": {
                    "created_from": "cim-start domain",
                    "projected_to": "nix-topology",
                    "cim_graph_version": "1.0"
                }
            }, indent=2)
        elif format_type == "dot":
            return '''digraph SDN {
  rankdir=LR;
  
  "server-01" [label="Server\\n(Cluster Tier)" shape=box];
  "gateway-01" [label="Gateway\\n(Leaf Tier)" shape=diamond];
  
  "server-01" -> "gateway-01" [label="ethernet\\n1Gbps"];
}'''
        else:
            return "// Cypher format not implemented yet"
    
    def _create_dev_topology(self, name: str, primary_isp: str) -> Dict[str, Any]:
        """Create dev mode base topology: single machine, single ISP, 1 public IP"""
        return {
            "topology_id": str(uuid.uuid4()),
            "name": name,
            "mode": "dev",
            "description": "Development mode: single machine, single ISP, 1 public IP",
            "nodes": {
                "router": {
                    "id": f"{name}-router",
                    "type": "gateway",
                    "tier": "leaf",
                    "interfaces": [
                        {
                            "name": "wan0",
                            "type": "ethernet",
                            "addresses": ["dhcp"],
                            "isp": primary_isp,
                            "public": True
                        },
                        {
                            "name": "lan0", 
                            "type": "ethernet",
                            "addresses": ["192.168.1.1/24"],
                            "network": "lan"
                        }
                    ],
                    "services": ["nat", "firewall", "dhcp-server"],
                    "metadata": {
                        "role": "edge-router",
                        "primary_isp": primary_isp,
                        "public_ip_count": "1"
                    }
                },
                "switch": {
                    "id": f"{name}-switch",
                    "type": "switch",
                    "tier": "leaf", 
                    "interfaces": [
                        {
                            "name": "uplink0",
                            "type": "ethernet",
                            "addresses": ["192.168.1.2/24"]
                        },
                        {
                            "name": "port1-8",
                            "type": "ethernet-multi",
                            "addresses": ["bridge"],
                            "port_count": 8
                        }
                    ],
                    "services": ["bridge", "stp"],
                    "metadata": {
                        "role": "access-switch",
                        "port_count": "8"
                    }
                },
                "dev-machine": {
                    "id": f"{name}-dev-machine", 
                    "type": "workstation",
                    "tier": "client",
                    "interfaces": [
                        {
                            "name": "eth0",
                            "type": "ethernet", 
                            "addresses": ["dhcp"]
                        }
                    ],
                    "services": ["networkmanager", "openssh"],
                    "metadata": {
                        "role": "development-workstation",
                        "environment": "development"
                    }
                }
            },
            "connections": {
                "router-to-isp": {
                    "from": f"{name}-router",
                    "to": f"external-{primary_isp}",
                    "interface_from": "wan0",
                    "type": "wan",
                    "properties": {"bandwidth": "auto", "isp": primary_isp}
                },
                "router-to-switch": {
                    "from": f"{name}-router", 
                    "to": f"{name}-switch",
                    "interface_from": "lan0",
                    "interface_to": "uplink0",
                    "type": "ethernet",
                    "properties": {"bandwidth": "1Gbps", "network": "lan"}
                },
                "switch-to-machine": {
                    "from": f"{name}-switch",
                    "to": f"{name}-dev-machine",
                    "interface_from": "port1",
                    "interface_to": "eth0", 
                    "type": "ethernet",
                    "properties": {"bandwidth": "1Gbps", "port": "1"}
                }
            },
            "networks": {
                "wan": {"subnet": "dhcp", "isp": primary_isp, "public_ips": 1},
                "lan": {"subnet": "192.168.1.0/24", "gateway": "192.168.1.1"}
            }
        }
    
    def _create_leaf_topology(self, name: str, primary_isp: str, failover_isp: str) -> Dict[str, Any]:
        """Create leaf mode base topology: dual ISPs with failover, 16 public IPs"""
        return {
            "topology_id": str(uuid.uuid4()),
            "name": name,
            "mode": "leaf",
            "description": "Leaf mode: dual ISPs with failover, 16 public IP addresses",
            "nodes": {
                "router": {
                    "id": f"{name}-router",
                    "type": "gateway", 
                    "tier": "leaf",
                    "interfaces": [
                        {
                            "name": "wan0",
                            "type": "ethernet",
                            "addresses": ["dhcp"],
                            "isp": primary_isp,
                            "public": True,
                            "priority": "primary"
                        },
                        {
                            "name": "wan1", 
                            "type": "ethernet",
                            "addresses": ["dhcp"],
                            "isp": failover_isp or "failover-isp",
                            "public": True,
                            "priority": "failover"
                        },
                        {
                            "name": "lan0",
                            "type": "ethernet", 
                            "addresses": ["10.0.1.1/24"],
                            "network": "lan"
                        }
                    ],
                    "services": ["nat", "firewall", "dhcp-server", "failover", "load-balancer"],
                    "metadata": {
                        "role": "edge-router",
                        "primary_isp": primary_isp,
                        "failover_isp": failover_isp or "failover-isp",
                        "public_ip_count": "16",
                        "high_availability": "true"
                    }
                },
                "switch": {
                    "id": f"{name}-switch",
                    "type": "switch",
                    "tier": "leaf",
                    "interfaces": [
                        {
                            "name": "uplink0",
                            "type": "ethernet",
                            "addresses": ["10.0.1.2/24"]
                        },
                        {
                            "name": "port1-24",
                            "type": "ethernet-multi", 
                            "addresses": ["bridge"],
                            "port_count": 24
                        }
                    ],
                    "services": ["bridge", "stp", "vlan", "lacp"],
                    "metadata": {
                        "role": "distribution-switch",
                        "port_count": "24",
                        "vlan_capable": "true"
                    }
                }
            },
            "connections": {
                "router-to-primary-isp": {
                    "from": f"{name}-router",
                    "to": f"external-{primary_isp}",
                    "interface_from": "wan0",
                    "type": "wan",
                    "properties": {"bandwidth": "auto", "isp": primary_isp, "priority": "primary"}
                },
                "router-to-failover-isp": {
                    "from": f"{name}-router", 
                    "to": f"external-{failover_isp or 'failover-isp'}",
                    "interface_from": "wan1",
                    "type": "wan",
                    "properties": {"bandwidth": "auto", "isp": failover_isp or "failover-isp", "priority": "failover"}
                },
                "router-to-switch": {
                    "from": f"{name}-router",
                    "to": f"{name}-switch", 
                    "interface_from": "lan0",
                    "interface_to": "uplink0",
                    "type": "ethernet",
                    "properties": {"bandwidth": "10Gbps", "network": "lan"}
                }
            },
            "networks": {
                "wan_primary": {"subnet": "dhcp", "isp": primary_isp, "public_ips": 8},
                "wan_failover": {"subnet": "dhcp", "isp": failover_isp or "failover-isp", "public_ips": 8}, 
                "lan": {"subnet": "10.0.1.0/24", "gateway": "10.0.1.1"}
            },
            "failover_config": {
                "mode": "active-passive",
                "health_check_interval": "30s",
                "failover_timeout": "60s",
                "primary_weight": 100,
                "failover_weight": 10
            }
        }
    
    def _generate_topology_visualization(self, format_type: str, layout: str, 
                                       color_scheme: str, show_details: bool) -> str:
        """Generate topology visualization in the specified format"""
        
        if format_type == "ascii":
            return self._generate_ascii_topology(layout, color_scheme, show_details)
        elif format_type == "mermaid":
            return self._generate_mermaid_topology(layout, color_scheme, show_details)
        elif format_type == "dot":
            return self._generate_dot_topology(layout, color_scheme, show_details)
        elif format_type == "svg":
            return self._generate_svg_topology(layout, color_scheme, show_details)
        else:
            return f"# Visualization format '{format_type}' not implemented yet"
    
    def _generate_ascii_topology(self, layout: str, color_scheme: str, show_details: bool) -> str:
        """Generate ASCII art representation of the topology"""
        if layout == "tier-based":
            return '''
╔═══════════════════════════════════════════════════════════════╗
║                    NETWORK TOPOLOGY (ASCII)                  ║
╠═══════════════════════════════════════════════════════════════╣
║                          WAN TIER                             ║
╠═══════════════════════════════════════════════════════════════╣
║                                                               ║
║     [ISP-1] ════════════════════╗                             ║
║                                 ║                             ║
║     [ISP-2] ══════════════════╗ ║                             ║
║                               ║ ║                             ║
╠═══════════════════════════════╬═╬═════════════════════════════╣
║                    LEAF TIER  ║ ║                             ║
╠═══════════════════════════════╬═╬═════════════════════════════╣
║                               ║ ║                             ║
║                        ◆ ROUTER-01 ◆ (High Availability)     ║
║                               ║                               ║
║                               ║                               ║
║                          ⬢ SWITCH-01 ⬢                       ║
║                         (24-port VLAN)                        ║
║                               ║                               ║
╠═══════════════════════════════╬═══════════════════════════════╣
║                 CLUSTER TIER  ║                               ║
╠═══════════════════════════════╬═══════════════════════════════╣
║                               ║                               ║
║    ▬ APP-01 ▬ ════════════════╬════════════════ ▬ APP-02 ▬    ║
║                               ║                               ║
║    ▬ DB-PRIMARY ▬ ═══════════╬═══════════ ▬ DB-REPLICA ▬     ║
║                               ║                               ║
╠═══════════════════════════════╬═══════════════════════════════╣
║                 CLIENT TIER   ║                               ║
╠═══════════════════════════════╬═══════════════════════════════╣
║                               ║                               ║
║         ○ WORKSTATION-01 ○ ═══╩══ ○ WORKSTATION-02 ○         ║
║                                                               ║
╚═══════════════════════════════════════════════════════════════╝

LEGEND:
◆ Gateway/Router    ⬢ Switch    ▬ Server    ○ Workstation
═══ Ethernet    ~~~ Wireless    ║ High-bandwidth uplink
'''
        else:
            return '''
NETWORK TOPOLOGY (Flat Layout):
═══════════════════════════════

ISP-1 ────┐
          ├─── ROUTER-01 ─── SWITCH-01 ─┬─── APP-01
ISP-2 ────┘                             ├─── APP-02
                                        ├─── DB-PRIMARY
                                        ├─── DB-REPLICA
                                        └─── WORKSTATIONS...
'''
    
    def _generate_mermaid_topology(self, layout: str, color_scheme: str, show_details: bool) -> str:
        """Generate Mermaid diagram representation"""
        base_colors = {
            "default": {"gateway": "#FF6B6B", "switch": "#4ECDC4", "server": "#45B7D1", "client": "#96CEB4"},
            "blue": {"gateway": "#2E86AB", "switch": "#A23B72", "server": "#F18F01", "client": "#C73E1D"},
            "dark": {"gateway": "#F8F9FA", "switch": "#E9ECEF", "server": "#DEE2E6", "client": "#CED4DA"}
        }
        
        colors = base_colors.get(color_scheme, base_colors["default"])
        
        return f'''graph TD
    %% Network Topology Diagram
    
    subgraph "WAN"
        ISP1[ISP-1<br/>Primary]
        ISP2[ISP-2<br/>Failover]
    end
    
    subgraph "Leaf Tier"
        Router{{"Router-01<br/>High Availability"}}
        Switch[["Switch-01<br/>24-port VLAN"]]
    end
    
    subgraph "Cluster Tier"
        App1["App-01<br/>Load Balanced"]
        App2["App-02<br/>Load Balanced"] 
        DB1["DB-Primary<br/>PostgreSQL"]
        DB2["DB-Replica<br/>Read-Only"]
        LB["Load Balancer<br/>HAProxy"]
    end
    
    subgraph "Client Tier"
        WS1(("Workstation-01"))
        WS2(("Workstation-02"))
    end
    
    %% Connections
    ISP1 ==> Router
    ISP2 -.-> Router
    Router ==> Switch
    Switch --> LB
    LB --> App1
    LB --> App2
    App1 --> DB1
    App2 --> DB1
    DB1 --> DB2
    Switch --> WS1
    Switch --> WS2
    
    %% Styling
    classDef gateway fill:{colors["gateway"]},stroke:#333,stroke-width:2px
    classDef switch fill:{colors["switch"]},stroke:#333,stroke-width:2px
    classDef server fill:{colors["server"]},stroke:#333,stroke-width:2px
    classDef client fill:{colors["client"]},stroke:#333,stroke-width:2px
    
    class Router gateway
    class Switch switch
    class App1,App2,DB1,DB2,LB server
    class WS1,WS2 client
'''
    
    def _generate_dot_topology(self, layout: str, color_scheme: str, show_details: bool) -> str:
        """Generate Graphviz DOT representation"""
        return '''digraph NetworkTopology {
    rankdir=TB;
    bgcolor="white";
    node [fontname="Arial", fontsize=10];
    edge [fontname="Arial", fontsize=8];
    
    subgraph cluster_wan {
        label="WAN Tier";
        color=lightgray;
        style=filled;
        fillcolor=lightblue;
        
        ISP1 [label="ISP-1\\nPrimary", shape=cloud, fillcolor="#FFE5B4", style=filled];
        ISP2 [label="ISP-2\\nFailover", shape=cloud, fillcolor="#FFE5B4", style=filled];
    }
    
    subgraph cluster_leaf {
        label="Leaf Tier";
        color=lightgray;
        style=filled;
        fillcolor=lightgreen;
        
        Router [label="Router-01\\nHA Gateway", shape=diamond, fillcolor="#FF6B6B", style=filled];
        Switch [label="Switch-01\\n24-port VLAN", shape=hexagon, fillcolor="#4ECDC4", style=filled];
    }
    
    subgraph cluster_cluster {
        label="Cluster Tier";
        color=lightgray;
        style=filled;
        fillcolor=lightyellow;
        
        App1 [label="App-01\\nApplication", shape=box, fillcolor="#45B7D1", style=filled];
        App2 [label="App-02\\nApplication", shape=box, fillcolor="#45B7D1", style=filled];
        DB1 [label="DB-Primary\\nPostgreSQL", shape=box, fillcolor="#96CEB4", style=filled];
        DB2 [label="DB-Replica\\nRead-Only", shape=box, fillcolor="#96CEB4", style=filled];
        LB [label="Load Balancer\\nHAProxy", shape=box, fillcolor="#FECA57", style=filled];
    }
    
    subgraph cluster_client {
        label="Client Tier";
        color=lightgray;
        style=filled;
        fillcolor=lightcyan;
        
        WS1 [label="Workstation-01", shape=circle, fillcolor="#DDA0DD", style=filled];
        WS2 [label="Workstation-02", shape=circle, fillcolor="#DDA0DD", style=filled];
    }
    
    // Connections
    ISP1 -> Router [label="Primary\\n100Mbps", color=green, penwidth=3];
    ISP2 -> Router [label="Failover\\n100Mbps", color=red, style=dashed, penwidth=2];
    Router -> Switch [label="10Gbps", penwidth=3];
    Switch -> LB [label="1Gbps"];
    LB -> App1 [label="Load Balanced"];
    LB -> App2 [label="Load Balanced"];
    App1 -> DB1 [label="Database"];
    App2 -> DB1 [label="Database"];
    DB1 -> DB2 [label="Replication", style=dotted];
    Switch -> WS1 [label="1Gbps"];
    Switch -> WS2 [label="1Gbps"];
}'''
    
    def _generate_svg_topology(self, layout: str, color_scheme: str, show_details: bool) -> str:
        """Generate SVG representation (placeholder)"""
        return '''<svg viewBox="0 0 800 600" xmlns="http://www.w3.org/2000/svg">
    <style>
        .tier-label { font-family: Arial; font-size: 14px; font-weight: bold; }
        .node-label { font-family: Arial; font-size: 10px; text-anchor: middle; }
        .gateway { fill: #FF6B6B; stroke: #333; stroke-width: 2; }
        .switch { fill: #4ECDC4; stroke: #333; stroke-width: 2; }
        .server { fill: #45B7D1; stroke: #333; stroke-width: 2; }
        .client { fill: #96CEB4; stroke: #333; stroke-width: 2; }
    </style>
    
    <!-- WAN Tier -->
    <text x="400" y="30" class="tier-label" text-anchor="middle">WAN Tier</text>
    <ellipse cx="350" cy="80" rx="60" ry="30" fill="#FFE5B4" stroke="#333"/>
    <text x="350" y="85" class="node-label">ISP-1 Primary</text>
    <ellipse cx="450" cy="80" rx="60" ry="30" fill="#FFE5B4" stroke="#333"/>
    <text x="450" y="85" class="node-label">ISP-2 Failover</text>
    
    <!-- Leaf Tier -->
    <text x="400" y="180" class="tier-label" text-anchor="middle">Leaf Tier</text>
    <polygon points="400,200 440,220 400,240 360,220" class="gateway"/>
    <text x="400" y="225" class="node-label">Router-01</text>
    <polygon points="380,300 420,300 430,320 410,340 390,340 370,320" class="switch"/>
    <text x="400" y="325" class="node-label">Switch-01</text>
    
    <!-- Cluster Tier -->
    <text x="400" y="420" class="tier-label" text-anchor="middle">Cluster Tier</text>
    <rect x="280" y="450" width="60" height="40" class="server"/>
    <text x="310" y="475" class="node-label">App-01</text>
    <rect x="360" y="450" width="60" height="40" class="server"/>
    <text x="390" y="475" class="node-label">LB</text>
    <rect x="440" y="450" width="60" height="40" class="server"/>
    <text x="470" y="475" class="node-label">App-02</text>
    
    <!-- Connections -->
    <line x1="350" y1="110" x2="380" y2="200" stroke="#333" stroke-width="3"/>
    <line x1="450" y1="110" x2="420" y2="200" stroke="red" stroke-width="2" stroke-dasharray="5,5"/>
    <line x1="400" y1="240" x2="400" y2="300" stroke="#333" stroke-width="3"/>
    <line x1="380" y1="340" x2="310" y2="450" stroke="#333" stroke-width="2"/>
    <line x1="400" y1="340" x2="390" y2="450" stroke="#333" stroke-width="2"/>
    <line x1="420" y1="340" x2="470" y2="450" stroke="#333" stroke-width="2"/>
    
    <text x="400" y="580" text-anchor="middle" font-family="Arial" font-size="12">Network Topology Visualization</text>
</svg>'''
    
    def handle_initialize(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """Handle initialize request"""
        return {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "logging": {}
            },
            "serverInfo": {
                "name": "cim-network-sdn",
                "version": "1.0.0"
            }
        }
    
    def handle_list_tools(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """Handle tools/list request"""
        return {"tools": self.tools}
    
    async def handle_call_tool(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """Handle tools/call request"""
        name = params.get("name")
        arguments = params.get("arguments", {})
        
        # Map MCP tool names to SDN commands
        command_map = {
            "initialize_sdn": "initialize_sdn",
            "create_base_topology": "create_base_topology",
            "add_sdn_node": "add_sdn_node",
            "connect_sdn_nodes": "connect_sdn_nodes",
            "generate_nix_topology": "generate_nix_topology",
            "generate_advanced_nix": "generate_advanced_nix",
            "get_sdn_state": "get_sdn_state",
            "export_context_graph": "export_context_graph",
            "visualize_topology": "visualize_topology",
        }
        
        if name not in command_map:
            return {
                "content": [{"type": "text", "text": f"❌ Unknown tool: {name}"}],
                "isError": True
            }
        
        # Execute the SDN command
        response = await self.execute_sdn_command(command_map[name], arguments)
        
        # Format response
        if response["success"]:
            result_text = f"✅ {response['message']}\n"
            
            if response.get("data"):
                if "configuration" in response["data"]:
                    result_text += f"\n```nix\n{response['data']['configuration']}\n```\n"
                elif "context_graph" in response["data"]:
                    result_text += f"\n```json\n{response['data']['context_graph']}\n```\n"
                else:
                    result_text += f"\nData: {json.dumps(response['data'], indent=2)}\n"
        else:
            result_text = f"❌ {response['message']}\n"
            if response.get("error"):
                result_text += f"\nError: {response['error']}\n"
        
        return {"content": [{"type": "text", "text": result_text}]}
    
    async def handle_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle incoming JSON-RPC request"""
        method = request.get("method")
        params = request.get("params", {})
        request_id = request.get("id")
        
        try:
            if method == "initialize":
                result = self.handle_initialize(params)
            elif method == "tools/list":
                result = self.handle_list_tools(params)
            elif method == "tools/call":
                result = await self.handle_call_tool(params)
            else:
                raise Exception(f"Unknown method: {method}")
            
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": result
            }
            
        except Exception as e:
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {
                    "code": -32000,
                    "message": str(e)
                }
            }
    
    async def run(self):
        """Run the SDN MCP server"""
        while True:
            try:
                # Read line from stdin
                line = await asyncio.get_event_loop().run_in_executor(None, sys.stdin.readline)
                if not line:
                    break
                
                # Parse JSON-RPC request
                request = json.loads(line.strip())
                
                # Handle request
                response = await self.handle_request(request)
                
                # Send response
                print(json.dumps(response), flush=True)
                
            except json.JSONDecodeError:
                continue
            except Exception as e:
                error_response = {
                    "jsonrpc": "2.0",
                    "id": None,
                    "error": {
                        "code": -32700,
                        "message": f"Parse error: {str(e)}"
                    }
                }
                print(json.dumps(error_response), flush=True)


async def main():
    """Main entry point"""
    server = SDNMCPServer()
    await server.run()


if __name__ == "__main__":
    asyncio.run(main())