# 6. Use LanceDB for auto-embeddings

Date: 2026-08-03

## Status

Accepted

## Context

Graph traversal answers structural questions: what is connected to what. It
cannot answer questions about meaning. "Find nodes similar to this one" or "find
documents about deployment failures" require vector similarity search, not a
path expression.

The intent is that this is transparent. Writing a node with text properties
should produce an embedding without the user asking for one, so that similarity
search is available alongside Cypher traversal rather than being a separate
system the user has to populate and keep in sync.

That needs somewhere to put vectors and an approximate nearest neighbour index
over them. fjall cannot do this: it is an ordered key value store with no
concept of vector distance, and a brute force scan over every vector defeats the
purpose.

The options were an external vector database such as Qdrant, Weaviate, or
Milvus, which breaks the single binary embedded model and turns dotilla into a
system that requires an operator; a hand rolled ANN index over fjall, which is a
research project in its own right; or an embedded vector store.

LanceDB is embedded, is usable from Rust, stores data in the columnar Lance
format, and provides ANN indexing.

## Decision

Use LanceDB as the vector store, embedded in the dotilla process, with one
LanceDB dataset per dotilla database stored alongside that database's fjall
data. See ADR 8 for the on-disk layout.

## Consequences

dotilla stays a single binary with no external services. Similarity search and
graph traversal are available from the same process against the same logical
database.

The columnar format is a good fit for vectors and for the scan heavy access
pattern of ANN search, which is a different pattern from fjall's point lookups.
Using the right structure for each is better than forcing both into one.

This introduces a second storage engine, and the two have no shared transaction.
A write that must land in both cannot be made atomic across them. This is the
significant cost of the decision and is dealt with separately in ADR 9.

Dependency weight increases substantially. LanceDB and Arrow are large, and
build times and binary size will both grow noticeably.

Disk usage roughly doubles for any property that is embedded, since the text is
stored in fjall and its vector in Lance. Vectors are dense floats and do not
compress well.

Every open dotilla database now holds two sets of engine resources rather than
one, which tightens the limit on how many can be open at once. See ADR 8.

LanceDB is a young and fast moving project. API churn should be expected, and
access to it should be confined behind our own types rather than spread through
the codebase.
