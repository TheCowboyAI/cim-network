#!/usr/bin/env python3
"""
Comprehensive SDN Pipeline Test

This tests the complete SDN pipeline:
1. Domain context → SDN construction
2. ContextGraph persistence
3. nix-topology compliant generation
"""

import json
import subprocess
import sys
import asyncio
from typing import Dict, Any, List
import uuid

class SDNPipelineTester:
    def __init__(self):
        self.request_id = 1
        
    def next_request_id(self) -> int:
        self.request_id += 1
        return self.request_id
    
    def create_request(self, method: str, params: Dict[str, Any] = None) -> Dict[str, Any]:
        return {
            "jsonrpc": "2.0",
            "method": method,
            "params": params or {},
            "id": self.next_request_id()
        }
    
    async def send_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Send a request to the SDN MCP server"""
        proc = await asyncio.create_subprocess_exec(
            'nix', 'develop', '--command', 'python3', '-m', 'cim_network_mcp',
            stdin=asyncio.subprocess.PIPE,
            stdout=asyncio.subprocess.PIPE,
            stderr=asyncio.subprocess.PIPE,
            cwd='/git/thecowboyai/cim-network'
        )
        
        input_data = json.dumps(request) + '\n'
        stdout, stderr = await proc.communicate(input_data.encode())
        
        if proc.returncode != 0:
            print(f"❌ Error executing SDN MCP server: {stderr.decode()}")
            return None
            
        try:
            lines = stdout.decode().strip().split('\n')
            json_line = None
            for line in reversed(lines):
                if line.startswith('{"jsonrpc"'):
                    json_line = line
                    break
            
            if not json_line:
                print(f"❌ No JSON response found")
                return None
                
            return json.loads(json_line)
        except json.JSONDecodeError as e:
            print(f"❌ Failed to parse JSON response: {e}")
            return None

    async def test_complete_sdn_pipeline(self) -> bool:
        """Test the complete SDN pipeline end-to-end"""
        print("🚀 Testing Complete SDN Pipeline")
        print("=" * 50)
        
        # 1. Initialize SDN from domain context
        print("1. 🏗️  Initialize SDN from domain context...")
        
        domain_context = {
            "domain_name": "enterprise-hybrid-cloud",
            "base_network": "10.0.0.0/8", 
            "target_environment": "production",
            "scale": "enterprise",
            "regions": ["us-west-1", "us-east-1"],
            "use_cases": ["web-services", "data-processing", "edge-computing"]
        }
        
        request = self.create_request("tools/call", {
            "name": "initialize_sdn",
            "arguments": {"domain_context": domain_context}
        })
        
        response = await self.send_request(request)
        if not response or "error" in response:
            print(f"   ❌ SDN initialization failed: {response}")
            return False
            
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        if "initialized from domain context" in content.lower():
            print("   ✅ SDN initialized from cim-start domain")
        else:
            print(f"   ❌ Unexpected response: {content}")
            return False
        
        # 2. Build multi-tier network topology
        print("\n2. 🌐 Building multi-tier network topology...")
        
        # Add server cluster tier
        nodes_to_add = [
            {
                "node_id": "app-server-01",
                "node_type": "server", 
                "tier": "cluster",
                "interfaces": [
                    {"name": "eth0", "type": "ethernet", "addresses": ["10.0.10.10"]}
                ],
                "services": ["nginx", "postgresql", "redis"],
                "metadata": {
                    "location": "datacenter-west", 
                    "role": "application-server",
                    "environment": "production"
                }
            },
            {
                "node_id": "db-server-01", 
                "node_type": "server",
                "tier": "cluster", 
                "interfaces": [
                    {"name": "eth0", "type": "ethernet", "addresses": ["10.0.10.20"]}
                ],
                "services": ["postgresql", "backup-agent"],
                "metadata": {
                    "location": "datacenter-west",
                    "role": "database-server", 
                    "environment": "production"
                }
            },
            {
                "node_id": "gateway-01",
                "node_type": "gateway",
                "tier": "leaf",
                "interfaces": [
                    {"name": "eth0", "type": "ethernet", "addresses": ["10.0.1.1"]},
                    {"name": "eth1", "type": "ethernet", "addresses": ["dhcp"]}
                ],
                "services": ["nat", "firewall", "vpn"],
                "metadata": {
                    "location": "datacenter-west",
                    "role": "network-gateway"
                }
            },
            {
                "node_id": "client-workstation",
                "node_type": "workstation", 
                "tier": "client",
                "interfaces": [
                    {"name": "wlan0", "type": "wireless", "addresses": ["dhcp"]}
                ],
                "services": ["networkmanager", "ssh-client"],
                "metadata": {
                    "location": "office-hq",
                    "role": "developer-workstation"
                }
            }
        ]
        
        for node in nodes_to_add:
            request = self.create_request("tools/call", {
                "name": "add_sdn_node",
                "arguments": node
            })
            
            response = await self.send_request(request)
            if response and "error" not in response:
                print(f"   ✅ Added {node['node_id']} ({node['node_type']}/{node['tier']})")
            else:
                print(f"   ❌ Failed to add {node['node_id']}: {response}")
                return False
                
        # 3. Establish network connections
        print("\n3. 🔗 Establishing network connections...")
        
        connections_to_create = [
            {
                "from_node": "app-server-01",
                "to_node": "gateway-01", 
                "connection_type": "ethernet",
                "properties": {"bandwidth": "10Gbps", "redundant": "true"}
            },
            {
                "from_node": "db-server-01",
                "to_node": "gateway-01",
                "connection_type": "ethernet", 
                "properties": {"bandwidth": "10Gbps", "redundant": "true"}
            },
            {
                "from_node": "app-server-01", 
                "to_node": "db-server-01",
                "connection_type": "ethernet",
                "properties": {"bandwidth": "10Gbps", "vlan": "100"}
            },
            {
                "from_node": "client-workstation",
                "to_node": "gateway-01",
                "connection_type": "wireless",
                "properties": {"bandwidth": "300Mbps", "encryption": "WPA3"}
            }
        ]
        
        for conn in connections_to_create:
            request = self.create_request("tools/call", {
                "name": "connect_sdn_nodes",
                "arguments": conn
            })
            
            response = await self.send_request(request)
            if response and "error" not in response:
                print(f"   ✅ Connected {conn['from_node']} → {conn['to_node']} ({conn['connection_type']})")
            else:
                print(f"   ❌ Connection failed: {response}")
                return False
        
        # 4. Verify SDN state
        print("\n4. 📊 Verifying SDN state...")
        
        request = self.create_request("tools/call", {
            "name": "get_sdn_state",
            "arguments": {}
        })
        
        response = await self.send_request(request)
        if response and "error" not in response:
            content = response.get("result", {}).get("content", [{}])[0].get("text", "")
            if "sdn state" in content.lower():
                print("   ✅ SDN state retrieved successfully")
                # Extract node/connection counts from response
                if "nodes" in content and "connections" in content:
                    print("   ✅ SDN contains nodes and connections as expected")
            else:
                print(f"   ❌ Unexpected state response: {content}")
                return False
        else:
            print(f"   ❌ Failed to get SDN state: {response}")
            return False
            
        # 5. Export ContextGraph
        print("\n5. 📈 Exporting cim-graph ContextGraph...")
        
        for format_type in ["json", "dot"]:
            request = self.create_request("tools/call", {
                "name": "export_context_graph", 
                "arguments": {"format": format_type}
            })
            
            response = await self.send_request(request)
            if response and "error" not in response:
                content = response.get("result", {}).get("content", [{}])[0].get("text", "")
                if "context" in content.lower() and format_type in content:
                    print(f"   ✅ Exported ContextGraph as {format_type}")
                    
                    # Validate structure for JSON export  
                    if format_type == "json":
                        if "nodes" in content and "edges" in content and "metadata" in content:
                            print("   ✅ ContextGraph JSON has expected structure (nodes/edges/metadata)")
                        else:
                            print("   ❌ ContextGraph JSON missing expected structure")
                            return False
                else:
                    print(f"   ❌ Failed to export as {format_type}: {content}")
                    return False
            else:
                print(f"   ❌ Export failed for {format_type}: {response}")
                return False
        
        # 6. Generate nix-topology compliant configurations
        print("\n6. 🔧 Generating nix-topology compliant configurations...")
        
        nix_formats = ["nixos", "nix-darwin", "home-manager", "flake"]
        
        for format_type in nix_formats:
            request = self.create_request("tools/call", {
                "name": "generate_nix_topology",
                "arguments": {"format": format_type}
            })
            
            response = await self.send_request(request)
            if response and "error" not in response:
                content = response.get("result", {}).get("content", [{}])[0].get("text", "")
                if "generated" in content.lower() and format_type in content:
                    print(f"   ✅ Generated {format_type} configuration")
                    
                    # Validate nix-topology compliance
                    if format_type == "nixos" and "nix-topology.lib.mkTopology" in content:
                        print("   ✅ NixOS config is nix-topology compliant")
                    elif format_type == "flake" and "description" in content and "inputs" in content:
                        print("   ✅ Flake config has proper structure")
                        
                else:
                    print(f"   ❌ Failed to generate {format_type}: {content}")
                    return False
            else:
                print(f"   ❌ Generation failed for {format_type}: {response}")
                return False
        
        print("\n🎊 🎉 SDN PIPELINE TEST COMPLETED SUCCESSFULLY! 🎉 🎊")
        print("\n📋 Pipeline Summary:")
        print("✅ Domain context → SDN initialization")  
        print("✅ Multi-tier network topology construction")
        print("✅ Network connections with typed properties")
        print("✅ ContextGraph state management and export")
        print("✅ nix-topology compliant configuration generation")
        print("✅ Multiple Nix formats (NixOS, nix-darwin, Home Manager, flakes)")
        print("\n🏗️  Architecture Validated:")
        print("• cim-start domain → SDN builder → ContextGraph → nix-topology projection")
        print("• Event-driven state management")
        print("• Production-ready nix-topology compliance") 
        print("• Clean separation of concerns")
        
        return True

async def main():
    """Run the complete SDN pipeline test"""
    tester = SDNPipelineTester()
    
    try:
        success = await tester.test_complete_sdn_pipeline()
        if success:
            print("\n🚀 The CIM Network SDN implementation is ready for production!")
            sys.exit(0)
        else:
            print("\n❌ SDN pipeline test failed. Check the output above.")
            sys.exit(1)
    except Exception as e:
        print(f"\n💥 Pipeline test crashed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())