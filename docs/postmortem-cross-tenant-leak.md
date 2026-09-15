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

- **Root cause and the fix for it**: `docs/security-design.md` — read that
  file, not this one, for the actual model (least-privilege role, `FORCE
  ROW LEVEL SECURITY`, per-transaction tenant scoping, and a startup check
  that refuses to run against a superuser/bypass role).
- **Rate limiting on the trust boundaries involved**: `docs/auth-rate-limiting.md`.
- **The dead unauthenticated `/reconnect` store** mentioned alongside the
  leak: this repo has no such path. `src/main.rs::build_router` is the only
  place routes are wired, and everything in it is reachable and covered by
  `admin_auth` or `api_key_auth`. Keep it that way — see the "no dead code
  paths" rule in `docs/security-design.md`.