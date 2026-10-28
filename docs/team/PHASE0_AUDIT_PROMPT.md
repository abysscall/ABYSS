# Phase 0 — read-only baseline audit (prompt)

**How to use (owner):**

1. Commit the coordination pack first, so the working tree is clean.
2. Open the same repository in **two** agents (for example Codex and Kimi).
3. Paste everything inside the block below into each agent. Replace `<YOUR_NAME>` with `codex` or `kimi`.
4. Do **not** show one report to the other agent until both reports are saved.
5. Give both reports to Claude for comparison.

```text
ABYSS CORE — PHASE 0: READ-ONLY BASELINE AUDIT

Read AGENTS.md, docs/team/HANDOFF.md and docs/team/DECISION_LOG.md first.

DECISIONS IN FORCE
1. Approach A: reliable Core first.
2. Balanced-by-Design: privacy, security and performance each have a
   mandatory minimum. None may be reduced below its minimum to improve another.
3. No code changes are authorized at this stage.

CHANGE CONTROL
- Do not edit, create, delete, rename or format any project file, with ONE
  exception: create a single new file docs/audit/PHASE0_REPORT_<YOUR_NAME>.md
  containing your report. Create it last. Do not touch existing files.
- No patches, generators or dependency changes. Do not switch branches or run
  reset, rebase, merge, cherry-pick, stash, clean, fetch, pull or push.
- Allowed diagnostics (they write only to the git-ignored target/ directory):
    cargo metadata --locked
    cargo test --workspace --locked
    cargo clippy --workspace --all-targets --locked -- -D warnings
    cargo fmt --all -- --check
  First confirm that target/ is ignored: git check-ignore target
  Never run cargo without --locked.
- Run `git status --porcelain` at the start and at the end and include both
  outputs. If tests create untracked files (for example .abyss/), report them
  and do not delete them. The only allowed difference is your report file.
- Do not claim a test passed unless you ran it. Paste the summary line of
  every command you ran. If you cannot run commands, say so.
- Do not assume historical reports describe this checkout.
- Do not read, print or request secrets or environment variables.

EVIDENCE RULES
- Every finding needs path, function and line numbers.
- For every finding rated High or Critical, quote the relevant code (at most
  15 lines) so a reviewer without repository access can check it.
- Give each component exactly one status: Implemented, Partial, Concept only
  or Unknown. Keep separate: source evidence, test evidence, documentation
  claims, unverified assumptions.

TASK 1 — REPOSITORY STATE
Directory, branch, HEAD commit, working-tree status (staged, unstaged,
untracked), local and remote-tracking branches, in-progress work to preserve.
Do not fetch, pull or switch branches.

TASK 2 — ARCHITECTURE MAP
Workspace members, modules, entry points, dependency relationships, docs.
Locate at least: state and balances; transaction format and validation;
hashing and crypto interfaces; consensus, block validation, finalization;
commit and persistence; wallet key generation, storage and signing;
existing tests and CI.

TASK 3 — BLOCK LIFECYCLE
Trace: block input -> validation -> consensus -> finalization ->
state transition -> persistence -> restart and recovery. Cite the exact file
and function at every step. Do not infer behaviour from names alone. Report
missing calls, incomplete implementations, unclear invariants and possible
state divergence.

TASK 4 — EVIDENCE CLASSIFICATION
Classify each major component and separate the four kinds of evidence.

TASK 5 — CRITICAL RISKS (preliminary triage)
1. Invalid block acceptance or inconsistent chain and state.
2. Unsigned or improperly authorized transactions.
3. Replay, nonce, overflow and double-spend.
4. Non-atomic state updates; persistence and recovery failures.
5. Wallet key exposure or incorrect signing.
6. Privacy leaks and resource exhaustion.

TASK 6 — CONSISTENCY
Compare AC supply and allocation numbers in code, data/tokenomics.json and
the documents (README, constitution, docs/tokenomics, ADRs). Give a table of
discrepancies. Do not choose a value.

TASK 7 — SUSPECTED DEFECT TO PROVE OR REFUTE
Claim: "chain height advances but the state is not applied". Find where the
height is updated and where state transitions are applied. State whether the
two can diverge, with code evidence. If you cannot decide, answer Unknown.

REPORT FORMAT
Start with a 15-line summary in Russian for the owner. Then the English report:
A. Verified repository baseline
B. Workspace and module map
C. Block lifecycle trace
D. Findings with file, function, line, severity
E. Tests present, and which ones you actually ran
F. Unknowns that need more inspection
G. A minimal Phase 1 audit plan
H. Files likely to change in a later, separately authorized phase

STOP after the report is written and wait for explicit approval.
```
