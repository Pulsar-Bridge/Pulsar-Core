# Quota configuration

## Current state

Rate limiting today is a single flat quota, `RATE_LIMIT_PER_MINUTE`
(default 60/min), applied per API key (or per IP for unauthenticated
attempts) uniformly across every tenant — see `docs/auth-rate-limiting.md`
for the mechanism. There is no per-tenant override and no persisted quota
config; changing a tenant's limit today means changing the global env var
and redeploying.