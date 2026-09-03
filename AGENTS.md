# AGENTS.md - Poketto Repository Guide

Scope: This file governs the entire repository.

Read this first if you're contributing, reviewing, or acting as an automated coding agent.

## Project Overview

**Poketto** is a Visual Novel Game Launcher rewritten in pure Rust with a
Slint native UI (no webview, no JavaScript runtime):

- **UI**: Slint (`ui/*.slint`, compiled via `slint-build` in `build.rs`)
- **Backend**: Rust workspace, Tokio async runtime
- **Database**: SQLite via `rusqlite` (`bundled` feature) with WAL mode
- **APIs**: VNDB API integration, Discord Rich Presence
- **Targets**: Linux (Wayland/X11) and Windows. Budget: < 60 MB idle RAM,
  stutter-free virtualized cover grid at 60+ FPS scroll.

## Language Policy (English Only)

- The application UI, code, identifiers, docs, commit messages, GitHub
  issues, and PR descriptions are **English only**.
- NEVER change the application language, add translations, or introduce an
  i18n framework unless explicitly requested by the user.
- NEVER rename existing user-facing English strings to another language.
- Ported legacy strings stay in their original English wording.

## Reading Order

1. This file (`AGENTS.md`)
2. `./PRD.md` (rewrite spec, subsystem contracts, phased roadmap)
3. `crates/poketto-core/src/lib.rs` (domain logic entry: db, vndb, wine,
   discord, process)
4. `crates/poketto-app/ui/app.slint` (UI root) and
   `crates/poketto-app/src/main.rs` (runtime wiring)
5. Legacy reference only: `src-tauri/src/lib.rs` + `src/App.tsx` (frozen
   Tauri/SolidJS implementation; port logic FROM it, never extend it)
6. GitHub Issues: https://github.com/betadyne/Poketto/issues

## Installed Agent Skills

Project skills live in `.agents/skills/` and MUST be consulted for
migration work:

- `slint` (`slint-ui/ai-plugins`, official) - Slint language, layout,
  callbacks, models, interop, theming, debugging. Read before writing or
  reviewing any `.slint` file.
- `rust` (`ulpi-io/skills`) - Idiomatic Rust guidance for porting legacy
  backend modules into `poketto-core`.
- `sqlite-expert` (`rightnow-ai/openfang`) - SQLite schema, migration, and
  WAL-mode practices for the `redb`/JSON to `rusqlite` transition.

## Intent & Principles

- **SOLID, KISS, YAGNI** - Keep it simple and avoid over-engineering
- **Cross-platform first**: Support Windows and Linux (Wine/Proton)
- **Offline-capable**: Core functionality works without internet
- **Native UI**: No webview. No TypeScript. No Tailwind. Slint primitives
  only; design tokens before components
- **Testability**: Modular boundaries, pure functions where possible
- **Clarity**: Idiomatic Rust/Slint naming, minimal comments

## Expectations for Agents/Contributors

- Skim `crates/poketto-core/src/lib.rs` for backend architecture context
  before coding
- Read the `slint` skill before touching `ui/*.slint`
- Drive all planning via GitHub Issues (no in-repo trackers)
- Keep changes small and focused; one feature per PR
- Run `cargo check`, `cargo clippy`, and `cargo test` before committing
- NEVER start the migration build unless the user asks; planning and review
  only until then
- If behavior/architecture changed, update this AGENTS.md in the same commit

## Session Handoff Protocol (GitHub Issues)

- **Start**: Pick a ready P0 issue, self-assign, post a "Session Start" plan
- **During**: Post concise updates at milestones; adjust labels as needed
- **End**: Post "What landed" + "Next steps" and update labels/boards

## Phase Completion Protocol (Commit and Push)

- A phase is done only when all gates pass: `cargo check --workspace`,
  `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`
  (plus render verification for UI phases).
- Commit only phase files via selective `git add` (explicit paths, never
  `-A`); NEVER sweep unrelated dirty files into a phase commit.
- Message format `<type>(phase-N): <short description>`, English only.
- Push with `git push origin <branch>`; open or update the PR with
  `gh pr create` / `gh pr status`; post "What landed" + "Next steps" on
  the tracking issue with `gh issue comment`.
