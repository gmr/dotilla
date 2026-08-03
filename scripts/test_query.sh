#!/usr/bin/env bash
set -euo pipefail

HOST="${1:-localhost}"
PORT="${2:-6465}"
DB="${3:-test}"

QUERY='MATCH (f:Foo)-[b:BAR]->(z:Baz) RETURN f, b, z'

curl --fail -sS \
    -X POST \
    --data "$QUERY" \
    "http://${HOST}:${PORT}/${DB}"
