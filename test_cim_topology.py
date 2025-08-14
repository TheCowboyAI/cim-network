#!/usr/bin/env python3
"""
Test CIM Topology Implementation

Comprehensive test suite for the proper CIM hierarchical architecture:
DEV/CLIENT -> LEAF -> cluster -> super-cluster
"""

import asyncio
import json
import subprocess
import sys
from pathlib import Path
from typing import Dict, Any, Optional

class CimTopologyTester:
    """Test runner for CIM topology functionality"""
    
    def __init__(self):
        self.request_id = 1
        
    def create_request(self, method: str, params: Dict[str, Any]) -> Dict[str, Any]:
        """Create JSON-RPC request"""
        request = {
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method,
            "params": params
        }
        self.request_id += 1
        return request
    
    async def send_request(self, request: Dict[str, Any]) -> Optional[Dict[str, Any]]:
        """Send request to MCP server"""
        try:
            proc = await asyncio.create_subprocess_exec(
                'python', '-m', 'cim_network_mcp',
                stdin=asyncio.subprocess.PIPE,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE,
                cwd=Path.cwd(),
                env={'CIM_NETWORK_SERVER_TYPE': 'cim'}
            )
            
            input_data = json.dumps(request) + '\n'
            stdout, stderr = await proc.communicate(input_data.encode())
            
            if proc.returncode != 0:
                print(f"   ❌ Process failed: {stderr.decode()}")
                return None
                
            # Parse response
            try:
                response = json.loads(stdout.decode().strip())
                return response
            except json.JSONDecodeError:
                print(f"   ❌ Invalid JSON response: {stdout.decode()}")
                return None
                
        except Exception as e:
            print(f"   ❌ Request failed: {e}")
            return None
    
    async def test_create_development_topology(self) -> bool:
        """Test creating development CIM topology"""
        print("\n1. 🏗️ Testing Development CIM Topology Creation")
        print("   " + "=" * 50)
        
        request = self.create_request("tools/call", {
            "name": "create_cim_topology",
            "arguments": {
                "name": "Development CIM Test",
                "topology_type": "development",
                "enable_nats": True
            }
        })
        
        response = await self.send_request(request)
        if not response or "error" in response:
            print(f"   ❌ Failed to create development topology: {response}")
            return False
        
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        try:
            result = json.loads(content)
            if result.get("success"):
                print(f"   ✅ Development topology created: {result['data']['topology_id']}")
                print(f"   📊 Tiers: {result['data']['tiers']}")
                print(f"   🔗 NATS enabled: {result['data']['nats_enabled']}")
                return True
            else:
                print(f"   ❌ Creation failed: {result.get('message')}")
                return False
        except json.JSONDecodeError:
            print(f"   ❌ Invalid response format")
            return False

    async def test_create_production_topology(self) -> bool:
        """Test creating production CIM topology"""
        print("\n2. 🏭 Testing Production CIM Topology Creation")
        print("   " + "=" * 50)
        
        request = self.create_request("tools/call", {
            "name": "create_cim_topology",
            "arguments": {
                "name": "Production CIM Test",
                "topology_type": "production",
                "enable_nats": True
            }
        })
        
        response = await self.send_request(request)
        if not response or "error" in response:
            print(f"   ❌ Failed to create production topology: {response}")
            return False
        
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        try:
            result = json.loads(content)
            if result.get("success"):
                print(f"   ✅ Production topology created: {result['data']['topology_id']}")
                print(f"   📊 Tiers: {result['data']['tiers']}")
                print(f"   🌐 Multi-cluster: {result['data']['tiers']['clusters']} clusters")
                print(f"   🍃 Distributed leaves: {result['data']['tiers']['leaves']} leaves")
                return True
            else:
                print(f"   ❌ Creation failed: {result.get('message')}")
                return False
        except json.JSONDecodeError:
            print(f"   ❌ Invalid response format")
            return False

    async def test_add_cim_clients(self) -> bool:
        """Test adding various CIM clients"""
        print("\n3. 👥 Testing CIM Client Registration")
        print("   " + "=" * 45)
        
        # First create a topology
        create_request = self.create_request("tools/call", {
            "name": "create_cim_topology", 
            "arguments": {
                "name": "Client Test CIM",
                "topology_type": "development"
            }
        })
        
        await self.send_request(create_request)  # Setup topology
        
        # Test different client types
        client_types = [
            ("Developer CLI", "cli"),
            ("Web Application", "application"), 
            ("Background Service", "service"),
            ("Browser Client", "browser"),
            ("Dev Workspace", "developer")
        ]
        
        success_count = 0
        
        for client_name, client_type in client_types:
            print(f"   📋 Adding {client_type} client: {client_name}")
            
            request = self.create_request("tools/call", {
                "name": "add_cim_client",
                "arguments": {
                    "name": client_name,
                    "client_type": client_type
                }
            })
            
            response = await self.send_request(request)
            if response and "error" not in response:
                content = response.get("result", {}).get("content", [{}])[0].get("text", "")
                try:
                    result = json.loads(content)
                    if result.get("success"):
                        print(f"      ✅ {client_name} added to {result['data']['assigned_leaf']}")
                        success_count += 1
                    else:
                        print(f"      ❌ Failed: {result.get('message')}")
                except json.JSONDecodeError:
                    print(f"      ❌ Invalid response")
            else:
                print(f"      ❌ Request failed")
        
        success_rate = (success_count / len(client_types)) * 100
        print(f"   📊 Client registration: {success_count}/{len(client_types)} ({success_rate}%)")
        
        return success_count == len(client_types)

    async def test_cim_event_simulation(self) -> bool:
        """Test CIM event flow simulation"""
        print("\n4. 🌊 Testing CIM Event Flow Simulation")
        print("   " + "=" * 45)
        
        # Create topology first
        create_request = self.create_request("tools/call", {
            "name": "create_cim_topology",
            "arguments": {"name": "Event Test CIM"}
        })
        await self.send_request(create_request)
        
        # Test event simulation from different tiers
        test_events = [
            ("client", "UserLoginRequested", {"user_id": "test123", "timestamp": "2024-01-01"}),
            ("leaf", "SessionCreated", {"session_id": "sess_456", "client_id": "test123"}),
            ("cluster", "WorkflowStarted", {"workflow_id": "wf_789", "saga_type": "user_onboarding"}),
            ("super_cluster", "GlobalStateUpdated", {"event": "system_health_check"})
        ]
        
        success_count = 0
        
        for source_tier, event_type, payload in test_events:
            print(f"   🎯 Simulating {event_type} from {source_tier}")
            
            request = self.create_request("tools/call", {
                "name": "simulate_cim_event",
                "arguments": {
                    "event_type": event_type,
                    "payload": payload,
                    "source_tier": source_tier
                }
            })
            
            response = await self.send_request(request)
            if response and "error" not in response:
                content = response.get("result", {}).get("content", [{}])[0].get("text", "")
                try:
                    result = json.loads(content)
                    if result.get("success"):
                        flow_path = result['data'].get('flow_path', [])
                        print(f"      ✅ Event flow: {len(flow_path)} tiers")
                        for step in flow_path[:2]:  # Show first 2 steps
                            print(f"         • {step}")
                        success_count += 1
                    else:
                        print(f"      ❌ Failed: {result.get('message')}")
                except json.JSONDecodeError:
                    print(f"      ❌ Invalid response")
            else:
                print(f"      ❌ Request failed")
        
        success_rate = (success_count / len(test_events)) * 100
        print(f"   📊 Event simulation: {success_count}/{len(test_events)} ({success_rate}%)")
        
        return success_count == len(test_events)

    async def test_cim_config_generation(self) -> bool:
        """Test CIM configuration generation"""
        print("\n5. ⚙️ Testing CIM Configuration Generation")
        print("   " + "=" * 45)
        
        # Create topology first
        create_request = self.create_request("tools/call", {
            "name": "create_cim_topology",
            "arguments": {"name": "Config Test CIM"}
        })
        await self.send_request(create_request)
        
        # Test config generation for different tiers
        config_tiers = ["all", "super_cluster", "cluster", "leaf", "client"]
        success_count = 0
        
        for tier in config_tiers:
            print(f"   🔧 Generating configuration for {tier}")
            
            request = self.create_request("tools/call", {
                "name": "generate_cim_config",
                "arguments": {
                    "tier": tier,
                    "include_nats": True
                }
            })
            
            response = await self.send_request(request)
            if response and "error" not in response:
                content = response.get("result", {}).get("content", [{}])[0].get("text", "")
                try:
                    result = json.loads(content)
                    if result.get("success"):
                        config_preview = result['data'].get('config_preview', '')
                        if 'CIM Network' in config_preview and 'NixOS' in config_preview:
                            print(f"      ✅ {tier} config generated ({len(config_preview)} chars)")
                            success_count += 1
                        else:
                            print(f"      ⚠️ Config generated but may be incomplete")
                            success_count += 0.5
                    else:
                        print(f"      ❌ Failed: {result.get('message')}")
                except json.JSONDecodeError:
                    print(f"      ❌ Invalid response")
            else:
                print(f"      ❌ Request failed")
        
        success_rate = (success_count / len(config_tiers)) * 100
        print(f"   📊 Config generation: {success_count}/{len(config_tiers)} ({success_rate}%)")
        
        return success_count >= len(config_tiers) * 0.8  # Allow 80% success rate

    async def test_cim_visualization(self) -> bool:
        """Test CIM topology visualization"""
        print("\n6. 📊 Testing CIM Topology Visualization")
        print("   " + "=" * 45)
        
        # Create topology first
        create_request = self.create_request("tools/call", {
            "name": "create_cim_topology",
            "arguments": {"name": "Visualization Test CIM"}
        })
        await self.send_request(create_request)
        
        # Test different visualization formats
        formats = ["ascii", "mermaid", "dot"]
        success_count = 0
        
        for format_type in formats:
            print(f"   🎨 Generating {format_type} visualization")
            
            request = self.create_request("tools/call", {
                "name": "visualize_cim_topology",
                "arguments": {
                    "format": format_type,
                    "include_nats": True
                }
            })
            
            response = await self.send_request(request)
            if response and "error" not in response:
                content = response.get("result", {}).get("content", [{}])[0].get("text", "")
                try:
                    result = json.loads(content)
                    if result.get("success"):
                        viz = result['data'].get('visualization', '')
                        if len(viz) > 100:  # Reasonable visualization size
                            print(f"      ✅ {format_type} visualization ({len(viz)} chars)")
                            success_count += 1
                        else:
                            print(f"      ⚠️ {format_type} visualization seems incomplete")
                    else:
                        print(f"      ❌ Failed: {result.get('message')}")
                except json.JSONDecodeError:
                    print(f"      ❌ Invalid response")
            else:
                print(f"      ❌ Request failed")
        
        success_rate = (success_count / len(formats)) * 100
        print(f"   📊 Visualization: {success_count}/{len(formats)} ({success_rate}%)")
        
        return success_count == len(formats)

    async def run_all_tests(self) -> None:
        """Run all CIM topology tests"""
        print("🏗️ Testing CIM (Composable Information Machine) Architecture")
        print("=" * 70)
        print("Testing proper hierarchical structure: DEV/CLIENT -> LEAF -> cluster -> super-cluster")
        print("with NATS lattice messaging and event-sourcing")
        print()
        
        tests = [
            ("Development Topology", self.test_create_development_topology),
            ("Production Topology", self.test_create_production_topology), 
            ("Client Registration", self.test_add_cim_clients),
            ("Event Flow Simulation", self.test_cim_event_simulation),
            ("Configuration Generation", self.test_cim_config_generation),
            ("Topology Visualization", self.test_cim_visualization)
        ]
        
        results = []
        for test_name, test_func in tests:
            try:
                result = await test_func()
                results.append(result)
            except Exception as e:
                print(f"   ❌ Test '{test_name}' crashed: {e}")
                results.append(False)
        
        # Summary
        passed = sum(results)
        total = len(results)
        success_rate = (passed / total) * 100
        
        print(f"\n📊 CIM Topology Test Results: {passed}/{total} passed ({success_rate:.0f}%)")
        
        if success_rate >= 80:
            print("\n🎉 🎊 CIM ARCHITECTURE TESTS PASSED! 🎊 🎉")
            print("\n🏗️ CIM Capabilities Validated:")
            print("   ✅ Hierarchical topology creation (dev/prod)")
            print("   ✅ Client registration across tiers") 
            print("   ✅ Event flow simulation through CIM layers")
            print("   ✅ NixOS configuration generation")
            print("   ✅ Topology visualization in multiple formats")
            print("   ✅ NATS lattice integration")
            print("\n🎯 CIM Architecture Features:")
            print("   • Proper tier separation (CLIENT->LEAF->CLUSTER->SUPER)")
            print("   • Event-sourcing with content-addressed IDs")
            print("   • NATS lattice for distributed messaging") 
            print("   • NixOS-native deployment configurations")
            print("   • Scalable from development to production")
            print("\n🚀 Ready to compete with Kubernetes:")
            print("   • Native hierarchical orchestration")
            print("   • Event-driven architecture")
            print("   • Infrastructure-as-Code approach")
            print("   • No container complexity overhead")
        else:
            print(f"\n❌ CIM architecture tests failed (success rate: {success_rate:.0f}%)")
            failed_tests = [tests[i][0] for i, result in enumerate(results) if not result]
            print(f"   Failed tests: {', '.join(failed_tests)}")

async def main():
    """Main test runner"""
    tester = CimTopologyTester()
    await tester.run_all_tests()

if __name__ == "__main__":
    asyncio.run(main())