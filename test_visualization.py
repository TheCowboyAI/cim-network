#!/usr/bin/env python3
"""
Network Topology Visualization Test

Test the visualization capabilities of the CIM Network SDN system,
demonstrating ASCII, Mermaid, DOT, and SVG output formats.
"""

import json
import subprocess
import sys
import asyncio
from typing import Dict, Any, List
import uuid

class VisualizationTester:
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

    async def test_ascii_visualization(self) -> bool:
        """Test ASCII art topology visualization"""
        print("1. 🎨 Testing ASCII Visualization")
        print("   " + "=" * 40)
        
        request = self.create_request("tools/call", {
            "name": "visualize_topology",
            "arguments": {
                "format": "ascii",
                "layout": "tier-based",
                "color_scheme": "default",
                "show_details": True
            }
        })
        
        response = await self.send_request(request)
        if not response or "error" in response:
            print(f"   ❌ ASCII visualization failed: {response}")
            return False
        
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        if "ascii topology visualization" in content.lower():
            print("   ✅ ASCII visualization generated successfully")
            
            # Show a portion of the ASCII art
            if "NETWORK TOPOLOGY (ASCII)" in content:
                lines = content.split('\n')
                relevant_lines = []
                in_diagram = False
                
                for line in lines:
                    if "NETWORK TOPOLOGY (ASCII)" in line:
                        in_diagram = True
                    if in_diagram and ("╔" in line or "║" in line or "╠" in line or "╚" in line):
                        relevant_lines.append(line)
                    if in_diagram and len(relevant_lines) >= 8:
                        break
                
                if relevant_lines:
                    print("   📊 ASCII Preview:")
                    for line in relevant_lines:
                        print(f"      {line}")
                    if len(relevant_lines) >= 8:
                        print("      ... (truncated)")
            
            return True
        else:
            print(f"   ❌ Unexpected ASCII response: {content}")
            return False

    async def test_mermaid_visualization(self) -> bool:
        """Test Mermaid diagram visualization"""
        print("\n2. 🌊 Testing Mermaid Visualization")
        print("   " + "=" * 40)
        
        request = self.create_request("tools/call", {
            "name": "visualize_topology",
            "arguments": {
                "format": "mermaid",
                "layout": "hierarchical",
                "color_scheme": "blue",
                "show_details": True
            }
        })
        
        response = await self.send_request(request)
        if not response or "error" in response:
            print(f"   ❌ Mermaid visualization failed: {response}")
            return False
        
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        if "mermaid topology visualization" in content.lower():
            print("   ✅ Mermaid diagram generated successfully")
            
            # Show key elements of the Mermaid diagram
            if "graph TD" in content:
                lines = content.split('\n')
                mermaid_lines = []
                for line in lines:
                    if any(keyword in line for keyword in ["graph TD", "subgraph", "ISP", "Router", "Switch", "classDef"]):
                        mermaid_lines.append(line.strip())
                        if len(mermaid_lines) >= 6:
                            break
                
                if mermaid_lines:
                    print("   🌊 Mermaid Preview:")
                    for line in mermaid_lines:
                        print(f"      {line}")
                    print("      ... (truncated)")
            
            return True
        else:
            print(f"   ❌ Unexpected Mermaid response: {content}")
            return False

    async def test_dot_visualization(self) -> bool:
        """Test Graphviz DOT visualization"""
        print("\n3. 🔗 Testing Graphviz DOT Visualization")
        print("   " + "=" * 40)
        
        request = self.create_request("tools/call", {
            "name": "visualize_topology",
            "arguments": {
                "format": "dot",
                "layout": "hierarchical",
                "color_scheme": "default",
                "show_details": True
            }
        })
        
        response = await self.send_request(request)
        if not response or "error" in response:
            print(f"   ❌ DOT visualization failed: {response}")
            return False
        
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        if "dot topology visualization" in content.lower():
            print("   ✅ Graphviz DOT diagram generated successfully")
            
            # Show key elements of the DOT diagram
            if "digraph NetworkTopology" in content:
                lines = content.split('\n')
                dot_lines = []
                for line in lines:
                    if any(keyword in line for keyword in ["digraph", "subgraph", "ISP", "Router", "Switch", "label="]):
                        dot_lines.append(line.strip())
                        if len(dot_lines) >= 6:
                            break
                
                if dot_lines:
                    print("   🔗 DOT Preview:")
                    for line in dot_lines:
                        print(f"      {line}")
                    print("      ... (truncated)")
            
            return True
        else:
            print(f"   ❌ Unexpected DOT response: {content}")
            return False

    async def test_svg_visualization(self) -> bool:
        """Test SVG visualization"""
        print("\n4. 🖼️  Testing SVG Visualization")
        print("   " + "=" * 40)
        
        request = self.create_request("tools/call", {
            "name": "visualize_topology", 
            "arguments": {
                "format": "svg",
                "layout": "tier-based",
                "color_scheme": "enterprise",
                "show_details": True
            }
        })
        
        response = await self.send_request(request)
        if not response or "error" in response:
            print(f"   ❌ SVG visualization failed: {response}")
            return False
        
        content = response.get("result", {}).get("content", [{}])[0].get("text", "")
        if "svg topology visualization" in content.lower():
            print("   ✅ SVG diagram generated successfully")
            
            # Show key elements of the SVG
            if "<svg" in content and "viewBox" in content:
                lines = content.split('\n')
                svg_lines = []
                for line in lines:
                    if any(keyword in line for keyword in ["<svg", "<style>", "WAN Tier", "Leaf Tier", "<ellipse", "<polygon"]):
                        svg_lines.append(line.strip())
                        if len(svg_lines) >= 6:
                            break
                
                if svg_lines:
                    print("   🖼️  SVG Preview:")
                    for line in svg_lines:
                        print(f"      {line}")
                    print("      ... (truncated)")
            
            return True
        else:
            print(f"   ❌ Unexpected SVG response: {content}")
            return False

    async def test_visualization_with_topologies(self) -> bool:
        """Test visualization with different base topologies"""
        print("\n5. 🏗️  Testing Visualization with Base Topologies")
        print("   " + "=" * 50)
        
        # Test with dev mode
        print("   📋 Creating dev mode topology for visualization...")
        dev_request = self.create_request("tools/call", {
            "name": "create_base_topology",
            "arguments": {
                "mode": "dev",
                "name": "visual-test-dev",
                "primary_isp": "test-isp"
            }
        })
        
        response = await self.send_request(dev_request)
        if response and "error" not in response:
            print("   ✅ Dev topology created for visualization")
        
        # Visualize dev topology
        vis_request = self.create_request("tools/call", {
            "name": "visualize_topology",
            "arguments": {
                "format": "ascii",
                "layout": "tier-based"
            }
        })
        
        response = await self.send_request(vis_request)
        if response and "error" not in response:
            print("   ✅ Dev topology visualization generated")
        
        # Test with leaf mode
        print("   📋 Creating leaf mode topology for visualization...")
        leaf_request = self.create_request("tools/call", {
            "name": "create_base_topology",
            "arguments": {
                "mode": "leaf",
                "name": "visual-test-leaf",
                "primary_isp": "primary-isp",
                "failover_isp": "failover-isp"
            }
        })
        
        response = await self.send_request(leaf_request)
        if response and "error" not in response:
            print("   ✅ Leaf topology created for visualization")
        
        # Visualize leaf topology with different format
        vis_request = self.create_request("tools/call", {
            "name": "visualize_topology",
            "arguments": {
                "format": "mermaid",
                "layout": "hierarchical",
                "color_scheme": "blue"
            }
        })
        
        response = await self.send_request(vis_request)
        if response and "error" not in response:
            print("   ✅ Leaf topology visualization generated")
            return True
        
        return False

    async def run_visualization_tests(self) -> bool:
        """Run all visualization tests"""
        print("🎨 Testing CIM Network Topology Visualization")
        print("=" * 60)
        print("Testing multiple visualization formats and layouts")
        
        test_functions = [
            self.test_ascii_visualization,
            self.test_mermaid_visualization,
            self.test_dot_visualization,
            self.test_svg_visualization,
            self.test_visualization_with_topologies
        ]
        
        results = []
        for test_func in test_functions:
            try:
                success = await test_func()
                results.append(success)
            except Exception as e:
                print(f"   ❌ Test {test_func.__name__} crashed: {e}")
                results.append(False)
        
        successful = sum(results)
        total = len(results)
        
        print(f"\n📊 Visualization Test Results: {successful}/{total} passed")
        
        if successful == total:
            print("\n🎉 🎊 ALL VISUALIZATION TESTS PASSED! 🎊 🎉")
            print("\n🎨 Visualization Capabilities Validated:")
            print("   ✅ ASCII art diagrams for terminal display")
            print("   ✅ Mermaid diagrams for documentation")
            print("   ✅ Graphviz DOT for professional presentations")
            print("   ✅ SVG graphics for web integration")
            print("   ✅ Multiple color schemes and layouts")
            print("   ✅ Integration with base topology modes")
            
            print("\n🔧 Usage Examples:")
            print("   • ASCII: Perfect for CLI tools and terminal output")
            print("   • Mermaid: Great for GitHub documentation")
            print("   • DOT: Professional network diagrams")
            print("   • SVG: Interactive web visualizations")
            
            print("\n🚀 Ready for Integration:")
            print("   • Claude Code MCP server visualization tool")
            print("   • Network documentation generation")
            print("   • Interactive topology exploration")
            print("   • Multi-format diagram export")
            
            return True
        else:
            print(f"\n❌ {total - successful} visualization tests failed")
            return False

async def main():
    """Run the visualization tests"""
    tester = VisualizationTester()
    
    try:
        success = await tester.run_visualization_tests()
        print(f"\n🚀 Visualization testing {'completed successfully' if success else 'failed'}!")
        sys.exit(0 if success else 1)
    except Exception as e:
        print(f"\n💥 Visualization test crashed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())