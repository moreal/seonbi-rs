---
name: e2e-fuzz-original-fix-loop
description: "Run a deterministic fix loop for CLI original-vs-ported fuzz E2E mismatches by executing `mise run e2e-cli-original-fuzz`, collecting failing cases from `crates/seonbi-cli/tests/fixtures/fuzz_regressions.jsonl`, analyzing root causes against the original Haskell implementation in `seonbi/`, patching Rust core implementation in `crates/seonbi`, and iterating until recorded fuzz regressions pass."
---

# E2E Fuzz Original Fix Loop

Use this skill when the user asks to repeatedly fix Rust behavior to match original seonbi based on failing `e2e-cli-original-fuzz` cases.

## Goal

Drive a strict compatibility loop:

1. Run fuzz E2E.
2. Extract and cluster failing cases.
3. Reproduce minimal differences.
4. Find root cause in Haskell source.
5. Patch `crates/seonbi` (not test-only masking).
6. Re-run loop until recorded cases pass.

## Preconditions

- Run from repository root.
- Ensure original binaries are configured via `mise.toml` env:
  - `SEONBI_ORIGINAL_BIN`
- Keep `SEONBI_E2E_FUZZ_SEED` fixed while debugging.
- Use non-destructive git workflow; do not reset unrelated changes.

## Loop Procedure

1. Run fuzz E2E.

```bash
mise run e2e-cli-original-fuzz
```

2. If failing, inspect recorded cases.

```bash
wc -l crates/seonbi-cli/tests/fixtures/fuzz_regressions.jsonl
cat crates/seonbi-cli/tests/fixtures/fuzz_regressions.jsonl
```

3. Cluster mismatch patterns (root-cause hints).

```bash
jq -r '
  def norm: gsub("쑨원";"손문");
  if ((.original_stdout|norm)==.ported_stdout) then "reading-only" else "other" end
' crates/seonbi-cli/tests/fixtures/fuzz_regressions.jsonl | sort | uniq -c
```

4. Reproduce one representative failure with exact args and input from JSONL.
- Compare original vs ported stdout/stderr byte-equivalently.
- Reduce to the smallest input that still reproduces the mismatch.

5. Trace root cause in both implementations.
- Rust target: `crates/seonbi/src/` only.
- Haskell reference: `seonbi/src/Text/Seonbi/*.hs`.
- Verify behavior expectation from Haskell code before patching.

6. Apply minimal Rust fix.
- Preserve existing semantics.
- Prefer localized changes in transformation logic or data loading.
- Do not "fix" by weakening e2e assertions or skipping failing cases.

7. Verify.

```bash
cargo fmt --all
cargo test -p seonbi-cli --test e2e_compare --no-run
mise run e2e-cli-original-fuzz
```

8. Repeat from step 1 until:
- `compare_recorded_fuzz_regressions_with_original` passes, and
- `compare_fuzz_matrix_with_original` adds no new mismatches for the chosen seed/iters.

## Guardrails

- Keep compatibility as highest priority over style refactors.
- Edit `crates/seonbi` for behavioral fixes; avoid patching only CLI tests.
- If failures come from missing large dictionary assets (for example Git LFS pointer files), document it explicitly and add minimal compatibility fallback in core where justified.
- Keep each logical fix as a separate commit when requested.

## Reporting Format

Report each loop round with:

1. failing case count
2. mismatch clusters
3. root cause hypothesis and evidence (file and line references)
4. Rust changes made
5. verification commands and outcomes
6. residual risk
