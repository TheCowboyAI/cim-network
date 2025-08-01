# Progress Tracking Rules for cim-network

## This Module's Progress

Progress for the cim-network module is tracked in:
- `doc/progress.md` - Human-readable progress
- `doc/progress.json` - Machine-readable progress (if needed)

## What to Track Here

1. **Detailed Implementation Progress**
   - Each domain aggregate implementation
   - Each test file completion
   - Each integration milestone

2. **Module-Specific Metrics**
   - Test coverage percentage
   - Performance benchmarks
   - Lines of code
   - Compilation time

3. **Module Dependencies**
   - Dependencies on other CIM modules
   - External crate dependencies
   - System requirements

## What NOT to Track Here

1. Overall CIM project progress (that's in parent)
2. Other modules' progress
3. Parent project milestones

## Update Frequency

- Update after each significant implementation
- Update when tests are added
- Update when milestones are reached
- Always use system date: `date -I`

## Progress States

- NOT_STARTED
- IN_PROGRESS 
- BLOCKED (with reason)
- COMPLETE
- VERIFIED