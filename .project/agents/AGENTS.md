# AI Agent Instructions

Version: 0.11.0

Read this file before changing this repository. Repo-specific rules are in [./repository.md](./repository.md).

## File Index

Read in order before starting: `AGENTS.md` → `../tasks/todo.md` → `./repository.md`

- `./repository.md`: Repo-specific rules.
- `./ai-notes.md`: Cross-task context & decisions.
- `../tasks/todo.md`: Active task index.
- `../tasks/` & `../tasks/archive/`: Active and archived task files.
- `../tasks/template.md`: Template for new task files.
- `../../CHANGELOG.md`: Notable approved changes.

## Task Statuses (`../tasks/todo.md`)

- **TODO**: Not ready — do not start.
- **PLAN**: AI writes plan, then sets to `REVIEW`.
- **BUILD**: AI starts build (plan approved), then sets to `REVIEW`.
- **NEXT**: Ready for AI — read task file for instructions, then set to `REVIEW`.
- **CONTINUE**: Resume using user's review comments, then set to `REVIEW`.
- **REVIEW**: Done by AI, awaiting user review.
- **CANCEL**: Abandoned — do not work on these.
- **DONE**: Completed and approved.

## General Rules

- **Language:** Match user's language in chat. English for code, comments, docs, commits, and files.
- **Model Suitability:** Assess if the current model is appropriate. Pause and ask the user if a stronger model is needed for complexity/safety, or if a cheaper/faster model suffices.
- **Commits:** Small, focused changes only. No unrelated refactoring. Do not commit, amend, squash, or merge unless explicitly asked.
- **When unsure:** Do not invent assumptions. Record uncertainty in the task file (or `ai-notes.md` for repo-wide concerns) and ask the user.
- **Protected changes:** Never modify without explicit instruction: secrets, `.env`, production configs, deployment credentials, or dependencies.
- **Workflow files:** Never modify `AGENTS.md` or the task template.

## Workflow

### 1. Prepare

1. Only work on `PLAN`, `BUILD`, `NEXT`, or `CONTINUE` tasks.
2. Create or reuse `../tasks/YYYY-MM-DD--slug.md` from the template. Remove inapplicable sections.
3. Add/update `task_started: YYYY-MM-DD Day HH:MM` at the top.
4. Link the file in `todo.md` below the task heading (relative link, e.g. `[2026-05-24--fix](./2026-05-24--fix.md)`).
5. **Branching:** If on `main`, create a branch `type/description` (`feat, fix, hotfix, refactor, perf, docs, test, release, ci, chore`). If not on `main`, report the unexpected state.

### 2. PLAN Mode (status: `PLAN`)

1. Do **not** modify any code.
2. Write a `* Planning` section in the task file including: date, problem summary, context, goals/non-goals, architecture, step-by-step plan, affected areas, risks, assumptions, open questions, proposed tests.
3. Set task to `REVIEW` and notify the user. **Stop.** User sets to `BUILD` to approve or `CONTINUE` if revisions are needed.

### 3. BUILD Mode (status: `BUILD`)

1. Document the intended approach briefly in the task file.
2. Implement changes. Update relevant docs/README. Run tests and linters. Update `CHANGELOG.md` if notable.
3. Write a `* Build` section in the task file: date, status, summary, changed files, checks performed, test results, blockers, open questions.
4. Set task to `REVIEW` and notify the user.
5. **After user approval:** Set task to `DONE`, update the heading to include the completion date (e.g. `# DONE YYYY-MM-DD HH:MM <title>`), add/update `task_completed: YYYY-MM-DD HH:MM`, move task file to `../tasks/archive/`, update the link in `todo.md`, and move the entry under `# Completed`.
   - Set the task to `DONE`.
   - Update the task heading in `todo.org` to include the completion date, e. g. `# DONE YYYY-MM-DD HH:MM <title>`.
   - Add `task_completed: YYYY-MM-DD HH:MM` near the top of the task file.
   - Move the full task description/body from the `todo.md` entry into the task file under a `# Original Task` section, if it is not already preserved there. In `todo.md`, keep only the completed task heading and the link to the archived task file. Do not keep the full task body in `todo.md`.
   - Move the task file to `../tasks/archive/`.
   - Update the task link in `todo.md` so it points to the archived task file.

   **If not approved:** set to `CONTINUE`, address review comments, repeat from step 2.
