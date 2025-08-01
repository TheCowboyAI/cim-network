# CIM Network Module Rules and Context

## Module Purpose
The cim-network module manages real-world network infrastructure through event-driven architecture. NOT graph theory algorithms.

## What This Module IS
1. **Network Infrastructure Management**
   - Router configurations (Cisco IOS, JunOS, VyOS, etc.)
   - Switch configurations and VLAN management
   - IP address allocation and subnet management
   - Container and VM network provisioning

2. **Nix Integration**
   - Generate nix-topology configurations
   - Deploy network configurations via Nix
   - Reproducible network deployments

3. **Event-Driven Network State**
   - All changes through events with correlation/causation IDs
   - Complete audit trail of network changes
   - State machines for device lifecycle

## What This Module IS NOT
- NOT a graph theory library
- NOT Hopfield networks or neural networks
- NOT small-world or scale-free network algorithms
- NOT academic network analysis

## Core Requirements
1. **MANDATORY Event Patterns**
   - Every event MUST have correlation_id (never optional)
   - Every event MUST have causation_id (never optional)
   - Use EventMetadata structure consistently

2. **Domain Model**
   - Physical devices: Router, Switch, Firewall, LoadBalancer
   - Virtual networks: Container networks, VM networks
   - Network primitives: VLAN, Subnet, IP allocation

3. **User Stories to Implement**
   - As a network admin, I can define router configurations that deploy automatically
   - As a DevOps engineer, I can provision container networks integrated with VLANs
   - As a system architect, I can visualize network topology via context graphs
   - As an operator, I can deploy configurations through Nix

4. **Integration Points**
   - Generate nix-topology modules
   - Project to context graphs for visualization
   - Manage CIM leaf node networking
   - Configure container/VM networks

## Implementation Rules
1. Start with TDD - write tests first
2. Use phantom types for state machines
3. No unwrap() in production code
4. All public types need documentation
5. Follow DDD aggregate boundaries
6. Events are the source of truth

## Anti-patterns to Avoid
- Direct state mutation
- CRUD operations
- Missing correlation IDs
- Synchronous cross-context calls
- Graph theory algorithms (unless for visualization)