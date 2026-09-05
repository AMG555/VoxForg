# ADR-002: Tri-Tier Persistence Architecture via DataStore Trait

## Status
Accepted

## Date
2026-09-05

## Context
VoxForg targets three distinct deployment profiles:
1. **Desktop & Single-Binary**: Standalone local user running on a laptop with zero database setup.
2. **Self-Hosted Docker**: Developer or company running a private instance with multiple concurrent users and persistent job history.
3. **Cloud Multi-Tenant**: Managed cloud instance backed by Supabase with Row-Level Security (RLS).

Hard-coding a single database driver (e.g. Postgres-only or SQLite-only) alienates two of the three targets.

## Decision
Abstract persistence behind a unified asynchronous Rust trait: `DataStore`. Implement three decoupled backends:
1. `SqliteStore`: Default for single binary and desktop (`voxforg.db` in WAL mode with embedded migrations).
2. `PostgresStore`: Selected when `DATABASE_URL=postgres://...` is set.
3. `SupabaseStore`: Supports cloud synchronization and Supabase Auth JWT verification.

## Alternatives Considered

### Direct ORM (e.g., SeaORM or Diesel)
- **Pros:** High-level query building.
- **Cons:** Heavy macro expansion overhead; cross-compilation across SQLite and PostgreSQL dynamic drivers often causes C-dependency linking friction in desktop distributions.
- **Rejected:** Prefer clean async trait with explicit SQLx queries and minimal runtime overhead.

## Consequences
- Single binary runs out of the box with zero external dependencies.
- Cloud and Docker users can scale horizontally without architectural refactoring.
- Storage schema must adhere to common SQL subset compatible with both SQLite and PostgreSQL.
