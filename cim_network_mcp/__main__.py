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
    server_type = os.getenv('CIM_NETWORK_SERVER_TYPE', 'cim')
    
    if server_type == 'cim':
        from .cim_mcp_server import main as cim_main
        asyncio.run(cim_main())
    elif server_type == 'sdn':
        from .sdn_server import main as sdn_main
        asyncio.run(sdn_main())
    else:
        # Fallback to CIM (default)
        from .cim_mcp_server import main as cim_main
        asyncio.run(cim_main())

if __name__ == "__main__":
    main()