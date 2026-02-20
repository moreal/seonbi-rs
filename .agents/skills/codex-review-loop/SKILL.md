---
name: codex-review-loop
description: Run iterative codex review cycles (review, fix, verify, re-review) until no actionable findings remain or max rounds is reached.
---

# Codex Review Loop

Use this skill when the user asks for `$codex-review-loop` or requests iterative review-and-fix automation.

## Purpose

Repeat this cycle up to 5 rounds:

1. Run three parallel `codex review` passes
2. Apply actionable fixes
3. Verify with formatting/tests
4. Re-review

Stop when no actionable issues remain or round limit is reached.

## Preconditions

- Run from repository root.
- Confirm `codex` CLI exists:

```bash
command -v codex >/dev/null || { echo "codex CLI is required."; exit 1; }
```

## Arguments

- `$ARGUMENTS` is optional and follows the same scope rules as `$codex-review`:
  - `--uncommitted`
  - `--base <branch-or-sha>`
  - `--all`
  - `--path <path>`
  - `<single-commit-sha>`
  - `<left>..<right>`
  - empty (auto merge-base)

## Configuration

- `MAX_ROUNDS=5`
- Review perspectives per round:
  - Code quality
  - Correctness and original seonbi compatibility
  - Performance

## Round Workflow

For round `N` from `1` to `5`:

1. Print round banner:

```text
## Round N / 5
```

2. Resolve effective scope:
   - Round 1 uses requested scope.
   - Rounds 2+ default to `--commit HEAD` if a fix commit was made in the previous round.
   - If no commit was made, keep prior scope.

3. Run three parallel reviews (same perspective prompts as `$codex-review`), each with:
   - `Respond with LGTM if no actionable issues are found.`

4. Collect and classify results:
   - actionable
   - non-actionable (style-only, out-of-scope, pre-existing noise for commit-scoped review)

5. Termination checks:
   - If all three reviews are LGTM or no actionable findings exist, stop successfully.
   - If `N == 5`, stop and report remaining items.

6. Apply fixes for actionable findings:
   - Prefer smallest safe patch set.
   - Skip purely stylistic suggestions unless they materially improve correctness/performance.
   - Keep compatibility requirements from `AGENTS.md` in scope.

7. Verification:
   - Run `cargo fmt --all` if Rust files changed.
   - Run targeted tests for affected crates.
   - If the project has dedicated `mise` tasks for changed areas, run those tasks too.

8. Commit fixes:

```bash
git add <changed-files>
git commit -m "Address codex review round N feedback"
```

9. Continue to the next round.

## Final Report Format (Korean)

```text
## Codex Review Loop 결과
- 총 라운드: N / 5
- 종료 사유: (모든 리뷰 통과 | 최대 라운드 도달)

### 라운드별 요약
| 라운드 | 코드 품질 | seonbi 호환성 | 성능 | 수정 사항 |
|--------|----------|---------------|------|----------|
| 1      | ...      | ...           | ...  | ...      |

### 남은 이슈
- ...

### 최종 판단
- ...
```

## Notes

- Run the three perspective reviews in parallel each round.
- If a proposed fix introduces regressions, revert that fix and mark it unresolved with rationale.
- For commit-based review scope, prioritize regressions caused by reviewed commits.
- For `--all` and `--path`, review holistically (no pre-existing/new split).
