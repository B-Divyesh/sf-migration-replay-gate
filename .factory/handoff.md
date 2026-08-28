# Migration Replay Gate — repair handoff

## Release decision — PASS locally (2026-08-28)

Repaired the independent verifier findings from candidate
`5411c206a8cd80cdabea6495ae2b5e8abf80f718` (report commit
`bf4f5857bef2b9b361c86453505a4893ff6f869e`). This remains the same Rust
`mrg` CLI and Vite static documentation site, now at version `0.1.1`.

## Repairs

- **P1 destructive `DROP INDEX` verdict:** the SQL policy now blocks every
  PostgreSQL `DROP` statement, `TRUNCATE`, and removal clauses in `ALTER
  TABLE` and `ALTER DOMAIN`. This covers `DROP INDEX`, `DROP VIEW`, `DROP
  SEQUENCE`, and `ALTER TABLE ... DROP CONSTRAINT` as well as the previously
  covered column/table forms. The same policy validates fixtures, so a
  destructive fixture requires `--allow-destructive-fixtures`.
- **P1 duplicate destructive findings:** normalized destructive evidence is
  de-duplicated when the same DDL appears in both the database log and command
  output. Each scenario now reports one finding for one observed statement.
- **P2 deployment cache rules:** added
  `site/public/staticwebapp.config.json`, the Azure Static Web Apps deployment
  configuration used by the factory static host. It makes `/assets/*` and all
  three versioned hero WebP files immutable for one year, while `/sw.js` is
  explicitly revalidated (`no-cache, max-age=0, must-revalidate`). The
  configuration is copied into `dist/site` by the production build.
- **Offline regression coverage:** service workers are also enabled for secure
  loopback development origins, allowing the browser suite to verify an actual
  offline reload rather than only source inspection. Production continues to
  register only on HTTPS.

## Exact regression coverage

- Rust unit coverage verifies `DROP INDEX`, `DROP VIEW`, `DROP SEQUENCE`,
  `ALTER TABLE ... DROP CONSTRAINT`, `ALTER DOMAIN ... DROP CONSTRAINT`, and
  ignores a commented `DROP` / non-destructive add-column statement.
- A black-box CLI integration test supplies a deterministic fake Docker
  runtime and runs `printf 'DROP INDEX idx_accounts;\\n'` through clean,
  repeat, and partial scenarios. It asserts exit **2**, `unsafe`, and exactly
  one `destructive_sql` finding per scenario.
- A second black-box CLI test supplies a partial fixture containing `DROP
  INDEX idx_accounts;` and asserts input exit **3** and the explicit
  `--allow-destructive-fixtures` guidance.
- Deployment configuration tests assert the immutable asset/image and
  revalidated-service-worker response policy. Browser tests now also cover
  offline reload plus no third-party requests.

## Verification performed

From a clean dependency install:

```sh
npm ci
npm test
npm run build
cargo build --release --workspace
cargo package -p migration-replay-gate --allow-dirty
```

- `npm ci`: completed; `npm audit` reported 0 vulnerabilities.
- `npm test`: passed all Rust unit/integration/doctests, TypeScript check,
  Node tests, production build, and Playwright. Browser result: **17 passed,
  1 intentionally skipped** across desktop and the 390 × 844 mobile project.
  This includes keyboard-arrow replay tabs, axe serious/critical checks on
  `/`, `/privacy/`, and `/terms/`, privacy/no-third-party requests, and a
  service-worker offline reload that can still run the recorded partial replay.
- `cargo build --release --workspace`: passed; binary is
  `target/release/mrg` (about 1.4 MB here).
- `cargo package -p migration-replay-gate --allow-dirty`: passed its package
  verification and produced `target/package/migration-replay-gate-0.1.1.crate`
  (about 15 KB). A separate `cargo install --path crates/mrg --root <temp>
  --force` succeeded; both `mrg --help` and `mrg gate --help` were exercised.
- Production site build: `dist/site`, including
  `staticwebapp.config.json`. Payloads: JS **4.77 KB** (2.11 KB gzip), CSS
  **11.96 KB** (3.51 KB gzip), mobile hero **13 KB**; all remain within budget.
- Local Lighthouse 12.8.2 mobile (Chromium): Performance **100**,
  Accessibility **100**, Best Practices **100**, SEO **100**; FCP **0.9 s**,
  LCP **1.2 s**, total blocking time **0 ms**, CLS **0**. The Lighthouse
  runner reported a post-audit headless-tab crash, but wrote the complete JSON
  report and all category/metric results above.

## Deployment and consumer handoff

- Static deployment root: `dist/site`; build command: `npm ci && npm run build`.
  The committed `staticwebapp.config.json` is required for the factory Azure
  Static Web Apps deployment to apply the cache policy.
- Release package is ready but was **not published** (factory owns registry
  credentials). Publish preparation command: `cargo package -p
  migration-replay-gate`.
- The CLI runtime still needs Docker or Podman plus access to
  `postgres:16-alpine`. This container has neither runtime, so
  `scripts/smoke-docker.sh` could not run; all parser/classification and fake
  runtime integration boundaries were exercised locally.

## Deployment evidence

- Deployed with `/opt/fleet/lib/deploy-static.sh migration-replay-gate
  dist/site`; Azure Static Web Apps deployment
  `3f78c5aa-76a8-4f70-aa97-a33e7a186071` completed successfully to
  `kind-glacier-03f56290f.7.azurestaticapps.net`, with the existing custom
  domain ready.
- Live <https://migration-replay-gate.sociobot.in/> now identifies as
  `v0.1.1`. Its hashed JS response and the mobile WebP response return
  `Cache-Control: public, max-age=31536000, immutable`; `/sw.js` returns
  `Cache-Control: no-cache, max-age=0, must-revalidate`.
- `/opt/fleet/lib/verify-url.sh` against the live URL passed: HTTPS 200,
  841 ms browser load in this worker, no console/page errors, title present,
  `lang=en`, exactly one H1, main landmark, no missing image alt text, and no
  unlabeled buttons.
