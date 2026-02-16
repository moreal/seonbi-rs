---
allowed-tools: Bash(codex review *), Bash(git log *), Bash(git diff *), Bash(git rev-list *), Bash(git add *), Bash(git commit *), Bash(cargo fmt *), Bash(cargo test *), Read, Grep, Glob, Edit, Task
---

# Codex Review Loop

Run iterative codex review cycles: review → fix → re-review, until no actionable findings remain or max rounds reached.

## Arguments
- `$ARGUMENTS`: (Optional) Commit range, scope, or options. Examples:
  - `HEAD~3..HEAD` - review last 3 commits
  - `--base main` - review changes from main branch
  - `--uncommitted` - review uncommitted changes
  - `--all` - review the entire codebase
  - `--path src/foo` - review only files under the given path
  - If empty, auto-detect using `git merge-base` (see below).

## Configuration
- **Max rounds**: 5 (to prevent runaway loops)
- **Review perspectives**: Code quality, Original seonbi compatibility, Performance (run in parallel each round)

## Steps

### 0. Determine review range and mode

**Mode A – Full codebase (`--all`)**:
```shell
ROOT=$(git rev-list --max-parents=0 HEAD | head -1)
RANGE="${ROOT}..HEAD"
SCOPE_NOTE="전체 코드베이스"
```

**Mode B – Path-specific (`--path <path>`)**:
```shell
ROOT=$(git rev-list --max-parents=0 HEAD | head -1)
RANGE="${ROOT}..HEAD"
TARGET_PATH="<path>"
SCOPE_NOTE="경로: ${TARGET_PATH}"
git diff --stat $RANGE -- $TARGET_PATH
```

**Mode C – Commit-based (default)**:
If `$ARGUMENTS` is empty, auto-detect the range:
```shell
MERGE_BASE=$(git merge-base upstream/main HEAD 2>/dev/null || git merge-base upstream/master HEAD 2>/dev/null || git merge-base origin/main HEAD 2>/dev/null || git merge-base origin/master HEAD)
RANGE="${MERGE_BASE}..HEAD"
SCOPE_NOTE="커밋 범위: ${RANGE}"
```
Then verify the detected range makes sense:
```shell
git log --oneline $RANGE
```
If `$ARGUMENTS` is provided (and is not `--all` or `--path`), use it directly as the `--commit` value or flags.

### Round Loop (repeat up to 5 times)

For each round N (1 to 5):

1. **Announce round**
   Print: `## Round N / 5`