- NEVER push a red build. If gates fail, fix first, then commit.


## Code Organization

### Solution Layout (Target)

```
Poketto/
├── PRD.md
├── AGENTS.md
├── Cargo.toml                    # Workspace definition
├── crates/
│   ├── poketto-core/             # Business logic, DB, network, platform
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── db/               # rusqlite repositories + migrations
│   │       ├── vndb/             # VNDB HTTPS client + JSON mappers
│   │       ├── wine/             # Proton/Wine scanner + prefix analyzer
│   │       ├── discord/          # Rich Presence background worker
│   │       ├── process/          # Game process launcher + playtime tracker
│   │       └── lib.rs
│   └── poketto-app/              # Entry point + Slint UI
│       ├── Cargo.toml
│       ├── build.rs              # slint_build::compile("ui/app.slint")
│       ├── ui/
│       │   ├── tokens.slint      # Theme colors, spacing, typography
│       │   ├── components/       # GameCard, SearchBar, Sidebar, TagBadge
│       │   ├── views/            # LibraryView, DetailView, SettingsView
│       │   └── app.slint         # Root Window
│       └── src/
│           ├── main.rs           # Tokio runtime + Slint event loop
│           ├── state.rs          # App state + background thread bridges
│           ├── adapters/         # Rust models -> Slint ModelData
│           └── image_loader.rs   # Async disk cache -> slint::Image
├── src-tauri/                    # LEGACY Tauri backend (frozen reference)
├── src/                          # LEGACY SolidJS frontend (frozen reference)
└── .agents/skills/               # slint, rust, sqlite-expert
```

### File Layout Rules

- **One component per file**: Each Slint component in its own `.slint` file
- **Tokens first**: No new component without entries in `ui/tokens.slint`
  (colors, radius, spacing, typography)
- **Domain modules**: Core logic grouped by feature under
  `crates/poketto-core/src/`; UI stays in `crates/poketto-app/`
- **Legacy is read-only**: `src-tauri/` and `src/` are porting references.
  NEVER add features, fix bugs, or regenerate bindings there
- **No JS/CSS porting**: NEVER transliterate JSX/Tailwind into Slint.
  Decompose each screen into `HorizontalLayout`, `VerticalLayout`, and
  `Rectangle` plus an explicit `TouchArea` per interactive element

## Workflow & Quality

### Development Commands

```bash
cargo check --workspace
cargo clippy --workspace -- -D warnings
cargo test --workspace
cargo run -p poketto-app
```

Run all commands through the rustup toolchain (`rust-toolchain.toml`,
`~/.cargo/bin` first in `PATH`). System cargo with a version-skewed
clippy (e.g. Fedora rustc 1.98 + clippy 1.95) fails with E0514; that is
an environment bug, never a code bug.

Slint tooling (when installed): `slint-viewer` for live `.slint` preview,
`slint-lsp` for editor support.

### Dependency Rules

- Pin every dependency to an explicit semver version. NEVER use
  `"latest"` in `Cargo.toml`; unpinned versions break reproducible builds.
- Baseline: Rust 1.82+, edition 2021, `tokio` full, `slint` with
  `backend-winit` + `renderer-femtovg` by default. `renderer-skia` is an
  opt-in feature for release benchmarking only.
- `reqwest` uses `default-features = false` with `rustls`, `json`,
  `stream` (0.13 renamed `rustls-tls` to `rustls`). Verify TLS root
  handling on Windows when touching network code.

### Threading Rules

- The Slint event loop thread NEVER blocks. No I/O, no `.wait()`, no
  `block_on` inside callbacks.
- All VNDB, filesystem, database, and process work runs on Tokio tasks.
  Filesystem scans run in `spawn_blocking`.
- Push results to the UI only via `slint::invoke_from_event_loop` with a
  `Weak<AppWindow>` handle.
- Lists use `Rc<slint::VecModel<T>>` + Slint `ListView`; NEVER rebuild the
  full model per frame.

### Database Rules

- SQLite via `rusqlite` (`bundled`), WAL mode, single writer.
- Schema changes ship as versioned migrations in `poketto-core/src/db/`,
  plus unit tests per migration.
