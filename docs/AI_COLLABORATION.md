# AI Collaboration Workflow

ABYSS can be developed with multiple AI assistants, including Codex and Claude,
as long as GitHub remains the source of truth.

## Recommended Workflow

1. Keep `C:\ABYSS` as the main working repository.
2. Use Git branches for separate work:
   - `codex/...` for Codex changes;
   - `claude/...` for Claude changes;
   - `main` only for reviewed, stable work.
3. Ask each assistant to explain changed files before merging.
4. Prefer pull requests over direct pushes to `main`.
5. Do not let two assistants edit the same file at the same time unless the
   task is very small and clearly coordinated.

## How Claude Can Collaborate

Claude cannot be directly "connected" to Codex inside one live coding session,
but it can collaborate through shared artifacts:

- GitHub repository;
- branches;
- pull requests;
- issue/task list;
- markdown specs in `docs/`;
- exported code snippets or patches.

The safest pattern is:

1. Claude proposes or edits on a branch.
2. Codex reviews, tests, and integrates.
3. The final result is merged through Git.

## Shared Task Board

Use `docs/TASKS.md` as a simple shared board:

- `Backlog` - ideas not started.
- `In Progress` - active work.
- `Review` - needs testing/review.
- `Done` - completed and verified.

## Conflict Rules

- Do not rewrite history on shared branches.
- Do not force-push without explicit agreement.
- Do not delete folders until their contents are reviewed.
- Keep old project folders archived until the main repository contains every
  useful file or idea.

