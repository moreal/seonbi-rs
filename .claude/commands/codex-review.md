---
allowed-tools: Bash(codex review *), Bash(git log --oneline *), Bash(git diff --stat *), Bash(git rev-list *), Read, Task
---

# Codex Review

Run `codex review` on recent commits from multiple perspectives in parallel and report consolidated results.

## Arguments
- `$ARGUMENTS`: (Optional) Commit range, scope, or options. Examples:
  - `HEAD~3..HEAD` - review last 3 commits
  - `--base main` - review changes from main branch
  - `--uncommitted` - review uncommitted changes
  - `--all` - review the entire codebase
  - `--path src/foo` - review only files under the given path
  - If empty, auto-detect using `git merge-base` (see below).

## Steps

1. **Determine review scope**

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
   # List files under the target path for prompt context
   git diff --stat $RANGE -- $TARGET_PATH
   ```

   **Mode C – Commit-based (default)**:
   If `$ARGUMENTS` is empty, auto-detect the range:
   ```shell
   MERGE_BASE=$(git merge-base upstream/main HEAD 2>/dev/null || git merge-base upstream/master HEAD 2>/dev/null || git merge-base origin/main HEAD 2>/dev/null || git merge-base origin/master HEAD)
   RANGE="${MERGE_BASE}..HEAD"
   SCOPE_NOTE="커밋 범위: ${RANGE}"
   git log --oneline $RANGE
   git diff --stat $RANGE
   ```
   If `$ARGUMENTS` is provided (and is not `--all` or `--path`), use it directly.

2. **Run parallel reviews using Task tool**
   Launch the following `codex review` commands **in parallel** using the Task tool (subagent_type: Bash), each with a different review prompt.
   All tasks should run in the background so they execute concurrently.

   For **Mode A/B** (full codebase or path-specific), adjust prompts to review all code holistically rather than focusing on diffs.
   For **Mode B**, append `Focus ONLY on files under '${TARGET_PATH}'.` to each prompt.

   **Review 1 - Code Quality**:
   ```shell
   # Mode C (commit-based):
   codex review --commit <range> "Review code quality: clarity, readability, idiomatic Rust patterns, proper error handling, unnecessary complexity, and code duplication. Focus only on the changed code."
   # Mode A (--all):
   codex review --commit <range> "Review the entire codebase for code quality: clarity, readability, idiomatic Rust patterns, proper error handling, unnecessary complexity, and code duplication."
   # Mode B (--path):
   codex review --commit <range> "Review the codebase for code quality: clarity, readability, idiomatic Rust patterns, proper error handling, unnecessary complexity, and code duplication. Focus ONLY on files under '${TARGET_PATH}'."
   ```

   **Review 2 - Correctness & Original seonbi Compatibility**:
   ```shell
   # Mode C (commit-based):
   codex review --commit <range> "Review for correctness bugs and compatibility with the original Haskell seonbi implementation. The original source is in seonbi/ and the reference binary is at .tools/original/seonbi-0.5.0/seonbi. Check edge cases, off-by-one errors, and semantic differences from the original. Note: CLI flag-level differences due to optparse-applicative vs clap are expected and acceptable."
   # Mode A (--all):
   codex review --commit <range> "Review the entire codebase for correctness bugs and compatibility with the original Haskell seonbi implementation. The original source is in seonbi/ and the reference binary is at .tools/original/seonbi-0.5.0/seonbi. Check edge cases, off-by-one errors, and semantic differences from the original. Note: CLI flag-level differences due to optparse-applicative vs clap are expected and acceptable."
   # Mode B (--path):
   codex review --commit <range> "Review for correctness bugs and compatibility with the original Haskell seonbi implementation. The original source is in seonbi/ and the reference binary is at .tools/original/seonbi-0.5.0/seonbi. Check edge cases, off-by-one errors, and semantic differences from the original. Note: CLI flag-level differences due to optparse-applicative vs clap are expected and acceptable. Focus ONLY on files under '${TARGET_PATH}'."
   ```

   **Review 3 - Performance**:
   ```shell
   # Mode C (commit-based):
   codex review --commit <range> "Review for performance issues: unnecessary allocations, redundant computations, inefficient algorithms, hot path overhead, and missing optimizations. Consider both compile-time and runtime performance impact."
   # Mode A (--all):
   codex review --commit <range> "Review the entire codebase for performance issues: unnecessary allocations, redundant computations, inefficient algorithms, hot path overhead, and missing optimizations. Consider both compile-time and runtime performance impact."
   # Mode B (--path):
   codex review --commit <range> "Review for performance issues: unnecessary allocations, redundant computations, inefficient algorithms, hot path overhead, and missing optimizations. Consider both compile-time and runtime performance impact. Focus ONLY on files under '${TARGET_PATH}'."
   ```

3. **Collect and read all review outputs**
   - Wait for all background tasks to complete
   - Read each output file

4. **Report consolidated results to the user in Korean**
   Present results grouped by review perspective:

   ```
   ## 리뷰 범위
   (SCOPE_NOTE + commit range or path, files changed)

   ## 코드 품질
   (Review 1 findings)

   ## 정확성 & seonbi 호환성
   (Review 2 findings)

   ## 성능
   (Review 3 findings)

   ## 종합 판단
   (Overall verdict: safe to merge or not, with actionable items)
   ```

5. **If actionable fixes are suggested**
   - Ask the user if they want to apply the fixes
   - If yes, implement the changes and run relevant tests

## Notes
- Always report results in Korean
- All three reviews MUST run in parallel (use run_in_background for Task tool)
- If `codex` is not installed, inform the user to install it
- For commit-based reviews (Mode C), focus on regressions (things that got worse) over pre-existing issues
- For full codebase / path-specific reviews (Mode A/B), review holistically — there is no "pre-existing vs new" distinction
- Each codex review runs independently, so findings may overlap - deduplicate when reporting
- `--all` reviews may produce large diffs; if the diff is too large, suggest using `--path` to narrow the scope
