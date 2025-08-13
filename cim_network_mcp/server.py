#!/usr/bin/env python3
"""
Network Topology Builder MCP Server

This MCP server provides Claude Code with tools to interactively build network topologies
using event-driven context graphs through the Rust-based NetworkTopologySubAgent.
"""

import json
import subprocess
import tempfile
import uuid
import asyncio
from pathlib import Path
from typing import Dict, Any, List, Optional, Sequence

from mcp.server import Server
from mcp.server.models import InitializationOptions
from mcp.server.stdio import stdio_server
from mcp.types import (
    Tool, 
    TextContent,
    CallToolRequest,
    CallToolResult,
    GetToolsRequest,
    GetToolsResult,
)

# Server instance
server = Server("network-topology-builder")

# Project root path
PROJECT_ROOT = Path(__file__).parent.parent


class NetworkTopologyMCP:
    """MCP server for the Network Topology Builder sub-agent"""
    
    def __init__(self):
        self.project_root = PROJECT_ROOT
        self.session_state = {}
    
    async def execute_subagent_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
        """Execute a sub-agent request via the Rust binary"""
        try:
            # Create temporary file with request JSON
            with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
                json.dump(request, f, indent=2)
                request_file = f.name
            
            # Execute the subagent demo with the request
            cmd = [
                "cargo", "run", "--example", "subagent_demo", "--", 
                "--request-file", request_file
            ]
            
            result = await asyncio.create_subprocess_exec(
                *cmd,
                cwd=self.project_root,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            
            stdout, stderr = await result.communicate()
            
            # Clean up temporary file
            Path(request_file).unlink()
            
            if result.returncode == 0:
                # Parse JSON response from stdout
                try:
                    response = json.loads(stdout.decode().strip())
                    return response
                except json.JSONDecodeError:
                    # Fallback to text response
                    return {
                        "success": True,
                        "message": stdout.decode().strip(),
                        "data": {},
                        "errors": []
                    }
            else:
                return {
                    "success": False,
                    "message": f"Sub-agent execution failed: {stderr.decode()}",
                    "data": {},
                    "errors": [stderr.decode()]
                }
                
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to execute sub-agent: {str(e)}",
                "data": {},
                "errors": [str(e)]
            }


# Global MCP instance
network_mcp = NetworkTopologyMCP()


@server.list_tools()
async def list_tools() -> Sequence[Tool]:
    """List available MCP tools"""
    return [
        Tool(
            name="build_topology",
            description="Start building a new network topology with optional initial parameters",
            inputSchema={
                "type": "object",
                "properties": {
                    "base_network": {
                        "type": "string",
                        "description": "Base IP network range (e.g., '10.0.0.0/8')"
                    },
                    "target_environment": {
                        "type": "string",
                        "description": "Target deployment environment (e.g., 'production', 'staging')"
                    },
                    "scale": {
                        "type": "string",
                        "description": "Expected scale (small/medium/large/enterprise)"
                    },
                    "use_case": {
                        "type": "string",
                        "description": "Primary use case description"
                    }
                }
            }
        ),
        Tool(
            name="add_location",
            description="Add a network location to the topology",
            inputSchema={
                "type": "object",
                "properties": {
                    "location_id": {
                        "type": "string",
                        "description": "Unique identifier for the location"
                    },
                    "location_type": {
                        "type": "string",
                        "enum": ["datacenter", "office", "cloud", "edge", "segment"],
                        "description": "Type of network location"
                    },
                    "parameters": {
                        "type": "object",
                        "description": "Location-specific parameters",
                        "additionalProperties": {"type": "string"}
                    }
                },
                "required": ["location_id", "location_type"]
            }
        ),
        Tool(
            name="connect_locations",
            description="Connect two locations in the topology",
            inputSchema={
                "type": "object",
                "properties": {
                    "from_location": {
                        "type": "string",
                        "description": "Source location ID"
                    },
                    "to_location": {
                        "type": "string",
                        "description": "Destination location ID"
                    },
                    "connection_type": {
                        "type": "string",
                        "enum": ["fiber", "vpn", "internet", "directconnect", "virtual"],
                        "description": "Type of connection"
                    },
                    "parameters": {
                        "type": "object",
                        "description": "Connection-specific parameters",
                        "additionalProperties": {"type": "string"}
                    }
                },
                "required": ["from_location", "to_location", "connection_type"]
            }
        ),
        Tool(
            name="generate_configuration",
            description="Generate network configuration in specified format",
            inputSchema={
                "type": "object",
                "properties": {
                    "format": {
                        "type": "string",
                        "enum": ["nixos", "terraform", "ansible", "json", "yaml"],
                        "description": "Configuration output format"
                    }
                },
                "required": ["format"]
            }
        ),
        Tool(
            name="validate_topology",
            description="Validate the current network topology for completeness and correctness",
            inputSchema={
                "type": "object",
                "properties": {}
            }
        ),
        Tool(
            name="get_topology_status",
            description="Get current topology status and summary",
            inputSchema={
                "type": "object",
                "properties": {}
            }
        ),
        Tool(
            name="reset_topology",
            description="Reset the current topology to start over",
            inputSchema={
                "type": "object",
                "properties": {}
            }
        ),
        Tool(
            name="complete_topology",
            description="Complete the topology building process",
            inputSchema={
                "type": "object",
                "properties": {}
            }
        )
    ]