- There is NO legacy SQLite schema to port: the old app stored
  `games.json`, `settings.json`, `daily_playtime.json`, and a `redb`
  VNDB cache. New schema is designed from the legacy models
  (`GameMetadata`, `AppSettings`), and a one-shot importer covers
  existing users' JSON/redb data.

### Image Pipeline Rules

- VNDB covers download on background workers, raw files under the
  platform cache dir, thumbnails capped at display size (~300 px wide),
  decoded to `SharedPixelBuffer<Rgba8Pixel>` then `slint::Image`.
- Thumbnails need an eviction cap, a placeholder/error visual state, and
  load cancellation on fast scroll. No full-resolution texture on the UI
  thread.

### Test Structure (Target)

```
crates/poketto-core/
├── src/db/         # Migration + repository tests
├── src/vndb/       # JSON mapper + client tests (mocked HTTP)
├── src/wine/       # Classifier/detector/scanner tests
└── src/process/    # Launcher/tracker tests (mocked processes)

crates/poketto-app/
└── src/adapters/   # Model -> Slint data conversion tests
```

### Code Quality Rules

- Handle errors gracefully with user-friendly English messages
- Never commit secrets or tokens (`.env` is gitignored)
- Keep the repo clean: generated artifacts are in `.gitignore`
- **DO NOT add comments to the code** - The codebase intentionally avoids comments. Code should be self-documenting through clear naming and structure. Do not add inline comments, block comments, or documentation comments unless explicitly requested.

### Priority Labels

- **P0**: Critical - blocks release or core functionality
- **P1**: High - important feature or significant bug
- **P2**: Medium - nice to have, can wait

## Coding Standards

### Slint (UI)

- Tokens in `ui/tokens.slint` first; components reference `Theme.*`
- Every interactive element gets an explicit `TouchArea`; hover state via
  `touch.has-hover`, events forwarded through declared `callback`s
- Data lists bind to `VecModel` from Rust adapters; no inline data logic
  in `.slint` files
- Slint has no web backdrop-blur: NSFW hiding uses opaque overlays, NEVER
  a CSS-blur transliteration
- Icons are monochrome `.svg` files under `ui/icons/`, sourced from
  Tabler Icons (https://tabler.io/icons, docs at https://docs.tabler.io).
  Recolor at use sites with `Image { colorize: Theme.* }`. NEVER hand-draw
  glyphs as inline `Path` elements.

### Rust (Core + App)

- Use `Result<T, E>` for fallible operations with `thiserror` error types
- Naming: `snake_case` for functions/variables, `PascalCase` for types
- Core (`poketto-core`) MUST NOT depend on Slint or UI types; `poketto-app`
  owns all `slint::` imports and adapters
- Linux-only modules (Wine/Proton scan) gate with `#[cfg(target_os =
  "linux")]` and provide Windows stubs

### Replacing Tauri Patterns (Do Not Reintroduce)

- No `#[tauri::command]` / IPC handlers: core exposes plain async Rust
  APIs, the app layer bridges them to Slint callbacks
- No `tauri-specta` / `bindings.ts`: cross-boundary types are Rust structs
  converted in `adapters/`
- No Tauri plugins: dialogs via native crates, updates via release-check
  worker, logging via `tracing` with an in-app buffer

## Git Workflow

### Branch Strategy

- `main` - stable release branch
- Feature branches: `feature/<name>`
- Bug fixes: `fix/<name>`

### Commit Message Format (English only)

```
<type>: <short description>

[optional body]
```

Types: `feat`, `fix`, `docs`, `refactor`, `chore`, `test`

### Pull Request Guidelines (English only)

1. Create feature branch from `main`
2. Make changes, verify with `cargo test --workspace`
3. Commit with descriptive messages
4. Push and create PR via GitHub
5. Link related issues

## Ambiguity Resolution

- Prefer the simplest design that satisfies current requirements
- When in doubt, follow existing patterns in the codebase
- For Slint API questions, consult the `slint` skill references first
- User instructions take precedence over this document
- Ask for clarification rather than making assumptions.
