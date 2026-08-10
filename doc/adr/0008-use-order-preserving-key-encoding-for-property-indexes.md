# 8. Use order-preserving key encoding for property indexes

Date: 2026-08-04

## Status

Accepted

## Context

ADR 10 stores an entity's properties as a single AMQP field table keyed by
entity identifier. That layout answers one question well: give me this entity's
properties. It answers nothing else. `MATCH (n:Person) WHERE n.age > 30` under
that layout alone is a scan of every `Person` with a full table decode per node,
discarding almost all of it.

In an LSM tree a range predicate is naturally a seek followed by forward
iteration, but only if the byte order of the keys matches the semantic order of
the values they encode. AMQP field table encoding has no such property: the tag
precedes the payload, numeric payloads are not arranged for byte comparison, and
nothing relates the encoded bytes to the value's position in an ordering.

So a second encoding is required, with goals opposite to ADR 10's. That encoding
optimises for fidelity and compactness and must round-trip. This one optimises
for byte-lexicographic ordering and does not need to be reversible at all, since
the authoritative value is always available in the entity row.

String ordering raises a further question. Byte ordering of UTF-8 matches code
point order, which puts `"Z"` before `"a"` and sorts accented characters far
from their base letters. That is not what a user means by sorted, and locale
correct ordering is a property worth having rather than a limitation to
document.

`icu_collator` (ICU4X) provides `write_sort_key_to`, which emits bytes with
exactly the required guarantee: bytewise comparison of two sort keys generated
at the same strength gives the same result as a collation comparison of the
original strings. It is pure Rust, so ADR 3's argument against a C++ toolchain
survives.

ICU4X warns that durably stored sort keys must be presumed invalidated by a CLDR
update, a new Unicode version, or an ICU4X code change, and states explicitly
that it will not support pinning an older sort key algorithm against a newer
library. This is the same hazard that has repeatedly caught Postgres
installations through glibc collation changes.

## Decision

Add an `index` keyspace to each database's `fjall::Database`. Entries are

```
{label}\0{property}\0{type_rank}{ordered_value}{entity_id}  ->  (empty)
```

with the value empty because everything needed is in the key.

Encode ordered values as:

- Boolean: `0x00` or `0x01`.
- Integer: `((v as u64) ^ (1 << 63)).to_be_bytes()`.
- Float: normalise `-0.0` to `0.0`, exclude NaN from the index, then flip the
  sign bit for positives and all bits for negatives, big-endian.
- String: the collation sort key, or raw UTF-8 under binary collation.
- Bytes: the raw bytes.

Escape `0x00` as `0x00 0xFF` in variable length encodings and terminate with
`0x00 0x00`, so the entity identifier suffix is unambiguously separable.

Do not index nulls. ADR 10 does not store them, so an index entry exists for
exactly those entities that have the property. `IS NOT NULL` is therefore a
prefix scan of `{label}\0{property}\0`, and `IS NULL` is a label scan minus
index membership.

Give integers and floats distinct type ranks. A numeric range predicate seeks
both ranges and merges the results. The comparator used for the merge and for
boundary conditions must compare exactly and must not go through `i64 as f64`.

Maintain the index synchronously, in the same `fjall::Database::batch` as the
entity row write. This requires reading the previous row to know which index
entries to delete.

Make collation a per-database attribute, fixed at creation and immutable
thereafter. Record the locale and the strength in the database metadata. Offer a
`binary` collation that bypasses ICU entirely and compares raw UTF-8 bytes.
Default to the root locale `und` at tertiary strength.

Detect drift by fingerprinting behaviour rather than by recording a version.
ICU4X exposes no runtime accessor for its CLDR or collation data version, so
there is no equivalent of Postgres's `datcollversion` to compare against. At
database creation, generate sort keys for a fixed probe corpus using the
database's collator, hash them, and store the digest alongside an identifier for
which corpus produced it. At open, recompute and compare.

When the fingerprint does not match, open the database normally but disable
index-backed string predicates, falling back to label scans, and surface the
condition in logs and in the database metadata response until an explicit
reindex is performed.

## Consequences

Property values are now encoded by two codecs with opposing goals. They must be
named distinctly in the code, because reaching for the wrong one produces an
index that is subtly misordered rather than an error.

Collation sort keys are deliberately lossy. At primary strength ICU4X generates
identical keys for `"hello"` and `"Héłłö"`. The index therefore answers range
predicates exactly but can only produce candidates for equality, which must be
rechecked against the entity row. The index API should return candidates rather
than answers from the outset, since retrofitting that distinction later means
auditing every call site.

Sort keys are larger than the strings they encode, and generating one is
expensive relative to a single comparison. ICU4X says as much and recommends
sort keys only where the cost is amortised over many comparisons, which is
precisely the index case.

The index is a derived store, fully rebuildable from the `nodes` and `edges`
keyspaces. This is the same principle as ADR 9 and for the same underlying
reason: an external algorithm can shift beneath stored bytes. The two differ in
consistency requirement. A stale vector index returns worse similarity results;
a stale property index returns wrong query results, so this one cannot be
deferred to a worker.

Reindexing is therefore a first-class operation with an API, not a maintenance
script. It is needed when the collation fingerprint changes, when creating an
index on a non-empty database, and for recovery.

Choosing scan fallback over automatic rebuild means a routine `cargo update`
that bumps ICU4X degrades performance rather than taking databases offline or
making open time unbounded, which would fight ADR 8's lazy-open registry.
Results stay correct throughout. The cost is that a silently slow server is
easier to ignore than a failing one, so the condition must be loud in logs and
visible in the metadata response.

Fingerprinting behaviour is more precise than a version comparison in both
directions: an ICU4X release that does not alter collation triggers no reindex,
and a data change that arrives without a version bump is still caught. The cost
is coverage. A tailoring change affecting a script absent from the probe corpus
goes undetected, so the corpus needs multiple scripts, combining marks, case
pairs, digits, and variable characters such as punctuation and whitespace, where
tailorings most often differ. The corpus must itself be versioned, because
extending it changes every stored digest for a reason that is not drift.

Defaulting to `und` tertiary means every database pulls ICU4X collation data and
is exposed to version drift, including databases whose properties are all
identifiers where byte order would have been fine and stable. `binary` exists
for those, and choosing it is also the right answer for anyone who wants
guaranteed stability across upgrades.

ICU4X data adds to binary size, on top of LanceDB from ADR 6 and the embedding
model from ADR 7. Collation data can be scoped to selected locales at build
time if this becomes a problem.

`CollationKeySink`, the trait `write_sort_key_to` writes through, is marked
unstable. It is implemented for `Vec<u8>` among others, so ordinary use is fine,
but this is a dependency on an unstable API and worth pinning deliberately.

The `Collator` is `Send + Sync`, so one instance is constructed at database open
from the recorded locale and lives in the per-database `Database` value from
ADR 8, alongside the fjall and LanceDB handles.

Index maintenance makes every property write a read-modify-write. ADR 10 already
requires this for whole-row encoding, so the marginal cost is the index entry
computation rather than the read.

Lists are not indexed initially. Indexing them properly means one index entry
per element, which changes the maintenance logic and the cardinality
assumptions, and it should be a separate decision.

None of this pays off until the query planner can recognise that a predicate is
seekable and choose an index over a label scan. The storage is the easy half,
and building it first without the planner produces a correct index that nothing
uses.