@server.call_tool()
async def call_tool(name: str, arguments: Dict[str, Any]) -> Sequence[TextContent]:
    """Handle tool calls"""
    try:
        # Create the appropriate sub-agent request based on the tool
        request_id = str(uuid.uuid4())
        
        if name == "build_topology":
            initial_params = None
            if any(key in arguments for key in ["base_network", "target_environment", "scale", "use_case"]):
                initial_params = {
                    "base_network": arguments.get("base_network"),
                    "target_environment": arguments.get("target_environment"),
                    "scale": arguments.get("scale"),
                    "use_case": arguments.get("use_case")
                }
            
            request = {
                "request_id": request_id,
                "task": {
                    "BuildTopology": {
                        "initial_params": initial_params
                    }
                },
                "context": {}
            }
            
        elif name == "add_location":
            request = {
                "request_id": request_id,
                "task": {
                    "AddLocation": {
                        "location_id": arguments["location_id"],
                        "location_type": arguments["location_type"],
                        "parameters": arguments.get("parameters", {})
                    }
                },
                "context": {}
            }
            
        elif name == "connect_locations":
            request = {
                "request_id": request_id,
                "task": {
                    "ConnectLocations": {
                        "from": arguments["from_location"],
                        "to": arguments["to_location"],
                        "connection_type": arguments["connection_type"],
                        "parameters": arguments.get("parameters", {})
                    }
                },
                "context": {}
            }
            
        elif name == "generate_configuration":
            request = {
                "request_id": request_id,
                "task": {
                    "GenerateConfiguration": {
                        "format": arguments["format"]
                    }
                },
                "context": {}
            }
            
        elif name == "validate_topology":
            request = {
                "request_id": request_id,
                "task": "ValidateTopology",
                "context": {}
            }
            
        elif name == "get_topology_status":
            request = {
                "request_id": request_id,
                "task": "GetStatus",
                "context": {}
            }
            
        elif name == "reset_topology":
            request = {
                "request_id": request_id,
                "task": "Reset",
                "context": {}
            }
            
        elif name == "complete_topology":
            request = {
                "request_id": request_id,
                "task": "Complete",
                "context": {}
            }
            
        else:
            return [TextContent(type="text", text=f"Unknown tool: {name}")]
        
        # Execute the request
        response = await network_mcp.execute_subagent_request(request)
        
        # Format response
        if response["success"]:
            result_text = f"✅ {response['message']}\n"
            
            if "data" in response and response["data"]:
                result_text += f"\nData: {json.dumps(response['data'], indent=2)}\n"
            
            if "suggested_actions" in response and response["suggested_actions"]:
                result_text += f"\nSuggested next actions:\n"
                for action in response["suggested_actions"]:
                    result_text += f"• {action}\n"
        else:
            result_text = f"❌ {response['message']}\n"
            if response.get("errors"):
                result_text += f"\nErrors:\n"
                for error in response["errors"]:
                    result_text += f"• {error}\n"
        
        return [TextContent(type="text", text=result_text)]
        
    except Exception as e:
        return [TextContent(type="text", text=f"Error executing tool {name}: {str(e)}")]


async def main():
    """Main entry point for the MCP server"""
    async with stdio_server() as (read_stream, write_stream):
        await server.run(
            read_stream,
            write_stream,
            InitializationOptions(
                server_name="network-topology-builder",
                server_version="1.0.0",
                capabilities=server.get_capabilities(
                    notification_options=None,
                    experimental_capabilities=None,
                ),
            ),
        )


if __name__ == "__main__":
    asyncio.run(main())