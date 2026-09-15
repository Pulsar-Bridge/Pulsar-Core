#!/usr/bin/env bash
# Runs once, automatically, via Postgres's docker-entrypoint-initdb.d hook —
# and ONLY then — as the image's bootstrap superuser. This is the one place
# in the whole system a superuser is allowed to exist; nothing under src/ or
# any .env* file may ever connect as it. See docs/security-design.md.
set -euo pipefail

: "${PULSAR_APP_PASSWORD:?PULSAR_APP_PASSWORD must be set}"

psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    DO \$\$
    BEGIN
        IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'pulsar_app') THEN
            CREATE ROLE pulsar_app LOGIN PASSWORD '${PULSAR_APP_PASSWORD}'
                NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS;
        END IF;
    END
    \$\$;

    ALTER DATABASE "$POSTGRES_DB" OWNER TO pulsar_app;
    GRANT ALL PRIVILEGES ON DATABASE "$POSTGRES_DB" TO pulsar_app;
EOSQL

echo "pulsar_app role ready (NOSUPERUSER, NOBYPASSRLS) and owns ${POSTGRES_DB}"
