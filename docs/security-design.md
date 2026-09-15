# Security design

This document is the source of truth for the DB-role and tenant-isolation
model in pulsar-core. Read it before writing any query, connection string, or
new endpoint.

## The failure mode this design exists to prevent

`CLAUDE.md.pulsar-core` (this repo's standing engineering brief) describes a
serious bug from this codebase's design history: every environment file
connected to Postgres as the `initdb` bootstrap superuser. A superuser (or
any role with `BYPASSRLS`) ignores Row-Level Security policies entirely and
silently — so `/transactions*`, `/settlements*`, and `/ws` resync endpoints
returned unscoped cross-tenant data even though RLS policies existed and were
individually correct. A dead admin quota write path and an unauthenticated,
unwired `/reconnect` session store existed alongside it.