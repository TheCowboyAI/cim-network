# cim-network

Network infrastructure domain module for the Composable Information Machine (CIM).

## Status

🚧 **Fresh Start** - This module is being rebuilt from scratch to properly integrate with cim-domain v0.8.x patterns.

## Architecture

cim-network implements network infrastructure as a CIM domain using:

- **cim-domain** aggregate state machines for network device lifecycle
- **Port/Adapter pattern** for vendor-specific implementations (UniFi first)
- **Graph-based topology** using cim-graph for network relationships
- **Event-sourced state** for complete audit trails

## Vendor Adapters

Adapters are located in `src/adapters/` and implement the core ports:

- `unifi/` - Ubiquiti UniFi Controller integration (first adapter)

## Previous Implementation

The previous prototype is archived in the `prototype-archive` branch for reference.

## License

MIT OR Apache-2.0
