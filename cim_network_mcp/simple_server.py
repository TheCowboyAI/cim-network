#!/usr/bin/env python3
"""
Network Topology Builder Simple MCP Server

A simplified MCP-compatible server that communicates via JSON-RPC over stdio.
This version doesn't require the mcp package and can work in restricted environments.
"""

import json
import sys
import asyncio
import subprocess
import tempfile
import uuid
from pathlib import Path
from typing import Dict, Any, List, Optional


class SimpleMCPServer:
    """Simple MCP server implementation using JSON-RPC over stdio"""
    
    def __init__(self):
        self.project_root = Path(__file__).parent.parent
        self.tools = self._define_tools()
    
    def _define_tools(self) -> List[Dict[str, Any]]:
        """Define available tools"""
        return [
            {
                "name": "build_topology",
                "description": "Start building a new network topology with optional initial parameters",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "base_network": {"type": "string", "description": "Base IP network range"},
                        "target_environment": {"type": "string", "description": "Target deployment environment"},
                        "scale": {"type": "string", "description": "Expected scale (small/medium/large/enterprise)"},
                        "use_case": {"type": "string", "description": "Primary use case description"}
                    }
                }
            },
            {
                "name": "add_location", 
                "description": "Add a network location to the topology",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "location_id": {"type": "string", "description": "Unique identifier"},
                        "location_type": {"type": "string", "enum": ["datacenter", "office", "cloud", "edge", "segment"]},
                        "parameters": {"type": "object", "additionalProperties": {"type": "string"}}
                    },
                    "required": ["location_id", "location_type"]
                }
            },
            {
                "name": "connect_locations",
                "description": "Connect two locations in the topology", 
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "from_location": {"type": "string"},
                        "to_location": {"type": "string"},
                        "connection_type": {"type": "string", "enum": ["fiber", "vpn", "internet", "directconnect", "virtual"]},
                        "parameters": {"type": "object", "additionalProperties": {"type": "string"}}
                    },
                    "required": ["from_location", "to_location", "connection_type"]
                }
            },
            {
                "name": "generate_configuration",
                "description": "Generate network configuration",
                "inputSchema": {
                    "type": "object", 
                    "properties": {
                        "format": {"type": "string", "enum": ["nixos", "nix-darwin", "home-manager", "flake", "json"]}
                    },
                    "required": ["format"]
                }
            },
            {
                "name": "validate_topology",
                "description": "Validate the current network topology",
                "inputSchema": {"type": "object", "properties": {}}
            },
            {
                "name": "get_topology_status", 
                "description": "Get current topology status",
                "inputSchema": {"type": "object", "properties": {}}
            },
            {
                "name": "reset_topology",
                "description": "Reset the current topology",
                "inputSchema": {"type": "object", "properties": {}}
            },
            {
                "name": "complete_topology",
                "description": "Complete the topology building process", 
                "inputSchema": {"type": "object", "properties": {}}
            }
        ]
    
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
            
            process = await asyncio.create_subprocess_exec(
                *cmd,
                cwd=self.project_root,
                stdout=asyncio.subprocess.PIPE,
                stderr=asyncio.subprocess.PIPE
            )
            
            stdout, stderr = await process.communicate()
            
            # Clean up temporary file
            Path(request_file).unlink()
            
            if process.returncode == 0:
                try:
                    response = json.loads(stdout.decode().strip())
                    return response
                except json.JSONDecodeError:
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
    
    def handle_initialize(self, params: Dict[str, Any]) -> Dict[str, Any]:
        """Handle initialize request"""
        return {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "logging": {}
            },
            "serverInfo": {
                "name": "network-topology-builder",
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
        
        # Create sub-agent request
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
                "task": {"BuildTopology": {"initial_params": initial_params}},
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
                "task": {"GenerateConfiguration": {"format": arguments["format"]}},
                "context": {}
            }
            
        elif name in ["validate_topology", "get_topology_status", "reset_topology", "complete_topology"]:
            task_map = {
                "validate_topology": "ValidateTopology",
                "get_topology_status": "GetStatus", 
                "reset_topology": "Reset",
                "complete_topology": "Complete"
            }
            request = {
                "request_id": request_id,
                "task": task_map[name],
                "context": {}
            }
            
        else:
            return {
                "content": [{"type": "text", "text": f"Unknown tool: {name}"}],
                "isError": True
            }
        
        # Execute the request
        response = await self.execute_subagent_request(request)
        
        # Format response
        if response["success"]:
            result_text = f"✅ {response['message']}\n"
            
            if response.get("data"):
                result_text += f"\nData: {json.dumps(response['data'], indent=2)}\n"
            
            if response.get("suggested_actions"):
                result_text += f"\nSuggested next actions:\n"
                for action in response["suggested_actions"]:
                    result_text += f"• {action}\n"
        else:
            result_text = f"❌ {response['message']}\n"
            if response.get("errors"):
                result_text += f"\nErrors:\n"
                for error in response["errors"]:
                    result_text += f"• {error}\n"
        
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
        """Run the MCP server"""
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
    server = SimpleMCPServer()
    await server.run()


if __name__ == "__main__":
    asyncio.run(main())