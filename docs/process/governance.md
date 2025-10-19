# Governance Process

## Monthly Governance Review
- **Cadence:** Conducted on the first business day of every month.
- **Scope:** Review and audit all repository `AGENTS.md` files, security documentation (including `docs/security/` contents), process documentation within `docs/process/`, design references in `docs/design/`, and the active test expectations in `docs/testing/test-matrix.md`.
- **Objectives:** Ensure guidance reflects current workflows, verify security requirements (encryption, sandboxing, validation) remain accurate, confirm process artifacts align with operational practices, and validate the test matrix continues to drive pre-implementation failing tests.

## Roles and Responsibilities
- **Governance Lead:** Schedules the review, chairs the meeting, and ensures completion of agenda items.
- **Security Officer:** Evaluates security documentation updates, validates mitigations, and signs off on security-related changes.
- **Process Steward:** Confirms process documents remain accurate, proposes revisions, tracks pending updates, and ensures design/test-matrix changes are communicated before coding work begins.
- **Documentation Scribe:** Captures decisions, action items, and change ownership during each review session.
- **Sign-off Requirement:** Governance Lead, Security Officer, and Process Steward must each provide explicit approval before changes are merged.

## Change Tracking and Action Log
- **Review Record:** Maintain a dated entry for each review, summarizing findings across scope areas (including design diagrams, encryption coverage, and client deliverable expectations) and referencing updated documents.
- **Decisions:** Document approved changes, responsible owners, and expected completion dates.
- **Action Items:** Track open tasks with assigned owners and follow-up dates; revisit outstanding actions at each subsequent review until closed.
- **Repository Location:** Store review records in `docs/process/governance-log.md` (one section per month) to preserve accountability.
