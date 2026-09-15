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

This is not a fabricated incident with a timeline or impact numbers — it's
the specific failure class this fresh implementation is built to make
structurally impossible, not just discouraged by convention.

## The model this repo uses instead

1. **One application role, never a superuser.** `scripts/db/init-roles.sh`
   creates `pulsar_app` with `NOSUPERUSER NOCREATEDB NOCREATEROLE
   NOREPLICATION NOBYPASSRLS`, owning the database. It is the *only* role any
   `.env*` file or connection string in this repo may ever reference. The
   bootstrap superuser is used exactly once, automatically, by Postgres's own
   `docker-entrypoint-initdb.d` hook — never by application code.

2. **`FORCE ROW LEVEL SECURITY`, not just `ENABLE`.** `migrations/0001_create_transactions.sql`
   forces RLS even for the owning role. `ENABLE ROW LEVEL SECURITY` alone
   still lets the table owner bypass policies; `FORCE` closes that gap, so
   owning the table isn't itself a bypass path.

3. **Tenant scope is set per-transaction, not trusted from the query.**
   `db::begin_tenant_scoped` is the only way application code opens a
   transaction against tenant-owned tables. It runs
   `SELECT set_config('app.tenant_id', $1, true)` before anything else, and
   the `tenant_isolation` policy on `transactions` filters every statement in
   that transaction against it. A handler cannot accidentally read another
   tenant's rows by forgetting a `WHERE tenant_id = ...` clause, because
   there is no code path that queries `transactions` without first setting
   that scope.