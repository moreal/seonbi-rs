# Agent Instructions

## Commit Messages

- Do not use the Conventional Commits format.
- Do not add any prefix in commit messages (for example: `feat:`, `fix:`, `chore:`).
- Write clear, descriptive commit messages in English.

## Binding API Design

- Binding APIs must be idiomatic for each target language and consistent within that language.

## Compatibility with Original seonbi

- This project is a Rust port of the Haskell [seonbi](seonbi/) tool (original binary at `.tools/original/seonbi-0.5.0/`).
- The core library (`crates/seonbi`) must produce identical output to the original seonbi for the same input and configuration.
- CLI (`crates/seonbi-cli`) cannot achieve full flag-level compatibility with the original because the original uses Haskell's `optparse-applicative` while the Rust port uses `clap`. Differences in flag parsing behavior (e.g., option grouping, error messages) are expected and acceptable.
- When reviewing for compatibility, compare against the original Haskell source in `seonbi/` and the reference binary in `.tools/original/seonbi-0.5.0/seonbi`.

## Testing Policy

- E2E tests are the source of truth. When adding regression tests for bug fixes, always write E2E tests first.
- Unit tests may be added as supplementary tests, but E2E tests take priority.
- E2E comparison tasks are available via `mise`: `mise run e2e-cli-original`, `mise run e2e-api-original`, `mise run e2e-original-all`.
- Required E2E environment variables are provided by default in `mise.toml` (`SEONBI_ORIGINAL_BIN`, `SEONBI_ORIGINAL_API_BIN`, `SEONBI_ORIGINAL_API_URL`).

## Local Verification Before Commit

- When implementing work, add or update `mise` tasks so the change can be tested locally.
- Before committing, verify the change through the `pre-commit` hook and ensure it passes.

## Commit Workflow

- Each task must be committed separately.
- Before committing, run `codex-review --uncommitted` and address all actionable findings.
- Repeat until no actionable findings remain, then commit.
