# 5. Use RFC-7807 Problem Details for error responses

Date: 2026-07-31

## Status

Accepted

## Context

dotilla is an HTTP API, so errors are responses. Without a convention, error
bodies drift: some endpoints return a bare string, some return
`{"error": "..."}`, some return nothing at all and rely on the status code. A
client then has to special case each endpoint.

An HTTP status code alone is not enough. A `400` from a Cypher query could be a
syntax error, an unknown label, or a type mismatch, and the client needs to be
able to tell them apart, ideally while also showing something useful to a human.

RFC 7807 defines exactly this: a JSON object with `type`, `title`, `status`,
`detail`, and `instance`, served as `application/problem+json`, and it
explicitly permits additional members.

## Decision

Return all errors as RFC 7807 Problem Details documents.

Use `about:blank` as the `type` for now, with `title` carrying the human
readable summary, `status` mirroring the HTTP status code, `detail` describing
the specific occurrence, `instance` identifying the request, and a non-standard
`hint` member suggesting a fix.

## Consequences

Every error in the system has one shape, so a client writes one error handler.

`hint` is an extension member, not part of the specification. RFC 7807 allows
extensions, so this is legal, but a generic Problem Details client will ignore
it. That is acceptable: it exists for humans reading a response in a terminal,
and it is the field most likely to make a syntax error in a Cypher query
actionable.

Using `about:blank` for `type` means the status code is the only machine
readable classification. This is the specification's own default and is fine
while the error taxonomy is still moving, but it defers the useful part.
Assigning real URIs, for example `https://dotilla.dev/errors/cypher-syntax`,
turns the type into a stable identifier a client can branch on without parsing
prose. Worth doing once the error set settles, and it is a backwards compatible
change.

RFC 7807 was obsoleted by RFC 9457 in July 2023. The format is unchanged in the
parts used here, so nothing needs to move, but new references should cite 9457
and this ADR's title is already slightly stale.

Error bodies must not leak filesystem paths or internal type names in `detail`.
That is a discipline requirement, not something the format enforces.
