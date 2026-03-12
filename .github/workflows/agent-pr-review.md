---
on:
  pull_request:
    types: [opened, synchronize, ready_for_review]
    branches: [main]

permissions:
  contents: read
  pull-requests: write
  issues: read

safe-outputs:
  create-comment:
    body-prefix: "🤖 **xsshend-bot** — PR Review\n\n"

tools:
  github:
  filesystem:
---

# Automated PR Review — Stability Gate

Review all PRs targeting `main` before a human merges them.

## What to check

1. **Version bump** — Is Cargo.toml version bumped? Does it match main.rs version? Are they consistent with the expected semver (patch/minor/major)?

2. **CHANGELOG** — Is there a CHANGELOG.md entry for this version?

3. **Security** — Does the PR introduce new dependencies? If yes, are they from reputable crates? Does `deny.toml` need updating?

4. **Breaking changes** — Do any public API or CLI argument changes require a major version bump?

5. **Test coverage** — Are new modules covered by `#[cfg(test)]` blocks? New public functions should have at least one unit test.

6. **Build dependencies** — If `Cargo.toml` adds a crate that needs native libs (like `aws-lc-rs` → cmake/clang, or `openssl`), are the CI workflows updated with `apt-get install`?

7. **Documentation** — Are new public items documented with `///` doc comments?

## What to output

Write a review comment with:
- ✅ / ❌ for each check above
- Overall verdict: `READY TO MERGE` or `NEEDS CHANGES` + specific action items
- Be concise — max 20 lines

Do NOT approve or request changes via the GitHub review system — comment only.
Humans make the final merge decision.
