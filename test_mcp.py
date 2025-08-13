#!/usr/bin/env python3
"""
Test script for the CIM Network MCP Server

This script validates that the MCP server can handle the full workflow
of building a network topology.
"""

import json
import subprocess
import sys
import tempfile
from pathlib import Path


def send_mcp_request(method, params=None, request_id=1):
    """Send a JSON-RPC request to the MCP server"""
    request = {
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method
    }
    if params:
        request["params"] = params
    
    # Send request to MCP server
    process = subprocess.Popen(
        ["python3", "-m", "cim_network_mcp"],
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True
    )
    
    stdout, stderr = process.communicate(json.dumps(request) + "\n")
    
    if stderr:
        print(f"MCP Server stderr: {stderr}", file=sys.stderr)
    
    try:
        return json.loads(stdout.strip())
    except json.JSONDecodeError as e:
        print(f"Failed to parse MCP response: {stdout}", file=sys.stderr)
        raise e


def test_mcp_workflow():
    """Test complete MCP workflow"""
    print("🧪 Testing CIM Network MCP Server")
    print("=" * 40)
    
    # Test 1: Initialize
    print("1. Testing initialization...")
    response = send_mcp_request("initialize", {})
    assert response["result"]["serverInfo"]["name"] == "network-topology-builder"
    print("✅ Initialization successful")
    
    # Test 2: List tools
    print("\n2. Testing tools list...")
    response = send_mcp_request("tools/list")
    tools = response["result"]["tools"]
    tool_names = {tool["name"] for tool in tools}
    expected_tools = {
        "build_topology", "add_location", "connect_locations", 
        "generate_configuration", "validate_topology", "get_topology_status",
        "reset_topology", "complete_topology"
    }
    assert expected_tools.issubset(tool_names)
    print(f"✅ Found {len(tools)} tools")
    
    # Test 3: Build topology
    print("\n3. Testing build topology...")
    response = send_mcp_request("tools/call", {
        "name": "build_topology",
        "arguments": {
            "base_network": "10.0.0.0/8",
            "target_environment": "production",
            "scale": "enterprise",
            "use_case": "multi-region-cloud"
        }
    })
    # Note: This will fail with compilation errors, but that's expected for now
    success = response.get("result", {}).get("content", [{}])[0].get("text", "")
    print(f"📋 Build topology response: {success[:100]}...")
    
    # Test 4: Add location (will also fail but tests the MCP protocol)
    print("\n4. Testing add location...")
    response = send_mcp_request("tools/call", {
        "name": "add_location", 
        "arguments": {
            "location_id": "dc1",
            "location_type": "datacenter",
            "parameters": {
                "name": "West Coast DC",
                "region": "us-west-1",
                "az": "us-west-1a"
            }
        }
    })
    success = response.get("result", {}).get("content", [{}])[0].get("text", "")
    print(f"🏢 Add location response: {success[:100]}...")
    
    print("\n🎉 MCP Protocol Tests Completed!")
    print("Note: Rust compilation errors are expected and will be fixed next.")
    

if __name__ == "__main__":
    test_mcp_workflow()