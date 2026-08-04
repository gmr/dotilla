# 8. Store each database in its own directory

Date: 2026-08-04

## Status

Accepted

## Context

dotilla exposes databases as a top level construct, in the sense that Postgres
or ClickHouse do: `PUT /{db}` creates one, and every query names one. Inside a
database, the graph needs physically separate structures for nodes, edges, and
labels.

fjall provides exactly two levels of nesting. A `fjall::Database` is a
directory, a journal, a write buffer, a block cache, and a pool of background
threads. A `fjall::Keyspace` is a column family inside it, with its own LSM
tree. There is no third level. So a dotilla database with three internal
structures maps onto fjall in one of two ways:

1. One shared `fjall::Database` for the whole server, with keyspaces named
   `{db}_nodes`, `{db}_edges`, `{db}_labels`.
2. One `fjall::Database` per dotilla database, each containing keyspaces named
   `nodes`, `edges`, `labels`.

Option 1 is cheaper. `fjall::Database` resources are per instance and cannot be
shared between instances: `worker_threads` defaults to `min(cores, 4)`,
`max_journaling_size` defaults to 512 MiB, and `cache_size` and
`max_cached_files` are likewise per instance. fjall 3 provides no shared cache
API. Under option 2 all of these multiply by the number of open databases, and
the file descriptor cache is the limit that binds first.

Three things decided it for option 2.

The operations that belong to a database live on `fjall::Database`, not on
`fjall::Keyspace`: `persist`, `disk_space`, `snapshot`, `journal_count`, and
`batch`. Under option 1 all of these operate on the whole server, so
`disk_space` for one database, or fsyncing one database, would have to be
reimplemented by hand.

ADR 6 adds LanceDB as a second engine holding per database data. LanceDB owns
its own directory. Under option 1 a single dotilla database would exist in two
incompatible layouts: its graph data mixed into a shared fjall root, its vectors
in a per database directory. Dropping, backing up, moving, or measuring a
database would be a different operation for each half.

Under option 2 the directory is the unit. Create, drop, back up, move, and
measure are all directory operations, identical across both engines.

## Decision

Lay out the data directory as:

```
{data_dir}/
  {database}/
    graph/      one fjall::Database, keyspaces: nodes, edges, labels, outbox
    vectors/    one LanceDB dataset
```

Open `fjall::Database` and the LanceDB connection together as a single
`Database` value, held in a registry in `AppState` and keyed by database name.

Open lazily on first use, with single flight, and never evict.

Set `worker_threads`, `cache_size`, `max_journaling_size`, and
`max_cached_files` explicitly from a server wide budget in the TOML config
rather than relying on fjall's defaults, since those defaults assume a single
`Database` per process.

Reject requests to open a database beyond a configured maximum with `503`
rather than closing an existing one to make room.

## Consequences

A dotilla database is a directory. `DROP DATABASE` is remove from the registry,
drop the handles, remove the directory. Backup is copying a directory. Disk
usage is the size of a directory. Per database `persist` and `snapshot` come
from fjall directly.

Cross keyspace atomic batches are scoped to exactly one dotilla database, which
is the correct boundary and is what makes the outbox in ADR 9 work.

Opening is expensive: journal recovery, thread spawn, buffer allocation, plus
whatever LanceDB does. It is blocking work and must run on
`tokio::task::spawn_blocking`, never on a runtime worker thread.

Two concurrent requests for a database that is not yet open must not both open
it. The registry holds `Arc<tokio::sync::OnceCell<Database>>` per name so that
the second request waits on the first. `get_or_try_init` leaves the cell empty
on failure, so a failed open retries rather than poisoning the entry. If the
registry is a `DashMap`, its guard must be dropped before any await point.

Databases are never evicted. Dropping the last handle closes the database, and
a TTL or LRU policy would turn an idle database into a latency spike on next
use, or worse, close a database with a query in flight. Bounding the count and
rejecting at the limit is the safe failure mode. The consequence is that a
server with many databases must be sized for them, and the maximum is an
operational parameter rather than something the server hides.

The resource budget is now hand managed. This is the real cost of the decision:
the shared design would have provided one global budget for free, and instead
the per database budget has to be derived from a server total and kept correct
as the code grows.

Database names become directory names, which imposes two constraints.

`dotilla_system` and any other server level directory name must be reserved and
rejected by `DatabaseName`, so that the type carries the guarantee rather than the
creation path checking for it.

macOS filesystems are case insensitive by default, so `Foo` and `foo` are
distinct entries in a tracked list but the same directory on disk. Database
names are therefore normalised to lowercase at parse time, so that the stored
name and the created directory are the same string on every platform.
