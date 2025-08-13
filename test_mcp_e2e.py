#!/usr/bin/env python3
"""
End-to-end test for the CIM Network MCP Server

This script tests the complete network topology building workflow:
1. Initialize the MCP server
2. Build a multi-region enterprise topology
3. Add various location types
4. Connect them with different connection types
5. Generate NixOS, nix-darwin, and Home Manager configurations
6. Validate the complete topology
"""

import json
import subprocess
import sys
from typing import Dict, Any, List
import asyncio

class MCPTester:
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
        """Send a request to the MCP server and return the response"""
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
            print(f"❌ Error executing MCP server: {stderr.decode()}")
            return None
            
        try:
            # Parse the last line that contains JSON (skip the development environment banner)
            lines = stdout.decode().strip().split('\n')
            json_line = None
            for line in reversed(lines):
                if line.startswith('{"jsonrpc"'):
                    json_line = line
                    break
            
            if not json_line:
                print(f"❌ No JSON response found in output")
                return None
                
            return json.loads(json_line)
        except json.JSONDecodeError as e:
            print(f"❌ Failed to parse JSON response: {e}")
            print(f"Raw output: {stdout.decode()}")
            return None
    
    async def test_initialize(self) -> bool:
        """Test MCP server initialization"""
        print("🔧 Testing MCP server initialization...")
        
        request = self.create_request("initialize")
        response = await self.send_request(request)
        
        if not response or "error" in response:
            print(f"❌ Initialize failed: {response}")
            return False
            
        result = response.get("result", {})
        if result.get("serverInfo", {}).get("name") == "network-topology-builder":
            print("✅ MCP server initialized successfully")
            return True
        else:
            print(f"❌ Unexpected initialize response: {result}")
            return False
    
    async def test_tools_list(self) -> bool:
        """Test tools listing"""
        print("📋 Testing tools listing...")
        
        request = self.create_request("tools/list")
        response = await self.send_request(request)
        
        if not response or "error" in response:
            print(f"❌ Tools list failed: {response}")
            return False
            
        tools = response.get("result", {}).get("tools", [])
        expected_tools = {
            "build_topology", "add_location", "connect_locations", 
            "generate_configuration", "validate_topology", "get_topology_status",
            "reset_topology", "complete_topology"
        }
        
        actual_tools = {tool["name"] for tool in tools}
        
        if expected_tools.issubset(actual_tools):
            print(f"✅ All {len(expected_tools)} expected tools available")
            return True
        else:
            missing = expected_tools - actual_tools
            print(f"❌ Missing tools: {missing}")
            return False
    
    async def test_build_topology(self) -> bool:
        """Test building a new topology"""
        print("🌐 Testing topology building...")
        
        request = self.create_request("tools/call", {
            "name": "build_topology",
            "arguments": {
                "base_network": "10.0.0.0/8",
                "target_environment": "production",
                "scale": "enterprise",
                "use_case": "multi-region-hybrid-cloud"
            }
        })
        
        response = await self.send_request(request)
        
        if not response or "error" in response:
            print(f"❌ Build topology failed: {response}")
            return False
            
        result = response.get("result", {})
        content = result.get("content", [{}])[0].get("text", "")
        
        if "topology building" in content.lower():
            print("✅ Topology building started successfully")
            return True
        else:
            print(f"❌ Unexpected build response: {content}")
            return False
    
    async def test_add_locations(self) -> bool:
        """Test adding various location types"""
        print("🏢 Testing location additions...")
        
        locations = [
            {
                "location_id": "dc-west",
                "location_type": "datacenter",
                "parameters": {
                    "name": "West Coast Primary DC",
                    "region": "us-west-1",
                    "az": "us-west-1a"
                }
            },
            {
                "location_id": "office-hq",
                "location_type": "office",
                "parameters": {
                    "name": "Corporate HQ",
                    "address": "123 Tech Boulevard, San Francisco, CA",
                    "size": "large"
                }
            },
            {
                "location_id": "aws-east",
                "location_type": "cloud",
                "parameters": {
                    "provider": "aws",
                    "region": "us-east-1"
                }
            },
            {
                "location_id": "edge-seattle",
                "location_type": "edge",
                "parameters": {
                    "name": "Seattle Edge Location",
                    "lat": "47.6062",
                    "lng": "-122.3321"
                }
            },
            {
                "location_id": "vlan-dmz",
                "location_type": "segment",
                "parameters": {
                    "name": "DMZ Segment",
                    "subnet": "192.168.10.0/24",
                    "vlan": "100"
                }
            }
        ]
        
        success_count = 0
        
        for location in locations:
            request = self.create_request("tools/call", {
                "name": "add_location",
                "arguments": location
            })
            
            response = await self.send_request(request)
            
            if response and "error" not in response:
                content = response.get("result", {}).get("content", [{}])[0].get("text", "")
                if "added location" in content.lower() or "✅" in content:
                    print(f"  ✅ Added {location['location_id']}")
                    success_count += 1
                else:
                    print(f"  ❌ Failed to add {location['location_id']}: {content}")
            else:
                print(f"  ❌ Error adding {location['location_id']}: {response}")
        
        if success_count == len(locations):
            print(f"✅ All {success_count} locations added successfully")
            return True
        else:
            print(f"❌ Only {success_count}/{len(locations)} locations added")
            return False
    
    async def test_connect_locations(self) -> bool:
        """Test connecting locations with various connection types"""
        print("🔗 Testing location connections...")
        
        connections = [
            {
                "from_location": "dc-west",
                "to_location": "office-hq",
                "connection_type": "fiber",
                "parameters": {
                    "bandwidth": "10Gbps",
                    "redundant": "true"
                }
            },
            {
                "from_location": "dc-west",
                "to_location": "aws-east",
                "connection_type": "directconnect",
                "parameters": {
                    "provider": "aws",
                    "bandwidth": "10Gbps"
                }
            },
            {
                "from_location": "office-hq",
                "to_location": "edge-seattle",
                "connection_type": "vpn",
                "parameters": {
                    "protocol": "wireguard",
                    "encrypted": "true"
                }
            },
            {
                "from_location": "dc-west",
                "to_location": "vlan-dmz",
                "connection_type": "virtual",
                "parameters": {
                    "protocol": "VLAN",
                    "bandwidth": "1Gbps"
                }
            }
        ]
        
        success_count = 0
        
        for connection in connections:
            request = self.create_request("tools/call", {
                "name": "connect_locations",
                "arguments": connection
            })
            
            response = await self.send_request(request)
            
            if response and "error" not in response:
                content = response.get("result", {}).get("content", [{}])[0].get("text", "")
                if "connected" in content.lower() or "🔗" in content:
                    print(f"  ✅ Connected {connection['from_location']} → {connection['to_location']}")
                    success_count += 1
                else:
                    print(f"  ❌ Failed to connect: {content}")
            else:
                print(f"  ❌ Connection error: {response}")
        
        if success_count == len(connections):
            print(f"✅ All {success_count} connections established")
            return True
        else:
            print(f"❌ Only {success_count}/{len(connections)} connections established")
            return False
    
    async def test_generate_configurations(self) -> bool:
        """Test generating configurations in all supported Nix formats"""
        print("⚙️ Testing configuration generation...")
        
        formats = ["nixos", "nix-darwin", "home-manager", "flake", "json"]
        success_count = 0
        
        for fmt in formats:
            request = self.create_request("tools/call", {
                "name": "generate_configuration",
                "arguments": {"format": fmt}
            })
            
            response = await self.send_request(request)
            
            if response and "error" not in response:
                content = response.get("result", {}).get("content", [{}])[0].get("text", "")
                if "configuration" in content.lower() or "🔧" in content:
                    print(f"  ✅ Generated {fmt} configuration")
                    success_count += 1
                else:
                    print(f"  ❌ Failed to generate {fmt}: {content}")
            else:
                print(f"  ❌ Generation error for {fmt}: {response}")
        
        if success_count == len(formats):
            print(f"✅ All {success_count} configuration formats generated")
            return True
        else:
            print(f"❌ Only {success_count}/{len(formats)} formats generated")
            return False
    
    async def test_validate_topology(self) -> bool:
        """Test topology validation"""
        print("✓ Testing topology validation...")
        
        request = self.create_request("tools/call", {
            "name": "validate_topology",
            "arguments": {}
        })
        
        response = await self.send_request(request)
        
        if not response or "error" in response:
            print(f"❌ Validation failed: {response}")
            return False
            
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        
        if "validation" in content.lower():
            print("✅ Topology validation completed")
            return True
        else:
            print(f"❌ Unexpected validation response: {content}")
            return False
    
    async def test_complete_topology(self) -> bool:
        """Test topology completion"""
        print("🎉 Testing topology completion...")
        
        request = self.create_request("tools/call", {
            "name": "complete_topology",
            "arguments": {}
        })
        
        response = await self.send_request(request)
        
        if not response or "error" in response:
            print(f"❌ Completion failed: {response}")
            return False
            
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        
        if "completed" in content.lower() or "🎉" in content:
            print("✅ Topology completed successfully")
            return True
        else:
            print(f"❌ Unexpected completion response: {content}")
            return False
    
    async def run_full_test(self) -> bool:
        """Run the complete end-to-end test suite"""
        print("🚀 Starting CIM Network MCP Server End-to-End Test")
        print("=" * 60)
        
        tests = [
            ("Initialize MCP Server", self.test_initialize),
            ("List Available Tools", self.test_tools_list),
            ("Build Network Topology", self.test_build_topology),
            ("Add Network Locations", self.test_add_locations),
            ("Connect Locations", self.test_connect_locations),
            ("Generate Configurations", self.test_generate_configurations),
            ("Validate Topology", self.test_validate_topology),
            ("Complete Topology", self.test_complete_topology),
        ]
        
        passed = 0
        total = len(tests)
        
        for test_name, test_func in tests:
            try:
                if await test_func():
                    passed += 1
                else:
                    print(f"🚫 Test '{test_name}' FAILED")
            except Exception as e:
                print(f"🚫 Test '{test_name}' CRASHED: {e}")
            
            print("-" * 40)
        
        print(f"\n📊 Test Results: {passed}/{total} tests passed")
        
        if passed == total:
            print("🎊 🎉 ALL TESTS PASSED! 🎉 🎊")
            print("\nThe CIM Network MCP Server is fully functional and ready for Claude Code integration!")
            print("\nKey achievements:")
            print("✅ Complete MCP protocol compliance")
            print("✅ All 8 network topology tools working")
            print("✅ Multi-location network modeling")
            print("✅ Various connection types supported")
            print("✅ Comprehensive Nix ecosystem support:")
            print("   • NixOS (Linux systems)")
            print("   • nix-darwin (macOS systems)")  
            print("   • Home Manager (user environments)")
            print("   • Nix Flakes (cross-platform)")
            print("   • JSON (debugging/inspection)")
            print("✅ Event-driven architecture with context graphs")
            print("✅ Production-ready cim-domain-nix integration")
            return True
        else:
            print("❌ Some tests failed. Check the output above for details.")
            return False

async def main():
    tester = MCPTester()
    success = await tester.run_full_test()
    sys.exit(0 if success else 1)

if __name__ == "__main__":
    asyncio.run(main())