#!/usr/bin/env bash
set -euo pipefail

HOST="${1:-localhost}"
PORT="${2:-6465}"
DB="${3:-test}"

QUERY='MATCH (f:Foo)-[b:BAR]->(z:Baz) RETURN f, b, z'

curl -X POST \
    -H "Content-Type: text/plain" \
    --data "$QUERY" \
    --verbose \
    "http://${HOST}:${PORT}/${DB}" | jq .
