---
on:
  issues:
    types: [opened, reopened]
  schedule:
    - cron: '0 8 * * 1'  # Lundi 8h UTC — bilan hebdomadaire

permissions:
  contents: read
  issues: write
  pull-requests: read

safe-outputs:
  create-comment:
    body-prefix: "🤖 **xsshend-bot** — Issue Triage\n\n"
  add-labels:
    allowed: [bug, enhancement, question, documentation, security, performance, needs-info, duplicate, wontfix, good-first-issue]

tools:
  github:
---

# Issue Triage & Weekly Health Report

## When triggered by a new issue

1. **Read** the issue title and body carefully
2. **Classify** it into one of:
   - `bug` — something doesn't work as expected
   - `enhancement` — new feature request
   - `question` — usage question
   - `documentation` — doc improvement
   - `security` — security concern (handle with care, do NOT expose details publicly)
   - `performance` — performance degradation
   - `needs-info` — insufficient information to act
3. **Add the appropriate label(s)**
4. **Leave a comment** that:
   - Acknowledges the issue
   - Asks for any missing information (reproduction steps, OS, xsshend version, SSH key type)
   - For bugs: ask for `xsshend --verbose` output and `cargo --version`
   - For security issues: acknowledge privately and ask them to see SECURITY.md
   - For questions: point to the relevant doc section if applicable

## When triggered on schedule (Monday weekly report)

Create a weekly status issue with:
- Open issues summary (count by label)
- PRs pending review
- Any security advisories in the last 7 days (check rustsec)
- Highlight: issues with no response in 7+ days

## Project context

xsshend is a Rust CLI tool for parallel SSH file upload and command execution.
- Version: 0.6.0
- Key features: upload, command, grep (new in 0.6.0)
- Common issues: SSH key compatibility, hosts.json format, SFTP path expansion
- Known limitation: RUSTSEC-2023-0071 (Marvin Attack) — documented, mitigated via aws-lc-rs backend
- Docs: https://willisback.github.io/xsshend/
