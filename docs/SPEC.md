# Project Specification

> Status: Draft | Updated: [DATE]

## Summary

[2-3 sentences: what this project does and why]

## Problem & Success Criteria

**Current state:** [What problems exist?]
**Desired state:** [What does success look like?]

- [ ] [Measurable outcome 1]
- [ ] [Measurable outcome 2]

## Scope

**In:** [Feature 1], [Feature 2], [Feature 3]
**Out:** [Excluded item 1], [Excluded item 2]

## Requirements

### FR1: [Requirement Name]
**Priority:** High | Medium | Low
- [ ] [Acceptance criterion 1]
- [ ] [Acceptance criterion 2]

### FR2: [Requirement Name]
**Priority:** High | Medium | Low
- [ ] [Acceptance criterion 1]
- [ ] [Acceptance criterion 2]

### Non-Functional
- **Performance:** [Response time, throughput targets]
- **Security:** [Auth, data protection requirements]
- **Reliability:** [Uptime, recovery targets]

## Technical Architecture

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Frontend | [tech] | [why] |
| Backend | [tech] | [why] |
| Database | [tech] | [why] |

### Key Components

**[Component 1]:** [Purpose, interfaces, dependencies]
**[Component 2]:** [Purpose, interfaces, dependencies]

## Data Model

| Entity | Key Fields | Relationships |
|--------|-----------|---------------|
| [Entity 1] | id, name, created_at | has many [Entity 2] |
| [Entity 2] | id, entity1_id, ... | belongs to [Entity 1] |

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | /api/resource | List resources |
| POST | /api/resource | Create resource |

## Testing Strategy

- **Unit:** 80% coverage, focus on business logic
- **Integration:** API endpoints, database operations
- **E2E:** Critical user flows

## Open Questions

- [ ] [Question 1]
- [ ] [Question 2]
