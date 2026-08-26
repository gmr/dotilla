# 3. Use fjall for storage layer

Date: 2026-07-31

## Status

Accepted

## Context

dotilla is a single binary with no external dependencies, so storage has to be
embedded rather than a separate server process.

Graph traversal is dominated by adjacency lookups: given a node, find its edges.
That is a prefix scan over sorted keys, so the store must maintain key order. A
hash-based key value store cannot serve this access pattern.

Writes are frequent and mostly appends, which suits an LSM tree better than a
B-tree.

The candidates were RocksDB, which is the reference implementation but drags in
a C++ toolchain and a long build; sled, whose most recent release is 0.34.7 from
2020 and which still describes its on-disk format as subject to breaking change
before 1.0; redb, which is pure Rust but a copy-on-write B-tree with no column
family concept; and fjall, a pure Rust LSM tree with keyspaces.

## Decision

Use `fjall` (currently 3.x) as the embedded storage engine.

## Consequences

Pure Rust, so `cargo build` needs no C++ compiler and cross compilation stays
simple.

Ordered keys give prefix and range scans, which is what adjacency traversal
needs. Keyspaces provide column family separation, so nodes, edges, and labels
can be physically distinct without separate database handles, and
`Database::batch` makes writes across them atomic.

fjall is a young project by a small number of maintainers, and the API is not
stable. The 2.x to 3.x transition renamed the core types: what was `Keyspace`
became `Database`, and what was `Partition` became `Keyspace`. That kind of
churn should be expected again. Wrapping fjall behind our own storage types,
rather than passing `fjall::Keyspace` around the codebase, keeps the blast
radius of the next rename small.

Keys are limited to 65536 bytes and values to 2^32 bytes. Node and edge
identifiers are far below this, but a property value could exceed it, so
oversized values need either rejection or key value separation.

fjall is a storage engine, not a database. There are no secondary indexes, no
query planner, no joins, and no notion of a schema. Every one of those is ours
to build on top. This is the intended trade: it does the durable ordered bytes
part well and stays out of the way of the graph model.

Resource management is per `Database` and is not shared between them. A server
holding many databases open therefore multiplies block caches, write buffers,
and file descriptors rather than drawing them from a common budget, so the
registry that owns those handles has to bound how many stay open.
