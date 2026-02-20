---
name: codex-review
description: Run codex review from three perspectives in parallel and report a consolidated Korean summary for uncommitted changes, base comparisons, paths, or commit ranges.
---

# Codex Review

Use this skill when the user asks for `$codex-review` or asks for a multi-perspective Codex review.

## Purpose

Run three independent `codex review` passes in parallel:

1. Code quality
2. Correctness and compatibility with original Haskell seonbi
3. Performance

Then merge and deduplicate findings into one Korean report.

## Preconditions

- Run from repository root.
- Verify CLI availability before running reviews:

```bash
command -v codex >/dev/null || { echo "codex CLI is required."; exit 1; }
```

## Arguments

- `$ARGUMENTS` is optional.
- Supported forms:
  - `--uncommitted`
  - `--base <branch-or-sha>`
  - `--all`
  - `--path <path>`
  - `<single-commit-sha>`
  - `<left>..<right>` commit range
- If empty: auto-detect merge-base from `upstream/main`, `upstream/master`, `origin/main`, `origin/master`.

## 1) Determine Scope

Use these rules:

1. If `$ARGUMENTS` is `--uncommitted`:
   - `SCOPE_MODE=uncommitted`
   - `SCOPE_NOTE="작업 트리 변경사항"`
2. If `$ARGUMENTS` starts with `--base `:
   - `SCOPE_MODE=base`
   - `SCOPE_VALUE=<base>`
   - `SCOPE_NOTE="base 비교: <base>"`
3. If `$ARGUMENTS` is `--all`:
   - `ROOT=$(git rev-list --max-parents=0 HEAD | head -1)`
   - `SCOPE_MODE=base`
   - `SCOPE_VALUE="$ROOT"`
   - `SCOPE_NOTE="전체 코드베이스(최초 커밋 기준)"`
4. If `$ARGUMENTS` starts with `--path `:
   - `TARGET_PATH=<path>`
   - `MERGE_BASE=$(git merge-base upstream/main HEAD 2>/dev/null || git merge-base upstream/master HEAD 2>/dev/null || git merge-base origin/main HEAD 2>/dev/null || git merge-base origin/master HEAD)`
   - `SCOPE_MODE=base`
   - `SCOPE_VALUE="$MERGE_BASE"`
   - `SCOPE_NOTE="경로 제한: <path> (base: $MERGE_BASE)"`
5. If `$ARGUMENTS` contains `..`:
   - `SCOPE_MODE=range`
   - `SCOPE_VALUE="$ARGUMENTS"`
   - `SCOPE_NOTE="커밋 범위: $ARGUMENTS"`
6. If `$ARGUMENTS` is a non-empty single token:
   - `SCOPE_MODE=commit`
   - `SCOPE_VALUE="$ARGUMENTS"`
   - `SCOPE_NOTE="단일 커밋: $ARGUMENTS"`
7. If empty:
   - same merge-base logic as path mode
   - `SCOPE_MODE=base`
   - `SCOPE_VALUE="$MERGE_BASE"`
   - `SCOPE_NOTE="자동 감지 커밋 범위: ${MERGE_BASE}..HEAD"`

For any mode except `uncommitted`, also print:

```bash
git log --oneline "${SCOPE_VALUE}..HEAD" 2>/dev/null || true
git diff --stat "${SCOPE_VALUE}..HEAD" 2>/dev/null || true
```

## 2) Prepare Perspective Prompts

Base prompts:

1. Code quality:
   - `Review code quality: clarity, readability, idiomatic Rust patterns, error handling, unnecessary complexity, and duplication.`
2. Correctness and compatibility:
   - `Review correctness and compatibility with original Haskell seonbi. Original source is in seonbi/ and reference binary is .tools/original/seonbi-0.5.0/seonbi. Focus on semantic differences and regressions. CLI flag-level parser differences (optparse-applicative vs clap) are acceptable.`
3. Performance:
   - `Review for performance issues: unnecessary allocations, redundant computation, inefficient algorithms, hot-path overhead, and optimization opportunities.`

Path mode addendum:

- Append `Focus ONLY on files under '<path>'.` to all three prompts.

## 3) Run Reviews in Parallel

Use temporary output files and launch all three reviews in parallel.

### Modes `uncommitted`, `base`, `commit`

Command shape:

```bash
# uncommitted
codex review --uncommitted "<PROMPT>"

# base
codex review --base "$SCOPE_VALUE" "<PROMPT>"

# commit
codex review --commit "$SCOPE_VALUE" "<PROMPT>"
```

### Mode `range` (`left..right`)

`codex review` works on one commit/base at a time. For explicit ranges, expand commits and review each commit:

```bash
COMMITS=$(git rev-list --reverse "$SCOPE_VALUE")
```

For each perspective, run:

```bash
for c in $COMMITS; do
  codex review --commit "$c" "<PROMPT>"
done
```

Run the three perspective loops in parallel and aggregate outputs.

## 4) Consolidate Findings

- Classify findings into:
  - actionable
  - non-actionable/informational
- Deduplicate overlaps across perspectives.
- For commit-scoped reviews, emphasize regressions over pre-existing issues.

## 5) Report Format (Korean)

Use this structure:

```text
## 리뷰 범위
- 모드:
- 범위:
- 참고 로그/변경 통계:

## 코드 품질
- ...

## 정확성 & seonbi 호환성
- ...

## 성능
- ...

## 종합 판단
- 머지 가능 여부
- 우선순위 높은 조치 항목
```

## Notes

- If `codex` is missing, report installation is required.
- Keep findings concrete with file paths and actionable suggestions.
- If the user asks to apply fixes, switch from review-only to implementation workflow after reporting.
