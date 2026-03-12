---
on:
  workflow_run:
    workflows: ["Pre-commit Quality Gate", "Tests and Quality Checks"]
    types: [completed]
    branches-ignore: [main, gh-pages]

permissions:
  contents: read
  issues: write
  pull-requests: write
  checks: read
  actions: read

safe-outputs:
  create-comment:
    body-prefix: "🤖 **xsshend-bot** — CI Analysis\n\n"
  create-pull-request:
    title-prefix: "fix(bot): "
    branch-prefix: "bot/autofix-"
    labels: [bot, automated-fix, needs-review]

tools:
  github:
  filesystem:
---

# CI Failure Investigation & Auto-Fix

Analyse the failed CI workflow and help fix the issues.

## Context

This workflow runs when a CI pipeline fails on a PR or feature branch. Your job is to:

1. **Investigate** — Read the failed workflow logs and identify the root cause
2. **Classify** — Determine if the failure is:
   - A **minor/auto-fixable issue**: formatting, trivial clippy lint (unused import, missing derive, etc.), doc typo
   - A **significant issue**: logic bug, compilation error, test failure, security problem
3. **Act** based on classification:
   - For **minor issues**: open a PR with the fix on the same branch
   - For **significant issues**: open a GitHub issue with full analysis and suggested fix

## Rules

- NEVER push directly to `main`
- NEVER merge PRs — humans must review
- For auto-fix PRs: keep changes minimal and targeted, one fix per PR
- Always explain what you found and what you did (or why you did nothing)
- If the failure is a flaky test (network, timeout, non-deterministic), note it in a comment but do NOT open an issue
- If you cannot determine the cause after reading logs, say so clearly

## What counts as "minor/auto-fixable"

- `cargo fmt` formatting differences
- Clippy: unused imports, missing `#[allow(...)]`, trivial type annotations
- Missing trailing newline in a file
- Obvious doc comment typo (single word)

## What counts as "significant"

- Any compilation error
- Any test assertion failure
- Clippy deny warnings that indicate real logic issues
- Security audit failures (new RUSTSEC advisory)
- Version inconsistencies between Cargo.toml and main.rs
