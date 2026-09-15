# Quota configuration

## Current state

Rate limiting today is a single flat quota, `RATE_LIMIT_PER_MINUTE`
(default 60/min), applied per API key (or per IP for unauthenticated
attempts) uniformly across every tenant — see `docs/auth-rate-limiting.md`
for the mechanism. There is no per-tenant override and no persisted quota
config; changing a tenant's limit today means changing the global env var
and redeploying.

`CLAUDE.md.pulsar-core` flags a **dead admin quota write path** as part of
the cross-tenant leak history: an endpoint existed to write quota config that
nothing downstream ever read. This repo has no such path — there is no quota
write endpoint at all yet, precisely to avoid re-introducing an unwired,
untested one. Build the read path and the enforcement path together with any
future write path, in the same PR, per the "no dead code paths" rule in
`docs/security-design.md`.