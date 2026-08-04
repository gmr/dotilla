# 4. Use TOML for configuration

Date: 2026-07-31

## Status

Accepted

## Context

The server needs a configuration file for the data directory, listen address,
and logging, with sensible defaults so that running the binary with no arguments
works.

The realistic formats are TOML, YAML, and JSON.

JSON has no comments, which makes it hostile as a hand edited configuration
format. YAML is comment friendly but the specification is large and its implicit
typing is a known source of surprise, where unquoted values are coerced into
booleans or numbers depending on how they are spelled. TOML is small, explicitly
typed, comment friendly, and is already the format every Rust developer edits
daily in `Cargo.toml`.

## Decision

Configure the server with a TOML file, located by a `--config` flag, with
defaults of `~/.dotilla` for the data directory and `127.0.0.1:6465` for the
listen address. Parse with `serde` and `toml`.

## Consequences

Configuration files can be commented, which matters for anything an operator
tunes and later has to explain to themselves.

`serde` derives the whole parser, so the config struct is the schema and adding
a field is one line.

Defaulting the listen address to loopback rather than `0.0.0.0` means the server
is not exposed to the network by accident. Exposure is a deliberate act.

TOML's table syntax gets awkward past two levels of nesting, particularly for
arrays of tables. This is a mild constraint in favour of a flat configuration,
which is a reasonable thing to be pushed towards.

TOML has no mechanism for secrets or interpolation. If dotilla ever needs
credentials, they will have to come from the environment or a file reference,
and that will be a separate decision.

Validating that the data directory is writable happens at load time rather than
at first write, so a misconfigured server fails at startup with a clear message
instead of failing later under load.
