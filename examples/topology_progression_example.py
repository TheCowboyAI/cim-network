#!/usr/bin/env python3
"""
Topology Progression Example

This example demonstrates how to evolve from a dev mode topology to a leaf mode 
topology as your organization grows and requirements change.

Scenario: A startup growing from development phase to production deployment
"""

import json
import asyncio
from pathlib import Path
import sys

# Add the project root to Python path
sys.path.insert(0, str(Path(__file__).parent.parent))

from cim_network_mcp.sdn_server import SDNMCPServer

class TopologyProgressionExample:
    def __init__(self):
        self.server = SDNMCPServer()
    
    async def demonstrate_topology_evolution(self):
        """Show how to evolve from dev to production topology"""
        print("📈 Network Topology Evolution: Dev → Production")
        print("=" * 65)
        
        # Phase 1: Startup Development Phase
        print("\n🚀 PHASE 1: Startup Development (Dev Mode)")
        print("=" * 50)
        print("Scenario: Small team, limited budget, single office")
        
        dev_response = await self.server.execute_sdn_command("create_base_topology", {
            "mode": "dev",
            "name": "startup-network", 
            "primary_isp": "local-isp"
        })
        
        if dev_response["success"]:
            print("   ✅ Development network established")
            print("   💰 Cost: ~$200/month (single ISP + basic equipment)")
            print("   👥 Team: 2-5 developers")
            print("   📊 Capacity: 1 public IP, basic services")
            
            # Add minimal development services
            await self.server.execute_sdn_command("add_sdn_node", {
                "node_id": "dev-server",
                "node_type": "server",
                "tier": "client", 
                "interfaces": [{"name": "eth0", "type": "ethernet", "addresses": ["dhcp"]}],
                "services": ["git-server", "ci-cd", "development-db"],
                "metadata": {"role": "all-in-one-dev-server", "phase": "startup"}
            })
            
            print("   🛠️  Development Infrastructure:")
            print("      • 1 router with single ISP")
            print("      • 8-port switch")
            print("      • 2 development workstations")  
            print("      • 1 all-in-one development server")
            print("      • Git repository and basic CI/CD")
        
        # Generate dev mode Nix config
        dev_nix = await self.server.execute_sdn_command("generate_nix_topology", {
            "format": "nixos", "mode": "dev"
        })
        
        if dev_nix["success"]:
            print("   📄 Generated NixOS configuration for development")
        
        # Phase 2: Growth Phase Assessment
        print("\n📊 PHASE 2: Growth Assessment")
        print("=" * 40)
        print("After 6 months of growth:")
        print("   📈 Team size: 15 people")
        print("   💼 First enterprise customers")
        print("   ⚠️  Reliability concerns emerging")
        print("   📊 Single ISP becomes risk factor")
        print("   🔒 Security requirements increasing")
        
        print("\n❗ Migration Triggers Identified:")
        print("   • Customer SLA requirements (99.9% uptime)")
        print("   • Single point of failure (ISP)")
        print("   • Need for public-facing services")  
        print("   • Regulatory compliance requirements")
        print("   • Team working on production systems")
        
        # Phase 3: Migration Planning
        print("\n🎯 PHASE 3: Migration Planning")
        print("=" * 35)
        print("Planning the evolution to production infrastructure:")
        
        migration_plan = {
            "current_limitations": [
                "Single ISP (no redundancy)",
                "1 public IP (insufficient for services)", 
                "8-port switch (capacity limit)",
                "Home-grade network (192.168.1.0/24)",
                "No high availability"
            ],
            "production_requirements": [
                "Dual ISP with automatic failover", 
                "16 public IPs for service hosting",
                "24+ port enterprise switch",
                "Enterprise network addressing",
                "High availability services"
            ]
        }
        
        print("   📋 Migration Requirements Analysis:")
        for i, (current, future) in enumerate(zip(migration_plan["current_limitations"], 
                                                 migration_plan["production_requirements"])):
            print(f"   {i+1}. Current: {current}")
            print(f"      Future:  {future}")
        
        # Phase 4: Production Deployment
        print("\n🏭 PHASE 4: Production Deployment (Leaf Mode)")
        print("=" * 50)
        print("Deploying production-grade infrastructure:")
        
        leaf_response = await self.server.execute_sdn_command("create_base_topology", {
            "mode": "leaf",
            "name": "production-network",
            "primary_isp": "tier1-provider",
            "failover_isp": "backup-provider"
        })
        
        if leaf_response["success"]:
            print("   ✅ Production network infrastructure deployed")
            print("   💰 Cost: ~$2,500/month (dual ISP + enterprise equipment)")
            print("   👥 Team: 15+ people across multiple teams")
            print("   📊 Capacity: 16 public IPs, enterprise services")
            
            # Add production infrastructure
            production_services = [
                {"id": "prod-lb-01", "role": "load-balancer", "tier": "leaf"},
                {"id": "prod-app-01", "role": "application-server", "tier": "cluster"},
                {"id": "prod-app-02", "role": "application-server", "tier": "cluster"},
                {"id": "prod-db-primary", "role": "database-primary", "tier": "cluster"},
                {"id": "prod-db-replica", "role": "database-replica", "tier": "cluster"},
                {"id": "prod-cache-01", "role": "cache-server", "tier": "cluster"},
                {"id": "prod-monitor-01", "role": "monitoring", "tier": "leaf"}
            ]
            
            for service in production_services:
                await self.server.execute_sdn_command("add_sdn_node", {
                    "node_id": service["id"],
                    "node_type": "server",
                    "tier": service["tier"],
                    "interfaces": [{"name": "eth0", "type": "ethernet", "addresses": ["dhcp"]}],
                    "services": ["systemd", "monitoring"],
                    "metadata": {"role": service["role"], "phase": "production"}
                })
            
            print("   🏭 Production Infrastructure:")
            print("      • High-availability router (dual ISP)")
            print("      • 24-port enterprise switch with VLAN")
            print("      • Load balancer with SSL termination")
            print("      • 2-node application cluster") 
            print("      • Database with read replica")
            print("      • Dedicated cache and monitoring servers")
        
        # Generate production Nix config
        prod_nix = await self.server.execute_sdn_command("generate_nix_topology", {
            "format": "nixos", "mode": "leaf"
        })
        
        if prod_nix["success"]:
            print("   📄 Generated production NixOS configuration")
        
        # Phase 5: Migration Benefits Analysis
        print("\n📊 PHASE 5: Migration Benefits Analysis")
        print("=" * 45)
        
        comparison_table = [
            ("Aspect", "Dev Mode", "Leaf Mode", "Improvement"),
            ("ISP Connections", "1", "2 (with failover)", "99.9% → 99.95% uptime"),
            ("Public IPs", "1", "16", "15x capacity for services"),
            ("Network Ports", "8", "24", "3x device capacity"),
            ("IP Range", "192.168.1.0/24", "10.0.1.0/24", "Enterprise standard"),
            ("Monthly Cost", "$200", "$2,500", "12.5x but justified by revenue"),
            ("Team Capacity", "2-5 devs", "15+ people", "Scales with business growth"),
            ("SLA Capability", "No guarantees", "99.9% uptime", "Enterprise customer ready"),
            ("Security Level", "Basic", "Enterprise", "Compliance ready")
        ]
        
        print("\n   📈 Before & After Comparison:")
        for aspect, dev, leaf, improvement in comparison_table:
            if aspect == "Aspect":
                print(f"   {aspect:<15} | {dev:<15} | {leaf:<20} | {improvement}")
                print("   " + "-" * 75)
            else:
                print(f"   {aspect:<15} | {dev:<15} | {leaf:<20} | {improvement}")
        
        # Phase 6: Future Scaling Path
        print("\n🚀 PHASE 6: Future Scaling Considerations")
        print("=" * 45)
        print("Planning for continued growth:")
        
        scaling_path = [
            "Multi-site deployment (additional leaf nodes)",
            "Data center colocation for core services", 
            "Cloud hybrid deployment (AWS/GCP integration)",
            "CDN integration for global performance",
            "Kubernetes orchestration layer",
            "Service mesh for microservices",
            "Multi-region disaster recovery"
        ]
        
        print("   🎯 Next Evolution Milestones:")
        for i, milestone in enumerate(scaling_path, 1):
            print(f"   {i}. {milestone}")
        
        print("\n💡 Key Lessons from Topology Evolution:")
        print("   ✅ Start simple with dev mode for early development")
        print("   ✅ Plan migration triggers based on business needs")
        print("   ✅ Leaf mode provides production-ready foundation") 
        print("   ✅ Cost scales with business value and requirements")
        print("   ✅ Infrastructure evolution enables business growth")
        print("   ✅ NixOS configurations make deployments reproducible")
        
        print("\n🎉 Topology Evolution Planning Complete!")
        print("\n📋 Recommended Migration Timeline:")
        print("   • Weeks 1-2: Plan and procure equipment")
        print("   • Weeks 3-4: Deploy parallel leaf infrastructure")
        print("   • Weeks 5-6: Migrate services with minimal downtime")
        print("   • Weeks 7-8: Optimize and monitor production systems")
        print("   • Week 9+: Decommission development infrastructure")

async def main():
    """Run the topology progression example"""
    example = TopologyProgressionExample()
    
    try:
        await example.demonstrate_topology_evolution()
        print("\n🚀 Topology progression example completed successfully!")
    except Exception as e:
        print(f"\n💥 Example failed: {e}")
        sys.exit(1)

if __name__ == "__main__":
    asyncio.run(main())