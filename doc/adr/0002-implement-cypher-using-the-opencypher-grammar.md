# 2. Implement Cypher using the OpenCypher grammar

Date: 2026-07-31

## Status

Accepted

## Context

A graph database needs a query language. The realistic options were to invent
one, or to adopt an existing one.

Inventing a language is the largest possible detour from the actual work, which
is graph storage and traversal. It also produces something nobody knows.

Of the existing graph query languages, Cypher has the widest mindshare, largely
through Neo4j. Gremlin is a traversal API rather than a declarative language and
reads poorly over HTTP. SPARQL assumes an RDF triple model that does not match a
labelled property graph. ISO GQL is the eventual standard but is young, and there
is no usable Rust tooling for it.

openCypher is the open specification of Cypher.

## Decision

Implement the query language as openCypher.

## Consequences

Users who know Neo4j can use dotilla without learning anything new, and the
existing body of Cypher documentation and examples applies.

Adopting Cypher's surface syntax is not the same as adopting Neo4j's behaviour.
Procedures, `APOC`, and Neo4j-specific extensions are out of scope, and openCypher
itself diverges from both Neo4j Cypher and ISO GQL. Full compatibility with
anything is not achievable and should not be promised. The claim is "Cypher
dialect", not "Cypher compatible".
