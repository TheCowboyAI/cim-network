#!/usr/bin/env python3
"""
Main entry point for the CIM Network MCP server

Use the SDN-focused server by default, which implements the correct approach:
1. Start from a domain established in cim-start
2. Build Software Defined Network using cim-graph ContextGraph
3. Generate nix-topology compliant Nix files as projections
"""

import asyncio
import sys
import os

def main():
    # Check for server type preference
    server_type = os.getenv('CIM_NETWORK_SERVER_TYPE', 'sdn')
    
    if server_type == 'sdn':
        from .sdn_server import main as sdn_main
        asyncio.run(sdn_main())
    else:
        # Fallback to the old complex server
        from .simple_server import main as simple_main
        asyncio.run(simple_main())

if __name__ == "__main__":
    main()