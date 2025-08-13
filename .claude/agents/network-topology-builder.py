#!/usr/bin/env python3
"""
Network Topology Builder Sub-Agent for Claude Code

This sub-agent enables Claude Code to interactively build network topologies
using event-driven context graphs. It provides a conversational interface
for creating complex network infrastructures.
"""

import json
import subprocess
import sys
import tempfile
import uuid
from pathlib import Path
from typing import Dict, Any, List, Optional


class NetworkTopologyBuilderAgent:
    """Claude Code sub-agent for building network topologies"""
    
    def __init__(self):
        self.name = "Network Topology Builder"
        self.version = "1.0.0"
        self.description = "Interactive sub-agent for building network topologies using event-driven context graphs"
        self.capabilities = [
            "Add network locations (data centers, offices, cloud regions, virtual segments)",
            "Connect locations with various connection types (fiber, VPN, internet, direct connect)",
            "Validate network topology completeness and correctness",
            "Generate configurations in multiple formats (NixOS, Terraform, Ansible, JSON, YAML)",
            "Provide intelligent suggestions based on current topology state",
            "Maintain full event audit trail for topology construction"
        ]
        self.project_root = Path(__file__).parent.parent.parent
    
    def build_topology(self, initial_params: Optional[Dict[str, str]] = None) -> Dict[str, Any]:
        """Start building a new network topology"""
        request = {
            "request_id": str(uuid.uuid4()),
            "task": {
                "BuildTopology": {
                    "initial_params": initial_params
                }
            },
            "context": {}
        }
        return self._execute_subagent_request(request)
    
    def add_location(self, location_id: str, location_type: str, parameters: Dict[str, str]) -> Dict[str, Any]:
        """Add a network location to the topology"""
        request = {
            "request_id": str(uuid.uuid4()),
            "task": {
                "AddLocation": {
                    "location_id": location_id,
                    "location_type": location_type,
                    "parameters": parameters
                }
            },
            "context": {}
        }
        return self._execute_subagent_request(request)
    
    def connect_locations(self, from_location: str, to_location: str, 
                         connection_type: str, parameters: Dict[str, str]) -> Dict[str, Any]:
        """Connect two locations in the topology"""
        request = {
            "request_id": str(uuid.uuid4()),
            "task": {
                "ConnectLocations": {
                    "from": from_location,
                    "to": to_location,
                    "connection_type": connection_type,
                    "parameters": parameters
                }
            },
            "context": {}
        }
        return self._execute_subagent_request(request)
    
    def generate_configuration(self, format_type: str) -> Dict[str, Any]:
        """Generate network configuration in specified format"""
        request = {
            "request_id": str(uuid.uuid4()),
            "task": {
                "GenerateConfiguration": {
                    "format": format_type
                }
            },
            "context": {}
        }
        return self._execute_subagent_request(request)
    
    def validate_topology(self) -> Dict[str, Any]:
        """Validate the current network topology"""
        request = {
            "request_id": str(uuid.uuid4()),
            "task": "ValidateTopology",
            "context": {}
        }
        return self._execute_subagent_request(request)
    
    def get_status(self) -> Dict[str, Any]:
        """Get current topology status"""
        request = {
            "request_id": str(uuid.uuid4()),
            "task": "GetStatus", 
            "context": {}
        }
        return self._execute_subagent_request(request)
    
    def reset_topology(self) -> Dict[str, Any]:
        """Reset the current topology"""
        request = {
            "request_id": str(uuid.uuid4()),
            "task": "Reset",
            "context": {}
        }
        return self._execute_subagent_request(request)
    
    def complete_topology(self) -> Dict[str, Any]:
        """Complete the topology building process"""
        request = {
            "request_id": str(uuid.uuid4()),
            "task": "Complete",
            "context": {}
        }
        return self._execute_subagent_request(request)
    
    def _execute_subagent_request(self, request: Dict[str, Any]) -> Dict[str, Any]:
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
            
            result = subprocess.run(
                cmd,
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=30
            )
            
            # Clean up temporary file
            Path(request_file).unlink()
            
            if result.returncode == 0:
                # Parse JSON response from stdout
                try:
                    response = json.loads(result.stdout.strip())
                    return response
                except json.JSONDecodeError:
                    # Fallback to text response
                    return {
                        "success": True,
                        "message": result.stdout.strip(),
                        "data": {},
                        "errors": []
                    }
            else:
                return {
                    "success": False,
                    "message": f"Sub-agent execution failed: {result.stderr}",
                    "data": {},
                    "errors": [result.stderr]
                }
                
        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "message": "Sub-agent request timed out",
                "data": {},
                "errors": ["Request timeout"]
            }
        except Exception as e:
            return {
                "success": False,
                "message": f"Failed to execute sub-agent: {str(e)}",
                "data": {},
                "errors": [str(e)]
            }
    
    def process_natural_language_request(self, user_input: str) -> Dict[str, Any]:
        """Process natural language input and convert to appropriate sub-agent calls"""
        user_input_lower = user_input.lower().strip()
        
        # Start topology building
        if any(phrase in user_input_lower for phrase in [
            "start building", "create topology", "new topology", "build network"
        ]):
            return self.build_topology()
        
        # Add data center
        elif any(phrase in user_input_lower for phrase in [
            "add datacenter", "add data center", "create datacenter"
        ]):
            params = {"name": "Data Center", "region": "us-west-1"}
            return self.add_location("dc1", "datacenter", params)
        
        # Add office
        elif any(phrase in user_input_lower for phrase in [
            "add office", "create office", "add branch"
        ]):
            params = {"name": "Office", "address": "Corporate Campus", "size": "medium"}
            return self.add_location("office1", "office", params)
        
        # Add cloud region
        elif any(phrase in user_input_lower for phrase in [
            "add cloud", "add aws", "add azure", "cloud region"
        ]):
            provider = "aws"
            if "azure" in user_input_lower:
                provider = "azure"
            elif "gcp" in user_input_lower or "google" in user_input_lower:
                provider = "gcp"
            
            params = {"provider": provider, "region": "us-east-1"}
            return self.add_location("cloud1", "cloud", params)
        
        # Connect locations
        elif any(phrase in user_input_lower for phrase in [
            "connect", "link", "join"
        ]):
            # Default fiber connection
            params = {"bandwidth": "1Gbps", "redundant": "false"}
            return self.connect_locations("dc1", "office1", "fiber", params)
        
        # Generate configuration
        elif any(phrase in user_input_lower for phrase in [
            "generate config", "create config", "export", "nixos", "terraform"
        ]):
            format_type = "json"
            if "nixos" in user_input_lower or "nix" in user_input_lower:
                format_type = "nixos"
            elif "terraform" in user_input_lower:
                format_type = "terraform"
            elif "ansible" in user_input_lower:
                format_type = "ansible"
            elif "yaml" in user_input_lower:
                format_type = "yaml"
            
            return self.generate_configuration(format_type)
        
        # Validate topology
        elif any(phrase in user_input_lower for phrase in [
            "validate", "check", "verify"
        ]):
            return self.validate_topology()
        
        # Get status
        elif any(phrase in user_input_lower for phrase in [
            "status", "show", "list", "current"
        ]):
            return self.get_status()
        
        # Reset
        elif any(phrase in user_input_lower for phrase in [
            "reset", "clear", "start over"
        ]):
            return self.reset_topology()
        
        # Complete
        elif any(phrase in user_input_lower for phrase in [
            "complete", "finish", "done"
        ]):
            return self.complete_topology()
        
        # Default: show help
        else:
            return {
                "success": True,
                "message": "I can help you build network topologies! Try commands like:\n"
                          "• 'start building a topology'\n"
                          "• 'add a datacenter'\n" 
                          "• 'add an office'\n"
                          "• 'add a cloud region'\n"
                          "• 'connect the locations'\n"
                          "• 'generate nixos configuration'\n"
                          "• 'validate the topology'\n"
                          "• 'show status'\n"
                          "• 'complete the topology'",
                "data": {"capabilities": self.capabilities},
                "errors": []
            }


def main():
    """Main entry point for Claude Code sub-agent"""
    agent = NetworkTopologyBuilderAgent()
    
    if len(sys.argv) < 2:
        print("Usage: network-topology-builder.py '<natural language request>'")
        print(f"Available capabilities: {', '.join(agent.capabilities)}")
        sys.exit(1)
    
    user_request = " ".join(sys.argv[1:])
    response = agent.process_natural_language_request(user_request)
    
    # Output response as JSON for Claude Code
    print(json.dumps(response, indent=2))


if __name__ == "__main__":
    main()