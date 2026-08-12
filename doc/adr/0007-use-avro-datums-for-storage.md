# 7. Use Avro Datums for Storage

Date: 2026-08-12

## Status

Accepted

## Context

When we store structured data in the database, we need a way to encode it into
a format that can be stored as bytes and decoded back into the original data
structure.

Avro datums are a great way to handle this since we know the contract shape of
the data we need to store. They are also a well-supported format with a mature
ecosystem.

Previously I planned on using AMQ Tables, and had a fully working implementation,
however I realized we'd end up with wasted disk usage by encoding the field names
in every datum. If we were going to do that, we might as well have used MessagePack
instead.

## Decision

Use the apache-avro Rust library that provides traits for encoding and decoding
Avro datums using Serde. At the time of this writing, apache-avro has > 2M downloads
from crates.io and is the mature avro implementation for Rust.

## Consequences

We will have less code to maintain for the encoding and decoding of data.
