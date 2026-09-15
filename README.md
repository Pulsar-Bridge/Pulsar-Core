# Pulsar-Core

Rust off-chain relay for Pulsar Bridge — ingests Stellar Anchor Platform
webhooks, deduplicates and persists fiat deposit events, and mirrors them
on-chain via the Pulsar contract.

Read `CLAUDE.md.pulsar-core` before making changes — it's this repo's
standing engineering brief (sibling repos, security history, ownership).

## Status

- `POST /webhook`: **implemented.** Payload validation, Redis-backed
  idempotency (`X-Idempotency-Key`, 24h TTL, concurrent duplicates get 429),
  tenant-isolated Postgres persistence, and a call into
  `pulsar-core-contracts`' `register_callback()` guarded by a circuit
  breaker.
- Contract integration: **placeholder shape.** `pulsar-core-contracts` isn't
  cloned into this checkout, so the request/response shape in
  `src/contracts/mod.rs` is unverified — see `docs/contract-integration.md`
  before relying on it.
- `admin_auth` / `api_key_auth`: **implemented**, both rate limited on every
  attempt (success or failure) — see `docs/auth-rate-limiting.md`.
- DB roles: **least-privilege only.** No connection path in this repo may
  ever use a superuser or RLS-bypassing role — see `docs/security-design.md`.
- `src/error.rs`: **implemented** — typed `AppError`, not ad hoc
  `anyhow`/string errors at the boundary.
- Per-tenant quota configuration: **not implemented yet** — flat
  `RATE_LIMIT_PER_MINUTE` only. See `docs/quota-configuration.md` for the
  planned next step.

## Quick start

```
cp .env.development .env   # dev defaults; see .env.example for the full var list
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d postgres redis
cargo run
```