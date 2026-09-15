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

ALTER TABLE transactions ENABLE ROW LEVEL SECURITY;
ALTER TABLE transactions FORCE ROW LEVEL SECURITY;

CREATE POLICY tenant_isolation ON transactions
    USING (tenant_id = current_setting('app.tenant_id', true)::uuid)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::uuid);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    NEW.updated_at := now();
    RETURN NEW;
END;
$$;

CREATE TRIGGER transactions_set_updated_at
    BEFORE UPDATE ON transactions
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- Partition maintenance -------------------------------------------------
--
-- Called by the background job in src/db/partitions.rs every
-- PARTITION_MAINTENANCE_INTERVAL_SECONDS (default 24h): ensures the current
-- and next month's partitions exist, and drops partitions older than
-- PARTITION_RETENTION_MONTHS (default 12).

CREATE OR REPLACE FUNCTION ensure_transactions_partition(p_month date)
RETURNS void LANGUAGE plpgsql AS $$
DECLARE
    partition_name text := 'transactions_' || to_char(p_month, 'YYYY_MM');
    start_date date := date_trunc('month', p_month);
    end_date date := start_date + interval '1 month';
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_class WHERE relname = partition_name) THEN
        EXECUTE format(
            'CREATE TABLE %I PARTITION OF transactions FOR VALUES FROM (%L) TO (%L)',
            partition_name, start_date, end_date
        );
    END IF;
END;
$$;

CREATE OR REPLACE FUNCTION drop_transactions_partitions_older_than(p_cutoff date)
RETURNS TABLE(dropped_partition text) LANGUAGE plpgsql AS $$
DECLARE
    rec record;
BEGIN
    FOR rec IN
        SELECT c.relname
        FROM pg_inherits i
        JOIN pg_class c ON c.oid = i.inhrelid
        JOIN pg_class p ON p.oid = i.inhparent
        WHERE p.relname = 'transactions'
          AND c.relname ~ '^transactions_\d{4}_\d{2}$'
          AND to_date(substring(c.relname from 'transactions_(\d{4}_\d{2})$'), 'YYYY_MM') < p_cutoff
    LOOP
        EXECUTE format('DROP TABLE IF EXISTS %I', rec.relname);
        dropped_partition := rec.relname;
        RETURN NEXT;
    END LOOP;
END;
$$;