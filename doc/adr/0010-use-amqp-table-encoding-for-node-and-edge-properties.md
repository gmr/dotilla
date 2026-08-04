# 10. Use AMQP table encoding for node and edge properties

Date: 2026-08-04

## Status

Accepted

## Context

Nodes and edges carry properties. fjall stores bytes, so properties need an
encoding, and it has to be self-describing: a value read back must know whether
it is an integer, a string, or a boolean without consulting a schema, because
dotilla has no schema.

The candidates were CBOR (RFC 8949), MessagePack, a Rust-native format such as
`postcard` or `bincode`, and the AMQP 0-9-1 field table type system.

The Rust-native formats are the most compact but are not self-describing without
carrying a schema alongside, which reintroduces the thing a schemaless graph
database is trying to avoid.

CBOR and MessagePack are both good fits and are close to equivalent for this
purpose. CBOR has one genuine advantage, discussed under canonical form below.

AMQP field tables are self-describing, typed, compact, and long settled. The
usual objection to them is interoperability: the 0-9-1 specification, the
RabbitMQ errata, and Qpid disagree on several type tags, most notoriously `s`,
which is a short string in the original specification and a signed 16-bit
integer in RabbitMQ's. That objection does not apply here. dotilla is both the
producer and the consumer of these bytes, the encoding is internal, and there is
no third party to interoperate with. The variant simply has to be chosen and
written down.

Against that, the maintainer has deep familiarity with the format from having
implemented it before. On a project where sustained attention is the scarce
resource, building on something already understood is a real advantage and not
merely a sentimental one.

A field table is a map encoding, so the unit it naturally encodes is an entire
property map. Storing one property per key would use only the tagged value half
of the format and never the table half, which would remove most of the reason
for choosing it.

## Decision

Encode the complete property map of a node or edge as a single AMQP 0-9-1 field
table, following the RabbitMQ variant where implementations differ. Implement
the codec in Rust directly; no existing AMQP crate is taken as a dependency.

Store one row per entity. The key in the `nodes` and `edges` keyspaces is the
entity identifier, and the value is the encoded table. Reading an entity is a
single point lookup.

Represent the map in memory as `BTreeMap<PropertyName, Value>` behind a
`Properties` newtype, where `PropertyName` is a validated newtype over
`Box<str>` enforcing non-empty and at most 255 bytes, the AMQP short-string
limit.

Write only a canonical subset of the type tags, chosen to match Cypher's type
system:

| Cypher type | Tag                 |
| ----------- | ------------------- |
| Boolean     | `t`                 |
| Integer     | `l` (signed 64-bit) |
| Float       | `d` (64-bit)        |
| String      | `S`                 |
| Bytes       | `x`                 |
| List        | `A`                 |

Accept the wider tag set on decode, including the narrow integer types `b`, `B`,
`s`, `u`, `I`, and `i`, widening them to `l` on read. Never emit them.

Emit table keys in lexicographic order. This is dotilla's canonical form and is
a deliberate deviation from AMQP, which specifies no deterministic encoding.

Do not store nulls. In Cypher a property set to null and a property that was
never set are the same thing, so `SET n.x = null` removes the entry from the
map. `Properties` at rest therefore never contains a null, and the `V` void tag
is reachable only as a list element.

## Consequences

Reading an entity is one point lookup, which in an LSM tree can be short
circuited by bloom filters at every level. Had properties been stored one per
key, the same read would be a prefix scan, and a range iterator cannot be
filtered the same way: it must set up a merge across the memtable and every
level that might contain matching keys. `RETURN n` is the common case, so
privileging it is the right trade.

Storing one row per entity also avoids repeating the property name in every key,
and avoids multiplying LSM entry overhead, tombstone potential, bloom filter
capacity, and compaction work by the number of properties.

`BTreeMap` iteration order is the canonical form, so encoding is deterministic
by construction rather than by remembering to sort. There is no code path that
can produce non-canonical bytes. `HashMap` would require a sort before every
encode, and the failure mode of forgetting is not a crash but ADR 9 detecting a
spurious change and re-embedding on every write. Property maps are small enough
that `BTreeMap` also outperforms `HashMap` in practice, since hashing a string
costs more than a few comparisons against short keys.

Because the stored bytes are canonical, ADR 9's change detection is a byte
comparison between the newly encoded table and the stored one. No decode, no
sort, no reassembly.

Normalising integers to `l` on write means two equal values always produce equal
bytes. Had the encoder chosen the narrowest tag that fits, `5` stored as `b` and
`5` stored as `l` would compare unequal on encoded form, which would break that
byte comparison and any index lookup done on encoded values. Accepting the
narrow tags on decode costs one match arm each and keeps the door open to
reading data written by anything else.

`SET n.x = 1` is a read, decode, mutate, encode, write of the whole row rather
than a single key write. The read is a cheap point lookup, and it is needed
regardless: ADR 11's index maintenance requires the previous values in order to
delete the index entries they produced.

Projecting a single property from a scan of many entities decodes every property
of every entity, because a field table has no offset table and field access
within it is linear. At realistic property counts this is a constant factor
measured in nanoseconds, not a structural problem. The correct answer to a query
like `MATCH (n:Person) WHERE n.age > 30 RETURN n.name` is a property index, not
a different storage layout. See ADR 11.

Property values are therefore encoded twice by two different codecs: this one
for storage, and ADR 11's order-preserving encoding for index keys. They have
opposite design goals and must not be confused. Naming them distinctly in the
code is worth the effort.

Excluding nulls from storage means the property index in ADR 11 contains an
entry for exactly those entities that have the property, which makes
`IS NOT NULL` a pure prefix scan and `IS NULL` the complement of one.

AMQP's `T` timestamp is a 64-bit POSIX value in seconds, with no sub-second
precision and no timezone, so it is not in the write subset. Temporal values, if
dotilla grows Cypher temporal types, will need either a dedicated representation
or storage as `S`. That decision is deferred rather than made here.

AMQP's `D` decimal and `F` nested table are likewise excluded from the write
subset. `F` in particular can express nested maps, which Cypher's property model
does not have. Accepting them on decode while refusing to emit them keeps stored
data within what the query language can address. The decoder should enforce a
recursion limit regardless, since the wire format can express deeper nesting
than dotilla intends to accept and this is an untrusted input path.

The codec is written by hand in both directions rather than as a serde data
format. serde's data model preserves integer widths that this encoding
deliberately discards, decoding into a dynamic value type would go through
`deserialize_any` for everything, and most decisively `serialize_struct` emits
fields in declaration order, which would silently produce non-canonical bytes
for exactly the derive-based usage that would justify building the format.
Internal structs that need storing as tables get an explicit
`From<&T> for Properties` and `TryFrom<&Properties> for T` pair instead, keeping
both directions adjacent so they cannot drift. `Serialize` and `Deserialize` are
still derived on `Value` and `Properties` for the JSON boundary.

Property tables are the most frequently encoded thing in the system, so this
path deserves round-trip property tests over the full value domain, including
empty tables, empty strings, empty lists, and the boundaries of every numeric
type.

`amq-protocol-types` (10.6.3 at time of writing, roughly 12 million downloads)
implements this type system for the Rust AMQP ecosystem. It is not a dependency
here, because it is shaped for a message broker rather than a database and
because the codec is small enough to own. It is worth reading as a second
opinion on tag handling before finalising ours.
