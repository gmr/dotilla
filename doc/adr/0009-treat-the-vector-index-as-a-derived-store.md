# 9. Treat the vector index as a derived store

Date: 2026-08-04

## Status

Accepted

## Context

ADR 6 introduced LanceDB alongside fjall. ADR 8 placed them in sibling
directories under one dotilla database. Both leave the same problem unsolved.

fjall and LanceDB are independent storage engines with independent durability.
There is no transaction spanning both. Writing a node means writing to fjall and
writing a vector to Lance, and nothing makes those two writes atomic. A crash
between them leaves the vector index disagreeing with the graph, with nothing in
either store recording that it happened.

Doing the embedding inline on the write path does not fix this and makes it
worse. It adds CPU bound inference (ADR 7) to request latency, and it still has
a window between the two commits.

The question is not how to make the two writes atomic, because that is not
available. It is which store is allowed to be wrong, and how it gets fixed.

## Decision

fjall is the source of truth. The LanceDB dataset is a derived index: it
contains nothing that cannot be reconstructed by reading fjall, and it is never
read to answer a question about what the graph contains, only to answer which
nodes are similar.

Propagate writes with a transactional outbox. A write commits the graph change
and an outbox entry in a single `fjall::Database::batch`, which is atomic across
keyspaces within one database. A background worker per database drains the
`outbox` keyspace, generates the embedding, writes it to Lance, and only then
deletes the outbox entry.

Record the embedding model identity and vector dimension alongside the dataset,
and refuse to use a Lance dataset whose recorded model does not match the
running configuration.

## Consequences

The failure mode becomes staleness rather than corruption. A crash mid flight
leaves the outbox entry in place and the work replays on restart. The vector
index may lag the graph; it cannot silently diverge from it.

Replay requires embedding writes to be idempotent by node identity, which is a
constraint on how vectors are keyed in Lance: upsert by node id, never append.

If the Lance dataset is lost, corrupted, or invalidated by a model change, the
recovery is to delete `{database}/lancedb/` and rebuild by scanning the `nodes`
keyspace. This is only possible because no embedding exists solely in Lance, and
it is the main reason for insisting on the derived relationship. It also makes
changing the embedding model an operational procedure rather than a data loss
event.

Embedding moves off the request path. Writes return as soon as the graph change
and the outbox entry are committed, so write latency does not include inference,
and CPU bound embedding runs on its own worker rather than on a tokio runtime
thread.

Similarity queries are eventually consistent. A node written and immediately
searched for may not yet be in the index. This has to be stated in user facing
documentation rather than left to be discovered. If a strong read is ever
needed, the outbox depth is the signal for how far behind the index is, and it
is worth exposing.

The outbox is unbounded if the worker cannot keep up or is failing. It needs a
depth metric, and a decision about what happens when it grows without limit:
most likely apply back pressure to writes rather than accumulate indefinitely.
Poison entries, where a particular node's embedding fails repeatedly, need a
retry limit and somewhere to go so that one bad entry cannot stall the queue.

The outbox is a fourth keyspace in each database's `fjall::Database`, and the
worker is one more task per open database. This adds to the per database
resource cost that ADR 8 requires to be budgeted.
