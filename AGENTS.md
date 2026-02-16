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

## Local Verification Before Commit

- When implementing work, add or update `mise` tasks so the change can be tested locally.
- Before committing, verify the change through the `pre-commit` hook and ensure it passes.
