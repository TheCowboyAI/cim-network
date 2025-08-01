# CIM Network Module Index

## Module Rules Priority

1. **[CIM_NETWORK_MODULE.md](./CIM_NETWORK_MODULE.md)** - Module-specific rules (HIGHEST PRIORITY)
2. **[MANDATORY_CHECKLIST.md](./MANDATORY_CHECKLIST.md)** - Always check before committing
3. **[CLAUDE.md](./CLAUDE.md)** - General CIM rules
4. **[INDEX.md](./INDEX.md)** - Parent project structure reference

## Implementation Order

### Phase 1: Foundation (CURRENT)
1. ✅ Copy all rules from parent
2. ⏳ Create design document with user stories
3. ⏳ Create test scenarios
4. ⏳ Implement TDD - tests first

### Phase 2: Domain Model
1. Value objects with validation
2. Aggregates following DDD
3. Events with MANDATORY correlation/causation
4. State machines with phantom types

### Phase 3: Infrastructure
1. Configuration generators (Cisco first)
2. Nix topology generation
3. Deployment automation

### Phase 4: Integration
1. Container network with VLAN
2. VM network management
3. Context graph projections

## Critical Rules

### MUST HAVE
- Every event has correlation_id and causation_id (NEVER optional)
- All state transitions use phantom types
- TDD - write tests FIRST
- No unwrap() in production code
- Follow event sourcing patterns

### MUST NOT HAVE
- Graph theory algorithms (Hopfield, etc.)
- CRUD operations
- Direct state mutations
- Missing correlation IDs

## File Naming
- Use lowercase_with_underscores.rs
- No uppercase in filenames
- Use date -I for dates (YYYY-MM-DD)

## Testing
- Unit tests for every public function
- Integration tests for workflows
- Property tests for invariants
- 100% coverage goal

## Documentation
- Mermaid diagrams MANDATORY
- Document WHY not WHAT
- User stories drive development