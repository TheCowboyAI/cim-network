#!/usr/bin/env python3
"""
Dev Mode Topology Example

This example demonstrates how to create and customize a dev mode base topology,
then extend it with additional services and configurations.

Scenario: Setting up a development environment for a small software team
"""

import json
import asyncio
from pathlib import Path
import sys

# Add the project root to Python path
sys.path.insert(0, str(Path(__file__).parent.parent))

from cim_network_mcp.sdn_server import SDNMCPServer

class DevTopologyExample:
    def __init__(self):
        self.server = SDNMCPServer()
    
    async def create_dev_environment(self):
        """Create a complete development environment"""
        print("🚀 Creating Development Environment")
        print("=" * 50)
        
        # Step 1: Create base dev topology
        print("\n1. 📋 Creating Dev Mode Base Topology")
        topology_response = await self.server.execute_sdn_command("create_base_topology", {
            "mode": "dev",
            "name": "acme-dev",
            "primary_isp": "spectrum"
        })
        
        if topology_response["success"]:
            print(f"   ✅ {topology_response['message']}")
            topology_data = topology_response["data"]
            
            # Show key components
            print("   📦 Base Components:")
            for node_name, node_data in topology_data.get("nodes", {}).items():
                node_type = node_data.get("type", "unknown")
                role = node_data.get("metadata", {}).get("role", "unknown")
                print(f"      • {node_name}: {node_type} ({role})")
        else:
            print(f"   ❌ Failed: {topology_response['message']}")
            return
        
        # Step 2: Add development servers
        print("\n2. 🖥️  Adding Development Services")
        
        # Add a development database server
        db_response = await self.server.execute_sdn_command("add_sdn_node", {
            "node_id": "acme-dev-db",
            "node_type": "server",
            "tier": "client",
            "interfaces": [
                {
                    "name": "eth0",
                    "type": "ethernet",
                    "addresses": ["dhcp"]
                }
            ],
            "services": ["postgresql", "redis", "backup-agent"],
            "metadata": {
                "role": "development-database",
                "environment": "development",
                "location": "dev-lab"
            }
        })
        
        if db_response["success"]:
            print(f"   ✅ Added development database server")
        
        # Add a web development server
        web_response = await self.server.execute_sdn_command("add_sdn_node", {
            "node_id": "acme-dev-web",
            "node_type": "server", 
            "tier": "client",
            "interfaces": [
                {
                    "name": "eth0",
                    "type": "ethernet",
                    "addresses": ["dhcp"]
                }
            ],
            "services": ["nginx", "nodejs", "docker"],
            "metadata": {
                "role": "web-development-server",
                "environment": "development",
                "location": "dev-lab"
            }
        })
        
        if web_response["success"]:
            print(f"   ✅ Added web development server")
        
        # Step 3: Connect the services
        print("\n3. 🔗 Establishing Service Connections")
        
        # Connect dev machine to database
        db_conn_response = await self.server.execute_sdn_command("connect_sdn_nodes", {
            "from_node": "acme-dev-dev-machine",
            "to_node": "acme-dev-db", 
            "connection_type": "ethernet",
            "properties": {
                "bandwidth": "1Gbps",
                "purpose": "database-access",
                "vlan": "dev-services"
            }
        })
        
        # Connect dev machine to web server
        web_conn_response = await self.server.execute_sdn_command("connect_sdn_nodes", {
            "from_node": "acme-dev-dev-machine",
            "to_node": "acme-dev-web",
            "connection_type": "ethernet", 
            "properties": {
                "bandwidth": "1Gbps",
                "purpose": "web-development",
                "vlan": "dev-services"
            }
        })
        
        if db_conn_response["success"] and web_conn_response["success"]:
            print("   ✅ Connected development services")
        
        # Step 4: Generate development-optimized Nix configuration
        print("\n4. 🔧 Generating Development NixOS Configuration")
        
        nix_response = await self.server.execute_sdn_command("generate_nix_topology", {
            "format": "nixos",
            "mode": "dev"
        })
        
        if nix_response["success"]:
            print(f"   ✅ {nix_response['message']}")
            
            # Show key development features in the config
            config = nix_response["data"]["configuration"]
            dev_features = []
            
            if "docker" in config:
                dev_features.append("Docker containerization")
            if "git" in config:
                dev_features.append("Git development tools")  
            if "192.168.1" in config:
                dev_features.append("Home network IP range")
            if "wan0" in config:
                dev_features.append("Single ISP connection")
            
            if dev_features:
                print("   🛠️  Development Features:")
                for feature in dev_features:
                    print(f"      • {feature}")
        
        # Step 5: Show network state
        print("\n5. 📊 Development Network State")
        
        state_response = await self.server.execute_sdn_command("get_sdn_state", {})
        if state_response["success"]:
            print("   ✅ Network topology ready for development")
            print("   📈 Development Environment Summary:")
            print("      • 1 Router with NAT and firewall")
            print("      • 1 8-port access switch")
            print("      • 1 Development workstation")
            print("      • 1 Database server (PostgreSQL + Redis)")
            print("      • 1 Web development server (Node.js + Docker)")
            print("      • Single ISP connection (Spectrum)")
            print("      • Network: 192.168.1.0/24")
        
        # Step 6: Export context graph
        print("\n6. 📈 Exporting Development Context Graph")
        
        graph_response = await self.server.execute_sdn_command("export_context_graph", {
            "format": "json"
        })
        
        if graph_response["success"]:
            print("   ✅ Context graph exported for documentation")
            print("   📋 Use cases for this development environment:")
            print("      • Full-stack web application development")
            print("      • Database-driven application testing")  
            print("      • Container-based development workflows")
            print("      • Team collaboration and code sharing")
            print("      • Local CI/CD pipeline testing")
        
        print("\n🎉 Development Environment Setup Complete!")
        print("\n💡 Next Steps:")
        print("   • Deploy the NixOS configuration to your hardware")
        print("   • Configure development tools and IDEs")
        print("   • Set up Git repositories and collaboration tools")
        print("   • Implement backup and version control workflows")
        print("   • Consider upgrading to Leaf mode for production")

async def main():
    """Run the dev topology example"""
    example = DevTopologyExample()
    
    try:
        await example.create_dev_environment()
        print("\n🚀 Dev topology example completed successfully!")
    except Exception as e:
        print(f"\n💥 Example failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())