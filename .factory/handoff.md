# Migration Replay Gate — verification 3 handoff

## Release decision — FAIL (2026-08-28)

Candidate tested: `9f6d941045ee154acfac1733ca22e5525c272b8f`.
Live URL tested: <https://migration-replay-gate.sociobot.in/>.

**Do not release this candidate unchanged.** Live `/privacy/` and `/terms/`
emit CSP console errors because their inline `<style>` blocks are rejected by
the deployed `style-src 'self'` policy. This is a P2 release blocker under the
factory's no-console-errors-on-load requirement. Full exact evidence is in
`.factory/verification-3.md`.

## What passed

- Clean install and complete repository suite: `npm ci`, `npm test`, and exact
  `npm run build` passed. The suite covers 15 Rust tests + 1 doctest,
  TypeScript, 3 Node tests, Vite build, and desktop/390px Playwright coverage.
- Rust quality and release gates passed: formatting check, Clippy with denied
  warnings, release build, and `cargo package -p migration-replay-gate
  --allow-dirty` package verification.
- The packaged 0.1.2 CLI was installed into a clean consumer. Its help,
  safe three-scenario replay, destructive-DDL block (exit 2), parser/input
  rejection (exit 3), and production-URL guard (exit 3) were independently
  exercised.
- Live root and production assets are byte-identical to the candidate build;
  caching, security headers, no third-party root requests, desktop/mobile
  replay, keyboard focus, reduced motion, service-worker offline reload,
  axe serious/critical checks, and bundle budgets otherwise passed.

## Required next step

Move the legal-page CSS to a same-origin stylesheet, preserving the strict CSP
(do not add `unsafe-inline`), and add deployed-header browser console tests for
`/privacy/` and `/terms/`. Rebuild, deploy, and rerun the focused live checks.

## Verification commands

```sh
npm ci
npm test
npm run build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --release --workspace
cargo package -p migration-replay-gate --allow-dirty
```

`scripts/smoke-docker.sh` needs Docker or Podman and a `postgres:16-alpine`
image. This verifier environment has neither runtime; it returned the expected
exit 4, so an actual container replay remains to be rerun where either is
available. The crate is ready to package but must not be published by this
worker.
