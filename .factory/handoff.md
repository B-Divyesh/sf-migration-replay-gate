# Migration Replay Gate — build handoff

Work order: `migration-replay-gate-build-1`

Version: `0.1.0`

Completed: 2026-08-27

## What shipped

- A Rust `mrg` single-binary CLI built with clap. `mrg gate` starts one labeled disposable Postgres container through Docker or Podman, creates isolated clean and partial databases, seeds repeatable baseline/partial SQL fixtures, and runs the user’s real migration command in clean, repeat, and partial-state scenarios.
- Runtime safeguards: no database URL argument, rejection of embedded Postgres URLs/targets in the command, replacement of common ambient database variables, a loopback-only published Postgres port, explicit opt-in for destructive fixture SQL, bounded command/startup timeouts, output credential redaction, and container cleanup on normal completion/error paths.
- Behavioral reporting: captures command output and Postgres DDL logs; classifies command failures, non-idempotent repeats, partial-state failures, and destructive DDL. Human and stable JSON output are available. Exit codes are 0 safe, 2 unsafe, 3 invalid input, and 4 runtime failure.
- Known-outcome tests for clean, duplicate-table, and destructive changes, plus CLI safety/exit-code tests and an optional real-container smoke script at `scripts/smoke-docker.sh`.
- A Vite static docs site in `dist/site`, including the CLI reference, an interactive recorded replay with empty/loading/safe/blocked states, keyboard tabs, responsive 390 px layout, clipboard feedback, offline status/fallback, privacy and terms pages, cache headers, and a service worker.
- A product-specific luminous glass system documented in `.factory/design.md`. The original hero was generated with `/opt/fleet/lib/gen-image.sh` (`factory-image`) and optimized to responsive WebP assets (13 KB mobile, 26 KB tablet, 71 KB desktop). The prompt and provenance are recorded in the design file.

## Build and verification

From a clean clone:

```sh
npm ci
npm test
npm run build
cargo build --release --workspace
cargo package -p migration-replay-gate --allow-dirty
```

- `npm test`: passes Rust unit/integration/doc tests, TypeScript checking, Node demo-data tests, production site build, and 14 Chromium desktop/mobile Playwright checks (one deliberately skipped desktop-only duplicate of the mobile-specific assertion).
- Playwright + axe: no serious or critical violations on `/`, `/privacy/`, or `/terms/` at desktop and 390 px mobile sizes; navigation, keyboard arrow behavior, blocked-state feedback, title, main landmark, single H1, and console cleanliness are exercised.
- Lighthouse 12.8.2 mobile: Performance **100**, Accessibility **100**, Best Practices **100**, SEO **100**. FCP 0.9 s, LCP 1.1 s, Speed Index 0.9 s, total blocking time 0 ms, CLS 0.
- Production budgets: initial JS 4.69 KB (2.08 KB gzip), CSS 11.96 KB (3.51 KB gzip), no webfont payload, 13 KB mobile hero. All are below the supplied budgets.
- `npm audit`: 0 vulnerabilities.
- `cargo package`: produced `target/package/migration-replay-gate-0.1.0.crate` (13 KB) and completed package verification.
- Release binary: `target/release/mrg` (about 1.4 MB in this environment).

## Deploy and publish

- Static deployment root: `dist/site` (`index.html` is at that root).
- Build command: `npm ci && npm run build`.
- The factory should publish releases/registry artifacts; this worker did not publish. The ready-to-publish Rust command is `cargo package -p migration-replay-gate`.

## Known gap / next step

This worker image has neither Docker nor Podman, so the real-container smoke script could not be executed here. The container orchestration compiles, its parsing/safety/classification boundaries are tested, and the exact smoke command is checked in; run `scripts/smoke-docker.sh` on the first CI runner with Docker or Podman before cutting the binary release. No product functionality was replaced with a mock—the browser replay alone is recorded, as required for a static landing page.
