# Cross-tenant leak: design note

`CLAUDE.md.pulsar-core` names this file as required reading before touching
any query or connection-string code, in the same breath as describing a
cross-tenant data leak from this codebase's design history (unscoped
`/transactions*`, `/settlements*`, and `/ws` endpoints, caused by every
environment connecting as the Postgres bootstrap superuser, which bypasses
Row-Level Security regardless of how correct the policies are).