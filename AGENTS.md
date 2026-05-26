# AGENTS.md

Operating guide for AI coding agents (GitHub Copilot CLI, Copilot coding
agent, Cursor, Claude Code, Aider, Continue, and similar) working in the
`embedded-keyboard-rs` repository. Human contributors are welcome to
read it too — it doubles as a quick orientation to the workspace.

This file is the canonical source for agent-facing conventions in this
repo. The shorter `.github/copilot-instructions.md` only carries
commit-message and AI-attribution rules; both files should be read
together.

---

## 1. Repository at a glance

- **Name:** `embedded-keyboard-rs`
- **Organization:** [`OpenDevicePartnership`](https://github.com/OpenDevicePartnership)
- **Language:** Rust (edition 2021, MSRV `1.79`)
- **Type:** Cargo workspace (`resolver = "2"`) with two member crates.
- **License:** MIT
- **Target:** `no_std` embedded systems built on top of the
  [`embedded-hal`](https://crates.io/crates/embedded-hal) 1.x traits.
- **Default branch:** `main`
- **Code owners:** `@OpenDevicePartnership/ec-code-owners`
  (see `CODEOWNERS`).

### Workspace layout

```
embedded-keyboard-rs/
├── Cargo.toml              # workspace manifest
├── README.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
├── SECURITY.md
├── LICENSE                 # MIT
├── CODEOWNERS
├── .github/
│   └── copilot-instructions.md
├── embedded-keyboard/      # crate: HAL traits + keycode enum
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       └── keycode.rs
└── gpio-keyboard/          # crate: GPIO key-matrix driver (published as `embedded-keymatrix`)
    ├── Cargo.toml
    ├── README.md
    └── src/
        └── lib.rs
```

### Workspace members

| Path                 | Package name          | Role                                                                       |
| -------------------- | --------------------- | -------------------------------------------------------------------------- |
| `embedded-keyboard`  | `embedded-keyboard`   | Core HAL: `Keyboard`, `ErrorType`, `Error`/`ErrorKind`, `KeyEvent`, `KeyCode`, `Coordinate`. |
| `gpio-keyboard`      | `embedded-keymatrix`  | `KeyMatrix<ROWS, COLS, NKRO, I, O>` driver implementing the `Keyboard` trait on top of `embedded-hal` `InputPin`/`OutputPin`. |

Note the asymmetry: the directory `gpio-keyboard/` publishes a crate
named **`embedded-keymatrix`**. Always consult `Cargo.toml` for the
canonical package name before referencing crates in docs, error
messages, or external configs.

The workspace `Cargo.toml` also defines a
`[patch.crates-io] embedded-keyboard = { path = "embedded-keyboard" }`
so the in-tree crate is used when building `embedded-keymatrix`, even
though `gpio-keyboard/Cargo.toml` pins the dependency to `"0.1.0"`.
Keep that patch in mind when changing the public API of
`embedded-keyboard` — local changes will be consumed immediately by
`embedded-keymatrix` during a workspace build.

---

## 2. Why this project exists

`embedded-keyboard-rs` provides a small, `no_std`-friendly hardware
abstraction layer for keyboard controllers, plus a generic GPIO
key-matrix driver built on top of it. The intent is to let firmware
authors target a single `Keyboard` trait regardless of whether they
scan a passive GPIO matrix, talk to a dedicated keyboard controller IC,
or sit behind a more complex subsystem (e.g. an EC).

Design tenets:

- Stay `no_std`. The only `std` usage is gated behind `#[cfg(test)]`.
- Build on `embedded-hal` 1.x traits — do not invent new pin
  abstractions.
- Keep the public surface minimal and orthogonal. Add traits and
  variants only when concrete drivers need them.
- Errors are described by a trait (`Error`) plus a small
  `#[non_exhaustive]` `ErrorKind` enum so HAL implementations can
  define richer error types and still map back to a common set.
- The matrix driver is generic in `ROWS`, `COLS`, and `NKRO` (n-key
  rollover report length) via const generics; no heap allocation.

When proposing changes, preserve these tenets. New features that pull
in `std`, an allocator, or a non-`embedded-hal` pin abstraction need
explicit human sign-off in the PR description.

---

## 3. Toolchain and environment

- **Rust toolchain:** stable, MSRV `1.79` (see workspace
  `rust-version`). Do not introduce features that require a newer
  compiler without bumping `rust-version` in the workspace manifest
  and calling that out in the PR.
- **No `rust-toolchain.toml`** is checked in; agents should use the
  user's default stable toolchain via `rustup`.
- **Required components:** `rustfmt`, `clippy`. Both should already be
  installed with a default `rustup` profile; if not, run
  `rustup component add rustfmt clippy`.
- **No `cross`, QEMU, or hardware-in-the-loop setup is required.**
  All tests run on the host with `embedded-hal-mock`.
- **Editor/CI:** there are currently **no GitHub Actions workflows**
  in `.github/workflows/`. CI is effectively whatever a human runs
  locally before review. Treat the commands in §6 as the de-facto CI.

### Dependencies of note

Workspace-level (declared in the root `Cargo.toml`):

- `embedded-hal = "1.0.0"`
- `embedded-hal-mock = "0.11.1"` (dev only)
- `itertools = "0.13.0"` (dev only)

Per-crate:

- `embedded-keyboard`: optional `defmt = "0.3.8"` behind the `defmt`
  feature.
- `embedded-keymatrix` (`gpio-keyboard/`): same optional `defmt`
  dependency, plus `embedded-hal` and `embedded-keyboard`.

When adding a dependency, prefer adding it to
`[workspace.dependencies]` and re-exporting via
`dep.workspace = true` in each member that needs it. This keeps
versions consistent across the workspace.

---

## 4. Code conventions

### 4.1 `no_std` discipline

- Every library crate uses `#![cfg_attr(not(test), no_std)]`. Keep it.
- Do **not** import from `std::` in non-test code. Use `core::` (and
  `alloc::` only if a future crate opts into it explicitly — currently
  none do).
- Test modules (`#[cfg(test)] mod tests { ... }`) may freely use
  `std`. The existing tests in `gpio-keyboard/src/lib.rs` rely on
  `std::io::ErrorKind` and `embedded_hal_mock`.

### 4.2 Lints

`gpio-keyboard/Cargo.toml` enforces a strict lint profile:

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "forbid"

[lints.clippy]
correctness = "forbid"
suspicious  = "forbid"
perf        = "forbid"
style       = "forbid"
pedantic    = "forbid"
```

Practical implications for any change to that crate:

- **No `unsafe`.** Period. If you think you need it, surface that in
  the PR description and ask for human review before writing the code.
- **Every public item needs a doc comment.** Including struct fields
  that become `pub`, enum variants, trait methods, and free functions.
  `cargo doc` (see §6) will fail fast otherwise.
- **All forbidden clippy groups must pass.** `pedantic` is the most
  surprising one; expect to address lints like
  `needless_pass_by_value`, `must_use_candidate`,
  `module_name_repetitions`, etc. Prefer fixing the code over
  `#[allow(...)]`. If an allow is unavoidable, scope it as narrowly
  as possible (item-level, not crate-level) and add a short comment
  justifying it.

The `embedded-keyboard` crate does not declare lints today. Apply the
same hygiene by default; if you tighten its lints, do it in a focused
commit and confirm the workspace still builds cleanly.

### 4.3 Formatting

- Run `cargo fmt --all` before committing. There is no custom
  `rustfmt.toml`, so the default style applies.
- Group imports as rustfmt produces by default; do not hand-reorder
  modules in a way that fights the formatter.

### 4.4 Naming and API style

- Mirror `embedded-hal`'s conventions: `ErrorType` associated type,
  `Error` trait with a `kind()` method, `#[non_exhaustive]` error
  enums, blanket impls for `&mut T`.
- Public enums that may grow over time should be `#[non_exhaustive]`
  (`ErrorKind`, `KeyCode`, and `KeyEvent` set the precedent).
- Public types in `embedded-keyboard` that travel over `defmt`
  channels carry `#[cfg_attr(feature = "defmt", derive(defmt::Format))]`.
  Apply the same pattern to any new public enum or struct so that
  downstream `defmt` users get logging for free.
- Prefer `core::result::Result<T, E>` aliases at the crate root when
  a single error type dominates (see `gpio-keyboard`'s `Result`
  alias).
- Const generics over runtime configuration: `KeyMatrix<ROWS, COLS,
  NKRO, I, O>` is the model. New drivers should follow suit unless
  there is a strong reason otherwise.

### 4.5 Documentation

- Crate-level docs live in `//!` comments at the top of `lib.rs`.
  Keep the existing `#![doc(html_root_url = "...")]` annotations
  intact and update them if a crate is renamed or version-bumped.
- Every public item in `gpio-keyboard` **must** have a doc comment
  (enforced by `missing_docs = "forbid"`). Apply the same standard
  in `embedded-keyboard` voluntarily.
- Examples in doc comments should compile. If you add one that should
  not be compiled (e.g. references hardware), use ```` ```ignore ````
  or ```` ```no_run ```` deliberately and explain why in prose.

---

## 5. Branching, commits, and PR etiquette

The authoritative human guide is `CONTRIBUTING.md`. Agents must
follow it in addition to this file. Key rules:

- **No squash merges.** The maintainers explicitly disabled squashing
  to preserve commit history (`CONTRIBUTING.md` §"Clean Commit
  History"). Therefore:
  - Every commit must build cleanly without warnings.
  - Fixup/typo/formatting commits must be squashed locally
    (`git rebase -i`) before review.
  - Do not rely on the PR UI to clean up history for you.
- **Draft PRs first.** Open the PR as a draft so the linting/sanity
  checks can run before requesting reviewers. (Note: there are no
  Actions workflows in-tree today, so "checks" effectively means the
  local commands in §6 plus anything maintainers run manually.)
- **Branch naming:** there is no enforced scheme. Short, descriptive,
  kebab-case branch names (e.g. `add-i2c-keyboard-driver`,
  `fix-keymatrix-overflow`) are recommended.
- **Do not force-push to shared branches** (`main`, anything someone
  else is reviewing). Force-pushing your own feature branch during a
  rebase is fine.
- **Regression reports:** include the output of `git bisect` per
  `CONTRIBUTING.md`.

### Commit messages

From `.github/copilot-instructions.md`:

- Subject line: capitalized, ≤50 characters, imperative mood
  ("Add foo", not "Added foo" or "Adds foo").
- Blank line between subject and body.
- Wrap body at 72 columns.
- Body explains *what* and *why*, not *how*.
- Reference issues/PRs by number where relevant
  (`Fixes #123`, `Refs #456`).

### AI attribution (mandatory)

Every commit produced with AI assistance — whether the agent wrote
the diff, suggested it, or substantially refactored it — **must**
include an `Assisted-by` trailer:

```
Assisted-by: AGENT_NAME:MODEL_VERSION [TOOL1] [TOOL2]
```

- `AGENT_NAME`: e.g. `GitHub Copilot`, `Claude Code`, `Cursor`.
- `MODEL_VERSION`: the actual model in use, e.g.
  `claude-opus-4.7`, `gpt-5.3-codex`. **Verify your own identity
  before composing the trailer; do not hard-code a model name from a
  previous session.**
- Optional `[TOOL]` tags name specialized analyzers used in producing
  the change (e.g. `clang-tidy`, `coccinelle`). Basic tools (`git`,
  `cargo`, editors) are **not** listed.
- Agents **must not** add `Signed-off-by:` trailers. Only humans can
  certify the DCO.

Place trailers in the standard git footer block, separated from the
body by a blank line:

```
Fix keymatrix off-by-one in NKRO truncation

The scan loop wrote past the end of `report` whenever more keys
changed than NKRO allowed. Clamp the index before writing and add
a regression test that drives 8 simultaneous keys through a
KeyMatrix<2, 2, 6, _, _>.

Fixes #42
Assisted-by: GitHub Copilot:claude-opus-4.7
```

### Authoring identity for agent-driven commits

When an agent commits on behalf of a human, set the author identity
per-invocation rather than globally:

```bash
git -c user.name="First Last" \
    -c user.email="first.last@example.com" \
    commit -m "..."
```

Do **not** run `git config --global user.name/email`. Do not invent
an author identity — use the one the human operator provides for the
session.

---

## 6. Build, test, lint, and docs — the de-facto CI

Run these from the workspace root. They are cheap (seconds, not
minutes) and should all pass before pushing.

| Purpose             | Command                                                     |
| ------------------- | ----------------------------------------------------------- |
| Build all crates    | `cargo build --workspace --all-features`                    |
| Build (no features) | `cargo build --workspace`                                   |
| Run tests           | `cargo test --workspace --all-features`                     |
| Lint (clippy)       | `cargo clippy --workspace --all-targets --all-features -- -D warnings` |
| Format check        | `cargo fmt --all -- --check`                                |
| Apply formatting    | `cargo fmt --all`                                           |
| Build docs          | `cargo doc --workspace --all-features --no-deps`            |
| Strict docs build   | `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` |

Notes:

- `--all-features` exercises the `defmt` integration. Always run with
  it; a build that passes without `defmt` can still break the feature.
- Treat clippy warnings as errors locally (`-D warnings`). The crate
  lint profile in `gpio-keyboard` already forbids the dangerous
  groups, but using `-D warnings` catches everything else.
- There is no `cargo test --no-run` target for hardware; the test
  suite is pure host-side with `embedded-hal-mock`.
- If you add a new crate to the workspace, make sure it passes the
  same set of commands and add it to the `members = [...]` list in
  the root `Cargo.toml`.
- Some pedantic clippy lints fire only with `--all-targets`; never
  drop that flag when running clippy as a gating check.

### Feature matrix worth exercising

For each library crate:

- Default features (currently empty).
- `--features defmt`.
- `--no-default-features` (should be identical to default for now,
  but keep it green so downstream `default-features = false` users
  are not broken).

A reasonable one-liner before pushing:

```bash
cargo fmt --all -- --check && \
cargo clippy --workspace --all-targets --all-features -- -D warnings && \
cargo test --workspace --all-features && \
cargo build --workspace --no-default-features && \
cargo doc --workspace --all-features --no-deps
```

---

## 7. Testing guidance

- Unit tests live inline in each crate under `#[cfg(test)] mod tests`.
  Follow the existing patterns in `gpio-keyboard/src/lib.rs`:
  - Use `embedded_hal_mock::eh1::digital::{Mock, State, Transaction}`
    for `InputPin`/`OutputPin` doubles.
  - Always call `.done()` on every mock at the end of the test to
    assert that expectations were consumed.
  - Use `itertools::izip!` for multi-slice iteration when verifying
    state machines (see the integrator test).
- When you add a new public behavior, add at least one positive test
  and one negative/error-path test. The existing
  `error_enabling_column`, `error_reading_row`, and
  `error_disabling_column` tests are the model.
- Prefer table-driven tests (input vectors plus expected state
  vectors) over many near-duplicate test functions.
- Keep tests deterministic. No timing-dependent assertions, no
  reliance on iteration order of non-deterministic collections.
- If you need integration tests later, place them under
  `<crate>/tests/` per Cargo conventions — do not invent a new
  layout.

---

## 8. Working with `defmt`

The `defmt` feature is the project's only optional integration today.
When adding new public types:

- Derive `defmt::Format` behind the feature:

  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  #[cfg_attr(feature = "defmt", derive(defmt::Format))]
  pub struct MyType { /* ... */ }
  ```

- Do not introduce a hard dependency on `defmt`. Use the optional
  dep pattern already in both crate manifests:

  ```toml
  [dependencies]
  defmt = { version = "0.3.8", optional = true }

  [features]
  defmt = ["dep:defmt"]
  ```

- Avoid `defmt::println!`/`info!` in library code. Logging policy
  belongs to the application; the libraries should only surface
  `Format` derives so downstream firmware can log if it wishes.

---

## 9. Adding new functionality safely

When introducing new code, walk through this checklist before pushing:

1. **Scope:** Does the change belong in `embedded-keyboard`
   (trait/abstraction) or in a driver crate? Trait changes are
   load-bearing — discuss in an issue first if uncertain.
2. **`no_std`:** Confirmed no `std::` imports outside `#[cfg(test)]`.
3. **Allocations:** No `Box`, `Vec`, `String`, etc. unless the crate
   has opted into `alloc` (none currently has).
4. **`unsafe`:** None added (forbidden in `gpio-keyboard`,
   discouraged in `embedded-keyboard`).
5. **Public API docs:** Every new `pub` item documented.
6. **`defmt`:** New public enums/structs derive `defmt::Format`
   behind the feature flag.
7. **MSRV:** No language/library features newer than Rust 1.79.
   If you must bump it, also bump `workspace.package.rust-version`
   and call it out in the PR description.
8. **Tests:** Added unit tests covering at least one success and one
   failure path.
9. **Lints/format/docs:** The one-liner in §6 passes locally.
10. **Commit hygiene:** Subject ≤50 chars, body wrapped at 72,
    `Assisted-by:` trailer present, no `Signed-off-by:` added by the
    agent.

---

## 10. Things to avoid

- **Force-pushing to `main`** or to a branch under active review by
  someone else.
- **Editing `LICENSE`, `CODE_OF_CONDUCT.md`, `SECURITY.md`,
  `CODEOWNERS`, or `CONTRIBUTING.md`** as part of an unrelated
  change. These are governance files; touch them only when the PR is
  explicitly about them.
- **Reformatting unrelated files.** Limit `cargo fmt` churn to the
  files you actually modified, or do a separate formatting-only
  commit if a wholesale reformat is requested.
- **Introducing `Cargo.lock`** for library crates without a clear
  reason. The workspace currently does not commit `Cargo.lock`;
  if you start to, treat it as a deliberate policy change requiring
  human approval.
- **Adding CI workflows silently.** The repo has no
  `.github/workflows/` today. If you add one, mention it explicitly
  in the PR description and keep the initial workflow minimal
  (fmt + clippy + test + doc) so reviewers can audit it.
- **Bumping dependency major versions** as a side effect of another
  change. Do it in its own commit with a justification.
- **Renaming or moving public items** without a `#[deprecated]`
  bridge, unless the crate is still pre-1.0 (it is — 0.1.0 — so
  breaking changes are acceptable but should still be flagged in the
  PR description).
- **Writing to `/tmp` or other system temp directories** from
  scripts checked into the repo. Use the workspace itself
  (`target/`, ignored) for scratch output.

---

## 11. Quick reference for common agent tasks

### Add a new keycode

1. Edit `embedded-keyboard/src/keycode.rs`, append a variant to
   `KeyCode` with the next free `u16` discriminant.
2. Keep the enum `#[non_exhaustive]` and `#[repr(u16)]`.
3. Run `cargo build --workspace --all-features` and
   `cargo doc --workspace --all-features --no-deps`.
4. Commit per §5.

### Add a new error variant

1. Edit the relevant `KeyboardError`/`ErrorKind` enum.
2. If you touch `embedded_keyboard::ErrorKind`, remember it is
   `#[non_exhaustive]` — downstream code must not exhaustively match
   on it. Update `Display` and `kind()` mappings.
3. Add tests covering the new variant.

### Add a new driver crate

1. Create `<crate-name>/` next to `embedded-keyboard/` and
   `gpio-keyboard/`.
2. Add it to `members = [...]` in the root `Cargo.toml`.
3. Use `<field>.workspace = true` for `version`, `authors`,
   `license`, `repository`, `edition`, `rust-version`.
4. Depend on `embedded-keyboard` via the workspace patch
   (just `embedded-keyboard = "0.1.0"` is enough; the workspace
   `[patch.crates-io]` redirects it to the in-tree path).
5. Mirror the strict lint profile from `gpio-keyboard/Cargo.toml`
   unless you have a documented reason not to.
6. Implement the `Keyboard` and `ErrorType` traits.
7. Add `#[cfg(test)]` unit tests using `embedded-hal-mock`.

### Bump MSRV

1. Update `rust-version` in the root `Cargo.toml` (workspace).
2. Confirm both crates build on the new MSRV
   (`rustup toolchain install <ver>` and
   `cargo +<ver> build --workspace --all-features`).
3. Mention the bump prominently in the PR description and commit
   subject (e.g. "Bump MSRV to 1.81").

---

## 12. When in doubt

- **Read `CONTRIBUTING.md` and `.github/copilot-instructions.md`.**
  This document defers to them on commit format, AI attribution, and
  PR etiquette.
- **Ask, don't guess.** If a requested change conflicts with the
  tenets in §2 (no `std`, no allocator, `embedded-hal`-only pin
  abstractions, no `unsafe` in driver crates), surface that in the PR
  description or as a question in the issue rather than silently
  working around it.
- **Prefer small, reviewable commits** that each build cleanly over
  one large commit. The no-squash policy makes commit boundaries
  permanent.

Welcome aboard. Keep the public surface small, the doc comments
honest, and the tests green.

## Model selection & cost discipline

Premium models (Opus, GPT-5 family, "high"/"xhigh" reasoning variants)
cost an order of magnitude more than standard models (Sonnet, Haiku,
mini). Most steps in a typical task do not need premium reasoning,
and over-using premium models wastes credits without improving
outcomes. The rules below apply to *all* model selection: your own
session, sub-agents launched via the `task` tool, and parallel work
launched via `/fleet`.

### Default posture

- **Default to the cheapest model that can do the job.** Reach for a
  premium model only when one of the escalation triggers below is hit.
- **Plan with premium, execute with cheap.** Spend at most one or two
  premium turns on design / planning, then downshift to a cheaper
  model for mechanical execution of the plan.
- **Never bump the model "just in case."** If you cannot articulate
  *why* a cheaper model would fail, use the cheaper model.

### Escalation triggers (use a premium model)

Reach for a premium model when *any* of these are true:

- Cross-module refactor, architectural design, or API design from
  scratch.
- Subtle correctness reasoning: concurrency, lifetimes, `unsafe`,
  FFI ABI, cryptography, safety-critical control paths.
- Debugging a failure that survived one prior cheap-model attempt.
- Reviewing code on a safety-, security-, or money-critical path.
- The diff cannot be predicted in advance — i.e. there is genuine
  creative or design work to do, not just typing.

### De-escalation triggers (use a cheap model)

Use the cheapest available model when *any* of these are true:

- Searching, reading, summarising files or docs.
- Single-file mechanical edits: rename, format, lint fix, dependency
  bump, boilerplate, scaffolding from a known template.
- Generating tests for code that already works.
- Running builds, tests, linters, or other commands where the model
  only needs to report success/failure.
- Routine commits, PR descriptions, changelog entries.
- The diff is essentially predictable before generation.

### Sub-agent routing (the `task` tool)

When delegating with the `task` tool, set `model:` explicitly. Do not
let sub-agents inherit a premium default for cheap work.

| Sub-agent type    | Default model             | Override to                                     |
|-------------------|---------------------------|-------------------------------------------------|
| `explore`         | cheap                     | keep cheap (`claude-haiku-4.5` or `gpt-5-mini`) |
| `task` (run cmd)  | cheap                     | keep cheap                                      |
| `research`        | cheap for breadth         | premium only for the final synthesis            |
| `general-purpose` | match task                | cheap for mechanical work; premium for design   |
| `rubber-duck`     | premium                   | keep premium — this is where reasoning pays off |
| `code-review`     | premium on critical paths | cheap on cosmetic / mechanical diffs            |

### `/fleet` (parallel sub-agents) rules

- Fleet mode multiplies cost by the fleet width. Apply the rules
  above *per worker*, not in aggregate.
- Split a fleet job along complexity lines: route the cheap,
  parallelisable workers (file edits, test runs, doc updates) to a
  cheap model; reserve premium models for the small number of
  workers that need real reasoning.
- If every worker in a fleet would need a premium model, the work is
  probably not a good fit for fleet mode — reconsider the
  decomposition before paying N× premium.

### Session hygiene

- Keep sessions short and focused. Long premium sessions are the
  single largest source of waste because every turn re-processes the
  full history.
- Use `/compact` when the conversation grows long, and `/new` for
  unrelated work.
- Prefer `/ask` for one-off side questions so they don't extend the
  main session.

### When in doubt

Ask: *"If a cheaper model produced the wrong answer here, would I
catch it in seconds (compiler, tests, my own review) or in
weeks (production incident)?"* If the former, use the cheap model
and let the feedback loop do its job.
