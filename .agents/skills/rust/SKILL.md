---
name: rust
version: 2.1.0
description: |
  Write and change Rust the way THIS workspace already does it, NOT by generic defaults — a
  systems-programming reference carrying the real crate boundaries, error model, and conventions for
  storage engines, binary formats, Arrow/DataFusion, search and vector indexing, async concurrency,
  testing, and unsafe discipline, so a change lands idiomatic and review-ready instead of merely
  compiling. Use when a task touches this workspace's Rust crates and should follow its systems-level
  conventions rather than boilerplate.
allowed-tools:
  - Bash
  - Read
  - Write
  - Edit
  - Grep
argument-hint: "[Rust crate, subsystem, or implementation task]"
arguments:
  - request
when_to_use: |
  Use when the task touches Rust crates, Cargo manifests, async/concurrency code, storage layers,
  binary formats, Arrow/DataFusion, vector search, geo indexing, or systems-level testing. Examples:
  "fix this Rust crate", "add a storage-engine feature", "update this async path", "review this
  binary format code". Do NOT use for non-Rust code, and do not treat it as the action itself — it
  supplies the conventions a change follows; the actual bug hunt (find-bugs / review-crate), plan, or
  build is still its own skill.
effort: high
---

<EXTREMELY-IMPORTANT>
This skill is the routing shell over the Rust reference set, not the whole systems handbook.

Non-negotiable rules:
1. Identify the subsystem before coding. Load only the relevant references.
2. **Safe Rust by default.** `unsafe` only when justified — every `unsafe` block gets a `// SAFETY:` comment.
3. **Zero-copy where possible.** mmap'd segments, `&str`/`&[u8]` references into pages. Never copy data without reason.
4. **Error types per crate.** `thiserror` enums in library crates. Never `anyhow` in libraries — only in binaries/tests.
5. **Async with tokio.** All I/O is async. CPU-bound work on `spawn_blocking`. Never block the tokio runtime.
6. **Workspace crate structure.** One crate per subsystem. Depend downward — never circular.
7. **Property tests for invariants.** `proptest` for round-trips, type coercion, binary parsing.
</EXTREMELY-IMPORTANT>

# rust

## Inputs

- `$request`: The crate, subsystem, bug, feature, or review target

## Goal

Route Rust work through the right subsystem conventions so changes follow the workspace’s systems-programming patterns instead of generic Rust defaults.

## Step 0: Identify the subsystem

Decide which part of the Rust surface the task touches:

- storage engine
- binary formats
- type system
- Arrow or DataFusion
- wire protocols
- search or vectors
- arena or graph code
- geo indexing
- async or concurrency
- testing
- unsafe or error design

**Success criteria**: The task is mapped to the right subsystem before references are loaded.

## Step 1: Load only the relevant references

Use the routing table to pick the matching files. Do not bulk-load the full reference tree.

| Task / Area | Read |
|---|---|
| Toolchain, workspace layout, key crates, Cargo conventions | `references/stack.md` |
| WAL, mmap, segments, MVCC, compaction, io_uring, backends | `references/storage-engine.md` |
| Custom on-disk formats, zero-copy parsing, packed structs | `references/binary-formats.md` |
| Database type system, disk/exec types, Arrow interop | `references/type-system.md` |
| DataFusion table providers, UDFs, Arrow RecordBatch, planning | `references/datafusion-arrow.md` |
| pgwire server, MySQL protocol, dialect mapping, ORM compat | `references/wire-protocols.md` |
| tantivy FTS, HNSW vectors, SIMD distance, hybrid search | `references/search-vector.md` |
| bumpalo arenas, graph adjacency, traversal algorithms | `references/arena-graph.md` |
| R-tree spatial index, geo predicates, WGS84 distance | `references/geo-rtree.md` |
| tokio async, io_uring, crossbeam, lock-free structures, MVCC | `references/async-concurrency.md` |
| proptest, deterministic testing, integration test patterns | `references/testing.md` |
| thiserror hierarchies, unsafe patterns, safety invariants | `references/error-unsafe.md` |

Multiple tasks? Read multiple files.

**Success criteria**: Only the subsystem-relevant Rust guidance is active.

## Step 2: Implement with the core Rust guardrails

Keep these rules active:

- safe Rust by default; `unsafe` only when justified with `// SAFETY:`
- error types match crate boundaries (`thiserror` in libs, `anyhow` only in bins/tests)
- async code does not block the runtime; CPU work on `spawn_blocking`
- owned vs borrowed data choices are deliberate; prefer `&[u8]` over `Vec<u8>` in signatures
- explicit `use` imports — no glob imports except in test modules
- `#[must_use]` on functions returning `Result` or computed values
- `#[inline]` only on small functions in hot loops — never on public API
- `Send + Sync` bounds on trait objects crossing async boundaries
- `#[derive(Debug)]` on all public types; `#[derive(Clone)]` only when cheap
- feature flags for optional crate deps — `#[cfg(feature = "...")]`
- tests match the risk: unit, integration, property, snapshot, or domain-specific verification

**Success criteria**: The implementation fits the workspace’s systems-level quality bar.

## Step 3: Verify the change

Use the narrowest relevant verification loop:

- `cargo fmt`
- `cargo clippy -- -D warnings`
- focused crate tests
- subsystem-specific tests such as proptest or snapshots when appropriate

**Success criteria**: The Rust surface is validated the way this workspace expects.

## Guardrails

- Do not inline the whole Rust handbook in `SKILL.md`.
- Do not skip subsystem identification.
- Do not use `anyhow` in library crates unless the project specifically allows it there.
- Do not add `disable-model-invocation`; this is a normal domain skill.
- Do not leave unsafe invariants undocumented.

## When To Load References

- `references/stack.md`
  Use for workspace/toolchain context.

- then only the task-relevant subsystem files under `references/`

## Output Contract

Report:

1. which Rust references were loaded
2. the subsystem pattern applied
3. the change made
4. the verification run
