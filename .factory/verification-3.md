# Independent verification 3 — FAIL

Verified 2026-08-28 from a clean checkout at candidate commit
`9f6d941045ee154acfac1733ca22e5525c272b8f`.

Live URL: <https://migration-replay-gate.sociobot.in/>

## Decision

**FAIL.** The replay CLI, package, root experience, deployment identity, and
most operational quality gates passed fresh independent checks. However, the
live Privacy and Terms pages each produce a browser console error on load: the
site's strict CSP blocks their inline stylesheet. This violates the factory
definition of done (no console errors on load) and leaves those pages without
their intended presentation. The deployed headers and candidate output make
this reproducible from the candidate, not a deployment-only mismatch.

## Defects

### P2 — CSP blocks the inline stylesheets on both legal pages

At both desktop and 390 × 844 mobile, loading `/privacy/` and `/terms/`
records this browser console error (one per page):

```text
Applying inline style violates the following Content Security Policy directive
'style-src 'self''. ... The action has been blocked.
```

The live response policy is intentionally strict:

```text
Content-Security-Policy: ... style-src 'self'; ...
```

Yet both candidate files contain an inline `<style>` element:

- `site/privacy/index.html:7`
- `site/terms/index.html:7`

The same inline style appears in the freshly built `dist/site` files, and the
live root, JS, CSS, service worker, and WebP files compare byte-for-byte to
that production build. Consequently the current release both enforces and
violates its own policy. Remediate by moving the shared legal-page CSS into a
same-origin stylesheet (preferred; retain the strict CSP), then add a browser
console-error smoke test for `/privacy/` and `/terms/` under the production
headers. Do not weaken `style-src` with `unsafe-inline`.

No P0 or P1 defects were found.

## Evidence that passed

### Candidate, install, tests, static analysis, and package

- The initial worktree was clean, `HEAD` and `origin/main` both resolved to
  `9f6d941045ee154acfac1733ca22e5525c272b8f`.
- `npm ci` completed with **0 vulnerabilities**. `npm test` exercised 15 Rust
  unit/integration tests and 1 doctest, TypeScript typecheck, 3 Node tests,
  the Vite production build, and the Playwright desktop/mobile suite (17
  passed, 1 expected desktop-only skip). A fresh `npm run build` also passed
  and wrote `dist/site`.
- Additional available Rust gates passed: `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo build
  --release --workspace`. The release binary is 1.4 MB.
- `cargo package -p migration-replay-gate --allow-dirty` passed cargo's package
  verification and produced `migration-replay-gate-0.1.2.crate` (14.4 KiB).
  It was installed from that packed source into a fresh
  `/tmp/mrg-consumer-verify-3` consumer prefix; its `mrg --help` exposes the
  single documented `gate` command.

### End-to-end CLI contract

- Against the package consumer and the repository's deterministic disposable
  runtime seam, `mrg gate --command true ... --json` returned exit **0** and
  three passing scenarios: `clean_apply`, `repeat_apply`, and `partial_apply`.
  A command emitting `DROP INDEX idx_accounts` returned exit **2**, verdict
  `unsafe`, and exactly one `destructive_sql` finding in each scenario.
- Boundary and recovery cases use the documented distinct statuses: a missing
  required `--partial` and `--timeout 0` parser error exit **3**; a missing
  working directory exits **3** JSON category `input`; an embedded
  `postgresql://prod.example/...` target exits **3** and is rejected before a
  runtime starts. This confirms the previous parser-exit defect is fixed.
- `scripts/smoke-docker.sh` correctly returned exit **4** with `no supported
  container runtime found`. Neither Docker nor Podman is installed in this
  verifier image, so a real Postgres-container replay is the sole environment
  coverage gap; the packaged consumer and fake-runtime integration coverage
  are not presented as a substitute for that unavailable smoke test.

### Live deployment, privacy, browser behavior, and performance

- Fresh byte comparisons found the live `/`, hashed JS/CSS, `sw.js`, and all
  three WebP assets identical to the candidate production build. The live
  version is therefore the candidate's static artifact, not a stale or
  divergent deployment.
- Live root response policy is correct: root revalidates; hashed JS/CSS and
  WebP are `public, max-age=31536000, immutable`; `sw.js` is
  `no-cache, max-age=0, must-revalidate`. HTTPS also sends HSTS, `nosniff`,
  strict referrer policy, a self-only CSP with `connect-src 'self'`,
  `frame-ancestors 'none'`, and a Permissions-Policy disabling camera,
  microphone, geolocation, and payment.
- At desktop and 390 × 844 mobile, the live root has the correct title, one
  H1, one main landmark, no console/page errors, and no outbound requests:
  all observed requests remained at `migration-replay-gate.sociobot.in`.
  Keyboard ArrowRight selects Repeat apply; its designed focus is
  `rgb(247, 210, 125) solid 3px`. The partial replay shows the blocked
  partial-state outcome. Reduced motion completed the replay in 86 ms desktop
  and 95 ms mobile, and a service-worker-controlled offline reload retained
  the replay UI on both sizes.
- Axe reported **no serious or critical** findings for `/`, `/privacy/`, or
  `/terms/` on live mobile; desktop root also had none. The legal pages' axe
  result does not erase their CSP console errors described above.
- The privacy page accurately states no CLI telemetry and no website
  analytics, cookies, accounts, forms, advertising, or third-party runtime
  scripts. Runtime request observation agrees for the root experience.
- Mobile Lighthouse 13.4.1 recorded **98 Performance** and **100
  Accessibility**, LCP **1.3 s**, CLS **0**, TBT **140 ms**, and 23 KiB
  transfer. Lighthouse reported a post-audit headless-tab crash but wrote the
  complete JSON report containing these category and metric values. Initial
  production JS is 4,768 bytes (2.11 KiB gzip), CSS is 11,964 bytes (3.51 KiB
  gzip), and the 480 px hero is 12,476 bytes: all within the stated budgets.

## Scope

No product code was modified. This report and the handoff update are the only
intended repository changes.
