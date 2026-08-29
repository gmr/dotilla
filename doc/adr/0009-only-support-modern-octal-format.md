# 9. Only support modern octal format

Date: 2026-08-29

## Status

Accepted

## Context

Octal integers have modern `0o` and `0O` prefixes and a legacy leading-zero
form such as `0755`. Supporting both complicates number lexing and makes a
leading zero ambiguous.

## Decision

Only accept the modern `0o` and `0O` octal formats. Do not interpret a
leading-zero integer as octal.

## Consequences

Number lexing remains simpler and unambiguous. Queries using legacy octal
literals must be updated to use the modern prefix.
