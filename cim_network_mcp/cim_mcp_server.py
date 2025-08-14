#!/usr/bin/env python3
"""
CIM Network MCP Server

Proper CIM (Composable Information Machine) architecture implementation with:
DEV/CLIENT -> LEAF -> cluster -> super-cluster hierarchy using NATS lattice
"""

import json
import sys
import asyncio
import tempfile
import uuid
from pathlib import Path
from typing import Dict, Any, List, Optional

try:
    from .cim_topology import (
        CimTopologyBuilder, CimTopology, CimTier, ClientType, CimEvent,
        create_development_cim, create_production_cim, ClientId, LeafId, 
        ClusterId, SuperClusterId, NatsLatticeConfig
    )
    CIM_TOPOLOGY_AVAILABLE = True
except ImportError:
    CIM_TOPOLOGY_AVAILABLE = False


class CimMCPServer:
    """CIM-focused MCP server implementing proper hierarchical architecture"""
    
    def __init__(self):
        self.project_root = Path(__file__).parent.parent
        self.tools = self._define_tools()
        self.cim_topology = None
        self.event_chain = []  # Event sourcing chain

    def _define_tools(self) -> List[Dict[str, Any]]:
        """Define CIM-specific tools"""
        return [
            {
                "name": "create_cim_topology",
                "description": "Create CIM hierarchical topology (DEV/CLIENT -> LEAF -> cluster -> super-cluster)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Name of the CIM topology"},
                        "topology_type": {"type": "string", "enum": ["development", "production"], "default": "development"},
                        "enable_nats": {"type": "boolean", "default": True, "description": "Enable NATS lattice messaging"}
                    },
                    "required": ["name"]
                }
            },
            {
                "name": "add_cim_client",
                "description": "Add a client to the CIM topology (DEV/CLIENT tier)",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string", "description": "Client name"},
                        "client_type": {"type": "string", "enum": ["developer", "application", "service", "browser", "cli"]},
                        "preferred_leaf": {"type": "string", "description": "Optional preferred leaf node ID"}
                    },
                    "required": ["name", "client_type"]
                }
            },
            {
                "name": "scale_cim_tier",
                "description": "Scale a specific tier in the CIM hierarchy",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "tier": {"type": "string", "enum": ["leaf", "cluster"], "description": "Tier to scale"},
                        "action": {"type": "string", "enum": ["add", "remove"], "description": "Scale action"},
                        "count": {"type": "integer", "minimum": 1, "default": 1, "description": "Number of nodes to add/remove"}
                    },
                    "required": ["tier", "action"]
                }
            },
            {
                "name": "generate_cim_config",
                "description": "Generate nix-topology configuration for CIM deployment",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "tier": {"type": "string", "enum": ["client", "leaf", "cluster", "super_cluster", "all"], "default": "all"},
                        "node_id": {"type": "string", "description": "Generate config for specific node (optional)"},
                        "include_nats": {"type": "boolean", "default": True, "description": "Include NATS lattice configuration"}
                    }
                }
            },
            {
                "name": "get_cim_state",
                "description": "Get current state of the CIM topology",
                "inputSchema": {"type": "object", "properties": {}}
            },
            {
                "name": "simulate_cim_event",
                "description": "Simulate an event flowing through CIM tiers",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "event_type": {"type": "string", "description": "Type of event to simulate"},
                        "payload": {"type": "object", "description": "Event payload"},
                        "source_tier": {"type": "string", "enum": ["client", "leaf", "cluster", "super_cluster"], "default": "client"}
                    },
                    "required": ["event_type", "payload"]
                }
            },
            {
                "name": "visualize_cim_topology",
                "description": "Generate visualization of CIM hierarchy",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "format": {"type": "string", "enum": ["ascii", "mermaid", "dot"], "default": "ascii"},
                        "include_nats": {"type": "boolean", "default": True, "description": "Include NATS lattice connections"}
                    }
                }
            }
        ]

    async def execute_cim_command(self, command: str, args: Dict[str, Any]) -> Dict[str, Any]:
        """Execute CIM command"""
        try:
            if command == "create_cim_topology":
                return await self._create_cim_topology(args)
            elif command == "add_cim_client":
                return await self._add_cim_client(args)
            elif command == "scale_cim_tier":
                return await self._scale_cim_tier(args)
            elif command == "generate_cim_config":
                return await self._generate_cim_config(args)
            elif command == "get_cim_state":
                return await self._get_cim_state(args)
            elif command == "simulate_cim_event":
                return await self._simulate_cim_event(args)
            elif command == "visualize_cim_topology":
                return await self._visualize_cim_topology(args)
            else:
                return {
                    "success": False,
                    "message": f"❌ Unknown CIM command: {command}",
                    "data": {}
                }
        except Exception as e:
            return {
                "success": False,
                "message": f"❌ Error executing CIM command '{command}': {e}",
                "data": {}
            }

    async def _create_cim_topology(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """Create CIM hierarchical topology"""
        if not CIM_TOPOLOGY_AVAILABLE:
            return {
                "success": False,
                "message": "❌ CIM topology support not available. Please check installation.",
                "data": {}
            }

        name = args.get("name")
        topology_type = args.get("topology_type", "development")
        enable_nats = args.get("enable_nats", True)

        # Create CIM topology
        if topology_type == "development":
            self.cim_topology = create_development_cim(name)
        elif topology_type == "production":
            self.cim_topology = create_production_cim(name)
        else:
            return {
                "success": False,
                "message": f"❌ Unknown topology type: {topology_type}",
                "data": {}
            }

        # Generate event for topology creation
        if CIM_TOPOLOGY_AVAILABLE:
            event = CimEvent.create(
                payload={
                    "event_type": "TopologyCreated",
                    "topology_name": name,
                    "topology_type": topology_type,
                    "enable_nats": enable_nats
                },
                source_tier=CimTier.SUPER_CLUSTER,
                node_id="system"
            )
            self.event_chain.append(event)

        summary = self.cim_topology.get_tier_summary()

        return {
            "success": True,
            "message": f"🏗️ Created CIM topology '{name}' ({topology_type})",
            "data": {
                "topology_id": summary["topology_id"],
                "topology_type": topology_type,
                "version": summary["version"],
                "tiers": summary["tiers"],
                "hierarchy": summary["hierarchy"],
                "nats_enabled": enable_nats,
                "event_cid": event.event_cid.value if CIM_TOPOLOGY_AVAILABLE else None
            }
        }

    async def _add_cim_client(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """Add client to CIM topology"""
        if not self.cim_topology:
            return {
                "success": False,
                "message": "❌ No CIM topology exists. Create one first using create_cim_topology.",
                "data": {}
            }

        name = args.get("name")
        client_type_str = args.get("client_type")
        preferred_leaf_str = args.get("preferred_leaf")

        # Map string to enum
        client_type_map = {
            "developer": ClientType.DEVELOPER,
            "application": ClientType.APPLICATION,
            "service": ClientType.SERVICE,
            "browser": ClientType.BROWSER,
            "cli": ClientType.CLI
        }
        client_type = client_type_map.get(client_type_str, ClientType.CLI)

        # Find preferred leaf
        preferred_leaf = None
        if preferred_leaf_str:
            for leaf_id in self.cim_topology.leaves:
                if leaf_id.value == preferred_leaf_str:
                    preferred_leaf = leaf_id
                    break

        # Add client using topology builder
        builder = CimTopologyBuilder()
        builder.topology = self.cim_topology
        builder.with_client(name, client_type, preferred_leaf)
        self.cim_topology = builder.build()

        # Find the newly added client
        new_client = None
        for client_id, client_config in self.cim_topology.clients.items():
            if client_config.name == name and client_config.client_type == client_type:
                new_client = (client_id, client_config)
                break

        if new_client:
            client_id, client_config = new_client
            
            # Generate event for client registration
            if CIM_TOPOLOGY_AVAILABLE:
                event = CimEvent.create(
                    payload={
                        "event_type": "ClientRegistered",
                        "client_id": client_id.value,
                        "client_name": name,
                        "client_type": client_type_str,
                        "assigned_leaf": client_config.assigned_leaf.value
                    },
                    source_tier=CimTier.LEAF,
                    node_id=client_config.assigned_leaf.value,
                    previous_cid=self.event_chain[-1].event_cid if self.event_chain else None
                )
                self.event_chain.append(event)

            summary = self.cim_topology.get_tier_summary()

            return {
                "success": True,
                "message": f"👤 Added CIM client '{name}' ({client_type_str}) to {client_config.assigned_leaf.value}",
                "data": {
                    "client_id": client_id.value,
                    "name": name,
                    "client_type": client_type_str,
                    "assigned_leaf": client_config.assigned_leaf.value,
                    "total_clients": summary["tiers"]["clients"],
                    "event_cid": event.event_cid.value if CIM_TOPOLOGY_AVAILABLE else None
                }
            }
        else:
            return {
                "success": False,
                "message": "❌ Failed to add client to topology",
                "data": {}
            }

    async def _scale_cim_tier(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """Scale CIM tier (add/remove nodes)"""
        if not self.cim_topology:
            return {
                "success": False,
                "message": "❌ No CIM topology exists. Create one first.",
                "data": {}
            }

        tier = args.get("tier")
        action = args.get("action")
        count = args.get("count", 1)

        # For now, simulate scaling (would require more complex implementation)
        summary = self.cim_topology.get_tier_summary()
        
        return {
            "success": True,
            "message": f"🔧 Simulated {action} {count} {tier} node(s) in CIM topology",
            "data": {
                "tier": tier,
                "action": action,
                "count": count,
                "current_tiers": summary["tiers"],
                "note": "Scaling simulation - would modify topology in production"
            }
        }

    async def _generate_cim_config(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """Generate CIM configuration"""
        if not self.cim_topology:
            return {
                "success": False,
                "message": "❌ No CIM topology exists. Create one first.",
                "data": {}
            }

        tier = args.get("tier", "all")
        node_id = args.get("node_id")
        include_nats = args.get("include_nats", True)

        # Generate basic NixOS configuration for CIM
        config = self._generate_nixos_cim_config(tier, node_id, include_nats)

        return {
            "success": True,
            "message": f"⚙️ Generated CIM configuration for {tier}",
            "data": {
                "tier": tier,
                "node_id": node_id,
                "include_nats": include_nats,
                "config_preview": config[:500] + "..." if len(config) > 500 else config
            }
        }

    def _generate_nixos_cim_config(self, tier: str, node_id: Optional[str], include_nats: bool) -> str:
        """Generate NixOS configuration for CIM deployment"""
        summary = self.cim_topology.get_tier_summary()
        
        config_parts = [
            "# CIM Network NixOS Configuration",
            f"# Topology: {summary['topology_id']}",
            f"# Version: {summary['version']}",
            "",
            "{ config, pkgs, ... }:",
            "{",
            "  # CIM Network Configuration",
            "  services.cim-network = {",
            "    enable = true;",
            f"    topology.id = \"{summary['topology_id']}\";",
            f"    topology.version = {summary['version']};",
            ""
        ]

        if tier == "all" or tier == "super_cluster":
            config_parts.extend([
                "    # Super-cluster configuration",
                "    super-cluster = {",
                "      enable = true;",
                "      role = \"global-orchestrator\";",
                "      nats.gateway = true;",
                "    };",
                ""
            ])

        if tier == "all" or tier == "cluster":
            config_parts.extend([
                "    # Cluster configuration",
                "    cluster = {",
                "      enable = true;",
                "      role = \"domain-coordinator\";",
                "      nats.cluster = true;",
                "    };",
                ""
            ])

        if tier == "all" or tier == "leaf":
            config_parts.extend([
                "    # Leaf configuration",
                "    leaf = {",
                "      enable = true;",
                "      role = \"local-orchestrator\";",
                "      nats.leaf = true;",
                "      event-store.enable = true;",
                "    };",
                ""
            ])

        if tier == "all" or tier == "client":
            config_parts.extend([
                "    # Client configuration",
                "    client = {",
                "      enable = true;",
                "      role = \"request-originator\";",
                "      nats.client = true;",
                "    };",
                ""
            ])

        if include_nats:
            config_parts.extend([
                "  # NATS Lattice Configuration",
                "  services.nats = {",
                "    enable = true;",
                "    jetstream = true;",
                "    settings = {",
                "      max_payload = 1048576;",
                "      max_pending = 65536;",
                "    };",
                "  };",
                ""
            ])

        config_parts.extend([
            "  };",
            "",
            "  # Network configuration for CIM",
            "  networking.firewall.allowedTCPPorts = [ 4222 6222 7222 7422 ];",
            "",
            "  # System packages for CIM",
            "  environment.systemPackages = with pkgs; [",
            "    nats-server",
            "    nats-top",
            "  ];",
            "}"
        ])

        return "\n".join(config_parts)

    async def _get_cim_state(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """Get current CIM state"""
        if not self.cim_topology:
            return {
                "success": True,
                "message": "ℹ️ No CIM topology exists",
                "data": {
                    "topology_exists": False,
                    "event_chain_length": len(self.event_chain)
                }
            }

        summary = self.cim_topology.get_tier_summary()

        return {
            "success": True,
            "message": "📊 CIM topology state retrieved",
            "data": {
                "topology_exists": True,
                "summary": summary,
                "event_chain_length": len(self.event_chain),
                "last_event": self.event_chain[-1].payload if self.event_chain else None
            }
        }

    async def _simulate_cim_event(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """Simulate event flowing through CIM tiers"""
        if not self.cim_topology:
            return {
                "success": False,
                "message": "❌ No CIM topology exists. Create one first.",
                "data": {}
            }

        event_type = args.get("event_type")
        payload = args.get("payload")
        source_tier_str = args.get("source_tier", "client")

        tier_map = {
            "client": CimTier.CLIENT,
            "leaf": CimTier.LEAF,
            "cluster": CimTier.CLUSTER,
            "super_cluster": CimTier.SUPER_CLUSTER
        }
        source_tier = tier_map.get(source_tier_str, CimTier.CLIENT)

        # Create and simulate event flow
        if CIM_TOPOLOGY_AVAILABLE:
            event = CimEvent.create(
                payload={
                    "event_type": event_type,
                    "simulation": True,
                    **payload
                },
                source_tier=source_tier,
                node_id="simulation",
                previous_cid=self.event_chain[-1].event_cid if self.event_chain else None
            )
            self.event_chain.append(event)

            # Simulate event flow through tiers
            flow_path = self._simulate_event_flow(source_tier, event_type)

            return {
                "success": True,
                "message": f"🌊 Simulated CIM event '{event_type}' from {source_tier_str}",
                "data": {
                    "event_cid": event.event_cid.value,
                    "source_tier": source_tier_str,
                    "event_type": event_type,
                    "flow_path": flow_path,
                    "correlation_id": event.correlation_id.value
                }
            }
        else:
            return {
                "success": True,
                "message": f"🌊 Simulated CIM event '{event_type}' (mock mode)",
                "data": {
                    "event_type": event_type,
                    "source_tier": source_tier_str,
                    "mock_mode": True
                }
            }

    def _simulate_event_flow(self, source_tier: CimTier, event_type: str) -> List[str]:
        """Simulate how event flows through CIM tiers"""
        flow_path = []
        
        if source_tier == CimTier.CLIENT:
            flow_path = [
                "CLIENT: Event initiated by user/application",
                "LEAF: Command validation and local processing", 
                "CLUSTER: Domain coordination and saga orchestration",
                "SUPER_CLUSTER: Global orchestration and consistency"
            ]
        elif source_tier == CimTier.LEAF:
            flow_path = [
                "LEAF: Local event generated",
                "CLUSTER: Domain aggregation and coordination",
                "SUPER_CLUSTER: Global impact analysis"
            ]
        elif source_tier == CimTier.CLUSTER:
            flow_path = [
                "CLUSTER: Domain-level event",
                "SUPER_CLUSTER: Cross-domain coordination"
            ]
        else:  # SUPER_CLUSTER
            flow_path = [
                "SUPER_CLUSTER: Global system event",
                "CLUSTER: Domain-specific coordination",
                "LEAF: Local implementation",
                "CLIENT: Event notification"
            ]
            
        return flow_path

    async def _visualize_cim_topology(self, args: Dict[str, Any]) -> Dict[str, Any]:
        """Generate CIM topology visualization"""
        if not self.cim_topology:
            return {
                "success": False,
                "message": "❌ No CIM topology exists. Create one first.",
                "data": {}
            }

        format_type = args.get("format", "ascii")
        include_nats = args.get("include_nats", True)

        if format_type == "ascii":
            visualization = self._generate_ascii_cim_visualization(include_nats)
        elif format_type == "mermaid":
            visualization = self._generate_mermaid_cim_visualization(include_nats)
        elif format_type == "dot":
            visualization = self._generate_dot_cim_visualization(include_nats)
        else:
            return {
                "success": False,
                "message": f"❌ Unknown visualization format: {format_type}",
                "data": {}
            }

        return {
            "success": True,
            "message": f"📊 Generated CIM topology visualization ({format_type})",
            "data": {
                "format": format_type,
                "include_nats": include_nats,
                "visualization": visualization
            }
        }

    def _generate_ascii_cim_visualization(self, include_nats: bool) -> str:
        """Generate ASCII visualization of CIM hierarchy"""
        summary = self.cim_topology.get_tier_summary()
        
        lines = [
            "CIM Hierarchical Topology",
            "=" * 50,
            f"Topology ID: {summary['topology_id']}",
            f"Version: {summary['version']}",
            "",
            "Hierarchy:",
            ""
        ]

        # Super-cluster tier
        lines.extend([
            "┌─ SUPER-CLUSTER (Global Orchestration)",
            "│   ├─ Global event coordination",
            "│   ├─ Cross-domain workflows", 
            "│   └─ System-wide consistency",
            "│"
        ])

        # Cluster tier
        cluster_count = summary["tiers"]["clusters"]
        lines.extend([
            f"├─ CLUSTERS ({cluster_count}) (Domain Coordination)",
            "│   ├─ Domain boundary enforcement",
            "│   ├─ Distributed saga orchestration",
            "│   └─ Event stream aggregation",
            "│"
        ])

        # Leaf tier  
        leaf_count = summary["tiers"]["leaves"]
        lines.extend([
            f"├─ LEAVES ({leaf_count}) (Local Orchestration)",
            "│   ├─ Client session management",
            "│   ├─ Local event sourcing",
            "│   └─ Request validation",
            "│"
        ])

        # Client tier
        client_count = summary["tiers"]["clients"]
        lines.extend([
            f"└─ CLIENTS ({client_count}) (Request Origination)",
            "    ├─ User interfaces (CLI, Browser)",
            "    ├─ Applications and services",
            "    └─ Developer tools",
            ""
        ])

        if include_nats:
            lines.extend([
                "NATS Lattice Messaging:",
                "┌─ Gateway (Super-cluster connections)",
                "├─ Cluster (JetStream persistence)", 
                "├─ Leaf (Client connections)",
                "└─ Request/Reply + Event Streaming",
                ""
            ])

        lines.extend([
            "Event Flow Direction:",
            "CLIENT → LEAF → CLUSTER → SUPER-CLUSTER",
            ""
        ])

        return "\n".join(lines)

    def _generate_mermaid_cim_visualization(self, include_nats: bool) -> str:
        """Generate Mermaid diagram of CIM hierarchy"""
        summary = self.cim_topology.get_tier_summary()
        
        mermaid_lines = [
            "graph TB",
            "    subgraph \"CIM Hierarchical Architecture\"",
            f"        SC[Super-Cluster<br/>Global Orchestrator<br/>Count: {summary['tiers']['super_clusters']}]",
            f"        C[Clusters<br/>Domain Coordinators<br/>Count: {summary['tiers']['clusters']}]",
            f"        L[Leaves<br/>Local Orchestrators<br/>Count: {summary['tiers']['leaves']}]",
            f"        CL[Clients<br/>Request Originators<br/>Count: {summary['tiers']['clients']}]",
            "    end",
            "",
            "    CL -->|Request/Reply| L",
            "    L -->|Event Streaming| C", 
            "    C -->|Global Coordination| SC",
            "",
            "    classDef superCluster fill:#ffccbc",
            "    classDef cluster fill:#fff9c4",
            "    classDef leaf fill:#c8e6c9", 
            "    classDef client fill:#e1f5fe",
            "",
            "    class SC superCluster",
            "    class C cluster",
            "    class L leaf",
            "    class CL client"
        ]

        if include_nats:
            mermaid_lines.extend([
                "",
                "    subgraph \"NATS Lattice\"",
                "        NG[NATS Gateway]",
                "        NC[NATS Cluster]", 
                "        NL[NATS Leaf]",
                "    end",
                "",
                "    SC -.->|Gateway| NG",
                "    C -.->|JetStream| NC",
                "    L -.->|Leaf Conn| NL",
                "    CL -.->|Client Conn| NL"
            ])

        return "\n".join(mermaid_lines)

    def _generate_dot_cim_visualization(self, include_nats: bool) -> str:
        """Generate DOT graph of CIM hierarchy"""
        summary = self.cim_topology.get_tier_summary()
        
        dot_lines = [
            "digraph CIM_Topology {",
            "    rankdir=TB;",
            "    node [shape=box, style=filled];",
            "",
            f"    SuperCluster [label=\"Super-Cluster\\nGlobal Orchestrator\\nCount: {summary['tiers']['super_clusters']}\", fillcolor=lightcoral];",
            f"    Cluster [label=\"Clusters\\nDomain Coordinators\\nCount: {summary['tiers']['clusters']}\", fillcolor=lightyellow];", 
            f"    Leaf [label=\"Leaves\\nLocal Orchestrators\\nCount: {summary['tiers']['leaves']}\", fillcolor=lightgreen];",
            f"    Client [label=\"Clients\\nRequest Originators\\nCount: {summary['tiers']['clients']}\", fillcolor=lightblue];",
            "",
            "    Client -> Leaf [label=\"Request/Reply\"];",
            "    Leaf -> Cluster [label=\"Event Streaming\"];",
            "    Cluster -> SuperCluster [label=\"Global Coordination\"];",
            ""
        ]

        if include_nats:
            dot_lines.extend([
                "    subgraph cluster_nats {",
                "        label=\"NATS Lattice\";",
                "        color=gray;",
                "        Gateway [fillcolor=lightgray];",
                "        NATSCluster [fillcolor=lightgray];",
                "        NATSLeaf [fillcolor=lightgray];",
                "    }",
                "",
                "    SuperCluster -> Gateway [style=dashed];",
                "    Cluster -> NATSCluster [style=dashed];",
                "    Leaf -> NATSLeaf [style=dashed];",
                ""
            ])

        dot_lines.append("}")
        return "\n".join(dot_lines)

    async def handle_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Handle MCP request"""
        method = request.get("method")
        params = request.get("params", {})

        if method == "tools/list":
            return {
                "tools": self.tools
            }
        elif method == "tools/call":
            tool_name = params.get("name")
            arguments = params.get("arguments", {})
            
            result = await self.execute_cim_command(tool_name, arguments)
            
            return {
                "content": [
                    {
                        "type": "text", 
                        "text": json.dumps(result, indent=2)
                    }
                ]
            }
        else:
            return {
                "error": {
                    "code": -32601,
                    "message": f"Method not found: {method}"
                }
            }


async def main():
    """Main MCP server loop"""
    server = CimMCPServer()
    
    while True:
        try:
            line = await asyncio.get_event_loop().run_in_executor(None, sys.stdin.readline)
            if not line:
                break
                
            request = json.loads(line.strip())
            response = await server.handle_request(request)
            
            # Add jsonrpc and id fields
            response["jsonrpc"] = "2.0"
            response["id"] = request.get("id")
            
            print(json.dumps(response))
            sys.stdout.flush()
            
        except KeyboardInterrupt:
            break
        except Exception as e:
            error_response = {
                "jsonrpc": "2.0",
                "id": request.get("id") if 'request' in locals() else None,
                "error": {
                    "code": -32603,
                    "message": f"Internal error: {e}"
                }
            }
            print(json.dumps(error_response))
            sys.stdout.flush()


if __name__ == "__main__":
    asyncio.run(main())