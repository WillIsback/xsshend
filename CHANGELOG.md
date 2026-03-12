# Changelog

All notable changes to xsshend are documented here.
Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)

## [0.6.0] — 2026-03-12

### Security
- **RUSTSEC-2023-0071 mitigation** — `russh` now explicitly uses `features = ["aws-lc-rs"]`.
  SSH crypto operations (key exchange, signatures) route through `aws-lc-rs` (constant-time RSA),
  not the vulnerable `rsa` crate. Residual risk: RSA key parsing via `internal-russh-forked-ssh-key`
  is local-only. **Recommendation: use Ed25519 keys** (`ssh-keygen -t ed25519`).
- `deny.toml`: explicit ban on `ring` crate, updated advisory analysis.

### Added
- **`grep` subcommand** — parallel grep across all nodes of a cluster:
  ```bash
  xsshend grep jdupont --log-path "/u01/oracle/wls/logs/*.log" \
    --env Production --type WebLogic --first-match
  ```
  - `--first-match` / `-C` context lines / `--output-format json`
  - `--first-match` uses `tokio::sync::watch` + `JoinSet::abort_all()` for clean short-circuit
- **GitHub Agentic Workflows** (CI bots):
  - `agent-ci-fix` — investigates CI failures, auto-fixes minor issues (fmt, trivial lints), opens issues for significant ones
  - `agent-issue-triage` — labels and responds to new issues, weekly repo health report
  - `agent-pr-review` — automated stability checklist on every PR to main

### Changed
- **Connection Pool** (`ssh/pool.rs`) — SSH connections are reused across parallel operations.
  `DashMap<host_key, Arc<Mutex<SshClient>>>` + `Semaphore(10)`. Saves N-1 handshakes per host group.
  5 files × 3 servers = 3 SSH connections instead of 15.
- `executor.rs` — refactored to use `ConnectionPool`; broken connections auto-invalidated.
- `uploader.rs` — refactored to use `ConnectionPool`; pool shared across all file transfers.
- CI workflows — all jobs now install `cmake clang nasm` (required by `aws-lc-rs`).
- `release.yml` — `cargo publish` wrapped in `continue-on-error: true` (token may be revoked).

### Dependencies
- Added: `dashmap = "6"`

## [0.5.2] — prior

See git log for earlier history.
