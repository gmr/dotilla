# 6. Use edgebert for creating the vectors

Date: 2026-08-03

## Status

Accepted

## Context

ADR 6 decided where vectors are stored. This decision is about where they come
from.

The embeddings are meant to be transparent, produced as a side effect of writing
a node. That constrains the options more than it first appears.

Calling a hosted embedding API such as OpenAI or Cohere would mean every write
to dotilla depends on a network round trip, an API key, and someone else's rate
limit and uptime. A database whose writes fail because a third party is down is
not a database. It also means node contents leave the machine, which is not a
reasonable default for an embedded store.

Running a local inference server such as Ollama or a Python sidecar keeps data
local but reintroduces the external process that ADR 6 was trying to avoid.

That leaves in-process inference in Rust. The candidates are `candle`, which is
a general ML framework and correspondingly large, `ort` bindings to ONNX
Runtime, which pull in a C++ dependency and undo ADR 3's toolchain argument, and
`edgebert`, which is a focused BERT inference library for text embeddings with
no native dependencies.

## Decision

Generate embeddings in-process using the `edgebert` crate.

## Consequences

Embedding happens locally. No API key, no network, no data leaving the machine,
and no third party in the write path. Writes cannot fail because a vendor is
down.

The single binary story holds. No C++ toolchain, consistent with ADR 3.

Inference is CPU work happening inside an async server, so it must not run on a
tokio worker thread. Embedding belongs on a blocking pool or a dedicated worker,
which is a further reason for the asynchronous outbox in ADR 9 rather than
embedding inline on the write path.

Model weights have to be obtained and stored somewhere. Whether they are
bundled, downloaded on first use, or supplied by the operator is an open
question this ADR does not settle. Bundling inflates the binary; downloading
reintroduces a network dependency, though at startup rather than per write.

Quality will be lower than large hosted embedding models. For similarity search
over node properties this is likely an acceptable trade, but it is a real one
and should be stated rather than discovered.

`edgebert` is pre-1.0, recently published, and has modest adoption. This is a
larger bet than the other dependencies in this project. The embedding provider
should therefore sit behind a trait with the model dimension treated as
configuration, so that replacing it later is a new implementation rather than a
rewrite. That trait boundary is the actual mitigation and should exist from the
start.

Changing the embedding model invalidates every stored vector, because vectors
from different models are not comparable. The model identity and dimension must
be recorded alongside the data so that a mismatch is detected at open time
rather than silently returning meaningless similarity results. Re-embedding is
then a rebuild of a derived store, which ADR 9 makes possible.
