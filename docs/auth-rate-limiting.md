# Auth rate limiting

`admin_auth` and `api_key_auth` (`src/middleware/auth.rs`) are on the request
path before any handler runs, so they're the trust boundary a
credential-stuffing or key-guessing run would hit first. Both are rate
limited the same way:

1. The presented bearer token (or, if none was presented, the client IP from
   `ConnectInfo`) is checked against a keyed `governor` rate limiter
   (`AppState::rate_limiter`, quota from `RATE_LIMIT_PER_MINUTE`,
   default 60/min) **before** the token is validated. This means the limiter
   throttles guessing attempts themselves, not just successful callers.
2. On any auth failure — missing token, unknown token, rate limit exceeded —
   `crate::metrics::AUTH_FAILURES` is incremented and a `tracing::warn!` is
   emitted with only a key prefix (first 8 chars), never the full credential,
   so failures are alertable without logging secrets.
3. `admin_auth` compares candidate keys with a constant-time comparison
   (`constant_time_eq`) to avoid leaking key content via early-exit string
   comparison timing.

## What this does not yet do

- Rate limiting is in-process (per replica), not shared across replicas via
  Redis. Under `docker-compose.load.yml`'s 3 replicas, the effective limit is
  roughly `3 * RATE_LIMIT_PER_MINUTE`. If that's not acceptable, move the
  limiter state into Redis (`governor` supports a custom clock/store, or use
  a Lua-scripted token bucket directly).
- API keys live in `TENANT_API_KEYS`/`ADMIN_API_KEYS` env vars, not a
  database table, so rotation requires a redeploy. See
  `docs/quota-configuration.md` for the natural next step (a `tenants`/
  `api_keys` table with per-tenant quotas) if that becomes a blocker.
