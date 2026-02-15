# Agent Instructions

## Commit Messages

- Do not use the Conventional Commits format.
- Do not add any prefix in commit messages (for example: `feat:`, `fix:`, `chore:`).
- Write clear, descriptive commit messages in English.

## Binding API Design

- Binding APIs must be idiomatic for each target language and consistent within that language.

## Local Verification Before Commit

- When implementing work, add or update `mise` tasks so the change can be tested locally.
- Before committing, verify the change through the `pre-commit` hook and ensure it passes.
