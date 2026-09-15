# Cross-tenant leak: design note

`CLAUDE.md.pulsar-core` names this file as required reading before touching
any query or connection-string code, in the same breath as describing a
cross-tenant data leak from this codebase's design history (unscoped
`/transactions*`, `/settlements*`, and `/ws` endpoints, caused by every
environment connecting as the Postgres bootstrap superuser, which bypasses
Row-Level Security regardless of how correct the policies are).

This implementation was bootstrapped fresh, so there is no incident timeline,
affected-user count, or detection/remediation log to report here — writing
one would be fabricating history that didn't happen in this checkout. What
this file *can* do honestly is point at where that failure mode is addressed
structurally: