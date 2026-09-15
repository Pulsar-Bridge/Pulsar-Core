-- Time-partitioned, tenant-isolated deposit transactions table.
--
-- RLS is FORCE-d so that even the owning application role (pulsar_app) cannot
-- see rows outside the tenant set via `SET LOCAL app.tenant_id`. There is
-- intentionally no bypass role or admin superuser path here — see
-- docs/security-design.md for why.

CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE transactions (
    id                     uuid        NOT NULL DEFAULT gen_random_uuid(),
    tenant_id              uuid        NOT NULL,
    idempotency_key        text        NOT NULL,
    external_deposit_id    text,
    status                 text        NOT NULL DEFAULT 'pending'
                             CHECK (status IN ('pending', 'submitted', 'confirmed', 'failed')),
    amount                 numeric(20, 7) NOT NULL CHECK (amount > 0),
    asset_code             text        NOT NULL,
    stellar_account        text        NOT NULL,
    anchor_platform_payload jsonb      NOT NULL,
    contract_tx_hash       text,
    created_at             timestamptz NOT NULL DEFAULT now(),
    updated_at             timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (id, created_at)
) PARTITION BY RANGE (created_at);

-- Idempotency is authoritative in Redis (24h NX lock, see src/redis/idempotency.rs);
-- this index is a defense-in-depth lookup aid within a partition, not a hard
-- uniqueness guarantee across partition boundaries.
CREATE INDEX idx_transactions_tenant_idempotency
    ON transactions (tenant_id, idempotency_key);

CREATE INDEX idx_transactions_tenant_status
    ON transactions (tenant_id, status);