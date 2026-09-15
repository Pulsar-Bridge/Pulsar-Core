#!/usr/bin/env bash
# Runs once, automatically, via Postgres's docker-entrypoint-initdb.d hook —
# and ONLY then — as the image's bootstrap superuser. This is the one place
# in the whole system a superuser is allowed to exist; nothing under src/ or
# any .env* file may ever connect as it. See docs/security-design.md.
set -euo pipefail