2. **Run 3 parallel codex reviews** (all in background using Task tool)

   Select the appropriate prompt variant based on the mode determined in step 0.
   For **Mode B**, append `Focus ONLY on files under '${TARGET_PATH}'.` to each prompt.

   **Review 1 - Code Quality**:
   ```shell
   # Mode C (commit-based):
   codex review --commit <range> "Review code quality: clarity, readability, idiomatic Rust patterns, proper error handling, unnecessary complexity, and code duplication. Focus only on the changed code. Respond with LGTM if no issues found."
   # Mode A (--all):
   codex review --commit <range> "Review the entire codebase for code quality: clarity, readability, idiomatic Rust patterns, proper error handling, unnecessary complexity, and code duplication. Respond with LGTM if no issues found."
   # Mode B (--path):
   codex review --commit <range> "Review the codebase for code quality: clarity, readability, idiomatic Rust patterns, proper error handling, unnecessary complexity, and code duplication. Focus ONLY on files under '${TARGET_PATH}'. Respond with LGTM if no issues found."
   ```

   **Review 2 - Correctness & Original seonbi Compatibility**:
   ```shell
   # Mode C (commit-based):
   codex review --commit <range> "Review for correctness bugs and compatibility with the original Haskell seonbi implementation. The original source is in seonbi/ and the reference binary is at .tools/original/seonbi-0.5.0/seonbi. Check edge cases, off-by-one errors, and semantic differences from the original. Note: CLI flag-level differences due to optparse-applicative vs clap are expected and acceptable. Respond with LGTM if no issues found."
   # Mode A (--all):
   codex review --commit <range> "Review the entire codebase for correctness bugs and compatibility with the original Haskell seonbi implementation. The original source is in seonbi/ and the reference binary is at .tools/original/seonbi-0.5.0/seonbi. Check edge cases, off-by-one errors, and semantic differences from the original. Note: CLI flag-level differences due to optparse-applicative vs clap are expected and acceptable. Respond with LGTM if no issues found."
   # Mode B (--path):
   codex review --commit <range> "Review for correctness bugs and compatibility with the original Haskell seonbi implementation. The original source is in seonbi/ and the reference binary is at .tools/original/seonbi-0.5.0/seonbi. Check edge cases, off-by-one errors, and semantic differences from the original. Note: CLI flag-level differences due to optparse-applicative vs clap are expected and acceptable. Focus ONLY on files under '${TARGET_PATH}'. Respond with LGTM if no issues found."
   ```

   **Review 3 - Performance**:
   ```shell
   # Mode C (commit-based):
   codex review --commit <range> "Review for performance issues: unnecessary allocations, redundant computations, inefficient algorithms, hot path overhead, and missing optimizations. Respond with LGTM if no issues found."
   # Mode A (--all):
   codex review --commit <range> "Review the entire codebase for performance issues: unnecessary allocations, redundant computations, inefficient algorithms, hot path overhead, and missing optimizations. Consider both compile-time and runtime performance impact. Respond with LGTM if no issues found."
   # Mode B (--path):
   codex review --commit <range> "Review for performance issues: unnecessary allocations, redundant computations, inefficient algorithms, hot path overhead, and missing optimizations. Focus ONLY on files under '${TARGET_PATH}'. Respond with LGTM if no issues found."
   ```

3. **Collect and analyze all review outputs**
   - Read each output file
   - Classify findings as:
     - **Actionable**: Concrete bugs or improvements that can be fixed in code
     - **Non-actionable**: Style opinions, pre-existing issues, or informational notes
   - Print summary of findings for this round

4. **Check termination conditions**
   - If ALL three reviews report no actionable issues (LGTM or equivalent): **STOP** → go to Final Report
   - If round N equals 5: **STOP** → go to Final Report with note about remaining issues

5. **Apply fixes**
   - Fix all actionable issues found in this round
   - Run `cargo fmt --all` if Rust files were modified
   - Run `cargo test -p <affected-crate>` to verify fixes don't break anything
   - Amend the relevant commit or create a fixup commit:
     ```shell
     git add <changed-files>
     git commit -m "Address codex review round N feedback"
     ```
   - Update the commit range for the next round to include the new fix commit

6. **Continue to next round** with the updated commit range

### Final Report

After the loop ends, print a consolidated summary in Korean:

```
## Codex Review Loop 결과

- **총 라운드**: N / 5
- **종료 사유**: (모든 리뷰 통과 / 최대 라운드 도달)

### 라운드별 요약
| 라운드 | 코드 품질 | seonbi 호환성 | 성능 | 수정 사항 |
|--------|----------|---------------|------|----------|
| 1      | ...      | ...           | ...  | ...      |
| 2      | ...      | ...           | ...  | ...      |

### 남은 이슈 (있는 경우)
- (Non-actionable items or items that couldn't be resolved)

### 최종 판단
(Safe to merge / Needs manual review for remaining items)
```

## Notes
- Always report results in Korean
- All three reviews in each round MUST run in parallel
- If a fix introduces a test failure, revert the fix and report it as non-actionable
- For commit-based reviews (Mode C), do NOT fix pre-existing issues unrelated to the reviewed commits
- For full codebase / path-specific reviews (Mode A/B), review holistically — there is no "pre-existing vs new" distinction
- Do NOT apply purely stylistic suggestions that don't improve correctness or performance
- If `codex` is not installed, inform the user to install it
- `--all` reviews may produce large diffs; if the diff is too large, suggest using `--path` to narrow the scope
