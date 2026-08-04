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

openCypher is the open specification of Cypher, and a `tree-sitter-cypher`
grammar already exists, which means a parser is a dependency rather than a
project.

## Decision

Implement the query language as openCypher, parsed with `tree-sitter` using the
`tree-sitter-cypher` grammar.

## Consequences

Users who know Neo4j can use dotilla without learning anything new, and the
existing body of Cypher documentation and examples applies.

Parsing is solved on day one. The grammar is maintained externally, so its
coverage and its bugs are inherited rather than owned. Where the grammar is
wrong or incomplete, the options are to patch it upstream or to work around it
in lowering.

tree-sitter produces a concrete syntax tree, not an abstract one, and it is
error tolerant by design. That is the right behaviour for an editor and the
wrong behaviour for a database, which must reject malformed input rather than
guess. Lowering the CST into an internal representation, and rejecting anything
containing an error node, is work that is now on us.

`tree_sitter::Parser::parse` takes `&mut self`, so a parser cannot be shared
through the `Arc<AppState>` the handlers see. It therefore sits behind a
`Mutex`, which serialises parsing across concurrent requests. If that becomes a
bottleneck the fix is a small pool of parsers, or one per request, not a
redesign.

Adopting Cypher's surface syntax is not the same as adopting Neo4j's behaviour.
Procedures, `APOC`, and Neo4j-specific extensions are out of scope, and openCypher
itself diverges from both Neo4j Cypher and ISO GQL. Full compatibility with
anything is not achievable and should not be promised. The claim is "Cypher
dialect", not "Cypher compatible".
