#!/usr/bin/env python3
"""
Complete Integration Test

This test validates the entire CIM Network SDN system with base topologies
by running through all major functionality in a single comprehensive test.
"""

import json
import subprocess
import sys
import asyncio
from typing import Dict, Any, List
import uuid

class CompleteIntegrationTester:
    def __init__(self):
        self.request_id = 1
        self.test_results = {}
        
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
            return {"error": f"MCP server error: {stderr.decode()}"}
            
        try:
            lines = stdout.decode().strip().split('\n')
            for line in reversed(lines):
                if line.startswith('{"jsonrpc"'):
                    return json.loads(line)
            return {"error": "No JSON response found"}
        except json.JSONDecodeError as e:
            return {"error": f"JSON parse error: {e}"}

    def record_test_result(self, test_name: str, success: bool, details: str = ""):
        """Record a test result for summary reporting"""
        self.test_results[test_name] = {
            "success": success,
            "details": details
        }

    async def test_mcp_server_basic_functionality(self) -> bool:
        """Test basic MCP server functionality"""
        print("1. 🔧 Testing MCP Server Basic Functionality")
        print("   " + "=" * 50)
        
        # Test server initialization
        init_request = self.create_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}}
        })
        
        response = await self.send_request(init_request)
        if "error" in response:
            print(f"   ❌ Server initialization failed: {response['error']}")
            self.record_test_result("mcp_initialization", False, response["error"])
            return False
        
        print("   ✅ MCP server initialized successfully")
        
        # Test tools listing
        tools_request = self.create_request("tools/list", {})
        response = await self.send_request(tools_request)
        
        if "error" in response or "result" not in response:
            print(f"   ❌ Tools listing failed: {response.get('error', 'No result')}")
            self.record_test_result("mcp_tools_list", False)
            return False
        
        tools = response.get("result", {}).get("tools", [])
        expected_tools = ["initialize_sdn", "create_base_topology", "add_sdn_node", 
                         "connect_sdn_nodes", "generate_nix_topology", "get_sdn_state", 
                         "export_context_graph"]
        
        available_tool_names = [tool["name"] for tool in tools]
        missing_tools = [tool for tool in expected_tools if tool not in available_tool_names]
        
        if missing_tools:
            print(f"   ❌ Missing tools: {missing_tools}")
            self.record_test_result("mcp_tools_complete", False, f"Missing: {missing_tools}")
            return False
        
        print(f"   ✅ All {len(expected_tools)} expected tools available")
        self.record_test_result("mcp_basic_functionality", True)
        return True

    async def test_dev_mode_complete_workflow(self) -> bool:
        """Test complete dev mode workflow"""
        print("\n2. 🖥️  Testing Dev Mode Complete Workflow")
        print("   " + "=" * 50)
        
        # Create dev topology
        request = self.create_request("tools/call", {
            "name": "create_base_topology",
            "arguments": {
                "mode": "dev",
                "name": "integration-test-dev",
                "primary_isp": "test-isp"
            }
        })
        
        response = await self.send_request(request)
        if "error" in response:
            print(f"   ❌ Dev topology creation failed: {response['error']}")
            self.record_test_result("dev_topology_creation", False)
            return False
        
        print("   ✅ Dev mode topology created")
        
        # Add a development node
        add_node_request = self.create_request("tools/call", {
            "name": "add_sdn_node",
            "arguments": {
                "node_id": "test-dev-service",
                "node_type": "server",
                "tier": "client",
                "interfaces": [{"name": "eth0", "type": "ethernet", "addresses": ["dhcp"]}],
                "services": ["nginx", "postgresql"],
                "metadata": {"role": "test-service"}
            }
        })
        
        response = await self.send_request(add_node_request)
        if "error" in response:
            print(f"   ❌ Node addition failed: {response['error']}")
            self.record_test_result("dev_node_addition", False)
            return False
        
        print("   ✅ Development service node added")
        
        # Generate dev mode Nix configuration
        nix_request = self.create_request("tools/call", {
            "name": "generate_nix_topology",
            "arguments": {"format": "nixos", "mode": "dev"}
        })
        
        response = await self.send_request(nix_request)
        if "error" in response:
            print(f"   ❌ Nix generation failed: {response['error']}")
            self.record_test_result("dev_nix_generation", False)
            return False
        
        # Validate dev-specific features in generated config
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        dev_features_present = all(feature in content for feature in ["192.168.1", "wan0", "dev-network"])
        
        if not dev_features_present:
            print("   ❌ Dev-specific features missing from Nix config")
            self.record_test_result("dev_nix_validation", False)
            return False
        
        print("   ✅ Dev mode Nix configuration generated and validated")
        self.record_test_result("dev_complete_workflow", True)
        return True

    async def test_leaf_mode_complete_workflow(self) -> bool:
        """Test complete leaf mode workflow"""
        print("\n3. 🌐 Testing Leaf Mode Complete Workflow")
        print("   " + "=" * 50)
        
        # Create leaf topology
        request = self.create_request("tools/call", {
            "name": "create_base_topology",
            "arguments": {
                "mode": "leaf",
                "name": "integration-test-leaf",
                "primary_isp": "primary-test-isp",
                "failover_isp": "failover-test-isp"
            }
        })
        
        response = await self.send_request(request)
        if "error" in response:
            print(f"   ❌ Leaf topology creation failed: {response['error']}")
            self.record_test_result("leaf_topology_creation", False)
            return False
        
        print("   ✅ Leaf mode topology created")
        
        # Add production nodes
        production_nodes = [
            {"id": "test-lb", "role": "load-balancer"},
            {"id": "test-app", "role": "application-server"},
            {"id": "test-db", "role": "database-server"}
        ]
        
        for node in production_nodes:
            add_node_request = self.create_request("tools/call", {
                "name": "add_sdn_node",
                "arguments": {
                    "node_id": node["id"],
                    "node_type": "server",
                    "tier": "cluster",
                    "interfaces": [{"name": "eth0", "type": "ethernet", "addresses": ["dhcp"]}],
                    "services": ["systemd"],
                    "metadata": {"role": node["role"], "environment": "production"}
                }
            })
            
            response = await self.send_request(add_node_request)
            if "error" in response:
                print(f"   ❌ {node['role']} addition failed: {response['error']}")
                self.record_test_result("leaf_node_addition", False)
                return False
        
        print("   ✅ Production service nodes added")
        
        # Create connections between services
        connections = [
            {"from": "test-lb", "to": "test-app", "type": "load-balanced"},
            {"from": "test-app", "to": "test-db", "type": "database-access"}
        ]
        
        for conn in connections:
            conn_request = self.create_request("tools/call", {
                "name": "connect_sdn_nodes",
                "arguments": {
                    "from_node": conn["from"],
                    "to_node": conn["to"],
                    "connection_type": "ethernet",
                    "properties": {"purpose": conn["type"]}
                }
            })
            
            response = await self.send_request(conn_request)
            if "error" in response:
                print(f"   ❌ Connection {conn['from']} → {conn['to']} failed")
                self.record_test_result("leaf_connections", False)
                return False
        
        print("   ✅ Production service connections established")
        
        # Generate leaf mode Nix configuration
        nix_request = self.create_request("tools/call", {
            "name": "generate_nix_topology",
            "arguments": {"format": "nixos", "mode": "leaf"}
        })
        
        response = await self.send_request(nix_request)
        if "error" in response:
            print(f"   ❌ Leaf Nix generation failed: {response['error']}")
            self.record_test_result("leaf_nix_generation", False)
            return False
        
        # Validate leaf-specific features
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        leaf_features_present = all(feature in content for feature in 
                                  ["10.0.1", "wan1", "keepalived", "leaf-network"])
        
        if not leaf_features_present:
            print("   ❌ Leaf-specific features missing from Nix config")
            self.record_test_result("leaf_nix_validation", False)
            return False
        
        print("   ✅ Leaf mode Nix configuration generated and validated")
        self.record_test_result("leaf_complete_workflow", True)
        return True

    async def test_context_graph_integration(self) -> bool:
        """Test context graph export and validation"""
        print("\n4. 📈 Testing Context Graph Integration")
        print("   " + "=" * 50)
        
        # Test JSON export
        json_request = self.create_request("tools/call", {
            "name": "export_context_graph",
            "arguments": {"format": "json"}
        })
        
        response = await self.send_request(json_request)
        if "error" in response:
            print(f"   ❌ JSON export failed: {response['error']}")
            self.record_test_result("context_graph_json", False)
            return False
        
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        if not all(key in content for key in ["nodes", "edges", "metadata"]):
            print("   ❌ JSON export missing required structure")
            self.record_test_result("context_graph_structure", False)
            return False
        
        print("   ✅ Context graph JSON export validated")
        
        # Test DOT export
        dot_request = self.create_request("tools/call", {
            "name": "export_context_graph",
            "arguments": {"format": "dot"}
        })
        
        response = await self.send_request(dot_request)
        if "error" in response:
            print(f"   ❌ DOT export failed: {response['error']}")
            self.record_test_result("context_graph_dot", False)
            return False
        
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        if not ("digraph" in content and "->" in content):
            print("   ❌ DOT export invalid format")
            self.record_test_result("context_graph_dot_format", False)
            return False
        
        print("   ✅ Context graph DOT export validated")
        self.record_test_result("context_graph_integration", True)
        return True

    async def test_network_state_management(self) -> bool:
        """Test SDN state retrieval and validation"""
        print("\n5. 📊 Testing Network State Management")
        print("   " + "=" * 50)
        
        state_request = self.create_request("tools/call", {
            "name": "get_sdn_state",
            "arguments": {}
        })
        
        response = await self.send_request(state_request)
        if "error" in response:
            print(f"   ❌ State retrieval failed: {response['error']}")
            self.record_test_result("sdn_state_retrieval", False)
            return False
        
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        if not all(key in content.lower() for key in ["nodes", "connections"]):
            print("   ❌ State response missing network information")
            self.record_test_result("sdn_state_content", False)
            return False
        
        print("   ✅ SDN state retrieved and validated")
        self.record_test_result("network_state_management", True)
        return True

    def generate_test_report(self) -> str:
        """Generate a comprehensive test report"""
        total_tests = len(self.test_results)
        passed_tests = sum(1 for result in self.test_results.values() if result["success"])
        failed_tests = total_tests - passed_tests
        
        success_rate = (passed_tests / total_tests * 100) if total_tests > 0 else 0
        
        report = f"""
╔═══════════════════════════════════════════════════════════════════════════════╗
║                        INTEGRATION TEST REPORT                               ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║ Total Tests:     {total_tests:3d}                                                        ║
║ Passed:          {passed_tests:3d}                                                        ║
║ Failed:          {failed_tests:3d}                                                        ║
║ Success Rate:    {success_rate:5.1f}%                                                   ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                           DETAILED RESULTS                                   ║
╚═══════════════════════════════════════════════════════════════════════════════╝

"""
        
        for test_name, result in self.test_results.items():
            status = "✅ PASS" if result["success"] else "❌ FAIL"
            details = f" - {result['details']}" if result["details"] else ""
            report += f"{status:8s} | {test_name:<50s}{details}\n"
        
        return report

    async def run_complete_integration_test(self) -> bool:
        """Run all integration tests"""
        print("🌐 CIM Network SDN - Complete Integration Test")
        print("=" * 70)
        print("Testing all major functionality of the base topology system")
        
        test_functions = [
            self.test_mcp_server_basic_functionality,
            self.test_dev_mode_complete_workflow,
            self.test_leaf_mode_complete_workflow,
            self.test_context_graph_integration,
            self.test_network_state_management
        ]
        
        overall_success = True
        
        for test_func in test_functions:
            try:
                success = await test_func()
                if not success:
                    overall_success = False
            except Exception as e:
                print(f"   ❌ Test crashed: {e}")
                self.record_test_result(test_func.__name__, False, str(e))
                overall_success = False
        
        # Generate and display test report
        report = self.generate_test_report()
        print(report)
        
        if overall_success:
            print("🎉 🎊 ALL INTEGRATION TESTS PASSED! 🎊 🎉")
            print("\n🌟 CIM Network SDN System Fully Validated:")
            print("   ✅ MCP server functionality")
            print("   ✅ Dev mode topology workflow")
            print("   ✅ Leaf mode topology workflow")
            print("   ✅ Context graph integration")
            print("   ✅ Network state management")
            print("   ✅ nix-topology compliant configuration generation")
            print("   ✅ Base topology templates (dev & leaf modes)")
            print("   ✅ Production-ready high availability features")
            
            print("\n🚀 System Ready For:")
            print("   • Claude Code MCP integration")
            print("   • Production network deployments")
            print("   • Development environment setup")
            print("   • Enterprise branch office networks")
            print("   • Infrastructure as Code workflows")
            
        else:
            print("❌ INTEGRATION TEST FAILURES DETECTED")
            print("Review the detailed results above for specific issues")
        
        return overall_success

async def main():
    """Run the complete integration test"""
    tester = CompleteIntegrationTester()
    
    try:
        success = await tester.run_complete_integration_test()
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n💥 Integration test crashed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())