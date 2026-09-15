# Contributing to pulsar-core

## PR checklist

Before opening a PR, all of the following must be clean:

```
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets
cargo test
```

Additionally, for any PR that touches a DB connection, an auth path, or a new
endpoint:

- [ ] Every DB/service connection this PR adds or touches runs as a role with
      the minimum privilege the code path needs — never a superuser or
      RLS-bypassing role. See `docs/security-design.md`.
- [ ] Every new endpoint has an idempotency story (or an explicit note on why
      it doesn't need one) before merge, not after.
- [ ] `admin_auth`/`api_key_auth`/the `relay_signer` trust boundary stays rate
      limited, with failures logged and counted (`crate::metrics::AUTH_FAILURES`).
- [ ] No unauthenticated or session-based code path is left wired into the
      request graph without being reachable and tested, and no such path is
      left *unwired but present* either — delete dead code, don't strand it.
- [ ] If this PR changes what the service sends to `pulsar-core-contracts` or
      expects back, the sibling repo's `EVENTS.md`/ABI was actually read
      first — see the top of `CLAUDE.md.pulsar-core`.
- [ ] If this PR closes a README "Under Development"/"Planned" item, the
      README is updated in the same PR.

## Local setup

```
cp .env.example .env.development   # already checked in; edit if you need different local ports
docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d postgres redis
cargo run
```

Migrations run automatically on startup via `sqlx::migrate!`. To add a new
one: