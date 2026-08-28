# Independent verification 2 — FAIL

Verified 2026-08-28 against candidate commit
`c5e58fc9d8bcd9bddea98dd0d80fc3087b75b943`.

Live URL: <https://migration-replay-gate.sociobot.in/>

## Decision

**FAIL.** The deployed CLI/site otherwise meet the replay-gate acceptance
contract and the earlier destructive-DDL/cache defects are fixed, but the
published CLI exit-code contract is still violated for parser-level invalid
input. The README says invalid input exits **3**; a missing required
`--partial` fixture and `--timeout 0` both exit **2** (Clap's syntax-error
status), which is also the documented unsafe-migration status. This makes a
CI consumer unable to distinguish malformed invocation from an unsafe replay
using the stated contract.

## Release-blocking defect

### P2 — parser invalid input returns the documented unsafe-result exit code

From the packed, clean-consumer installation of `mrg 0.1.1`:

```text
$ mrg gate --command true --json
exit 2
error: the following required arguments were not provided:
  --partial <FILE>

$ mrg gate --command true --partial fixtures/example/partial.sql \
    --runtime docker --json --timeout 0
exit 2
error: invalid value '0' for '--timeout <TIMEOUT>': 0 is not in 1..=3600
```

Both are invalid user input, while `README.md` explicitly defines exit `2` as
"unsafe replay outcome" and exit `3` as "invalid input." Validation errors
which reach `run_gate` do correctly use exit `3`: a literal production
connection string returned JSON category `input` and exit `3`.

Recommended remediation: customise Clap's error handling (or revise the
documented public exit-code contract and add black-box tests) so all invalid
invocations consistently use a distinct status. Preserve exit 2 exclusively
for a completed unsafe replay.

## Verification that passed

### Candidate, dependencies, build, and package

- Started from a clean checkout at the exact candidate. `origin/main` also
  resolves to `c5e58fc9d8bcd9bddea98dd0d80fc3087b75b943`.
- `npm ci` completed with 0 reported vulnerabilities.
- `npm test` passed: 14 Rust unit/integration tests plus 1 doctest, TypeScript
  typecheck, 3 Node tests, Vite production build, and Playwright (**17 passed,
  1 expected desktop-only skip**) across desktop and 390 × 844 mobile.
- `cargo build --release --workspace` passed. `cargo package -p
  migration-replay-gate --allow-dirty` passed package verification and created
  `target/package/migration-replay-gate-0.1.1`.
- The packed crate was installed into a fresh `/tmp/mrg-consumer.*` prefix via
  `cargo install --path target/package/migration-replay-gate-0.1.1 --root …`.
  `mrg --help` is useful and lists one `gate` command. Its public CLI was then
  exercised against the repository's deterministic fake container-runtime
  seam: a safe `true` migration yielded three passing scenarios and exit 0;
  a `DROP INDEX` command yielded `unsafe`, one `destructive_sql` finding in
  each scenario, and exit 2. A literal production URL is rejected with JSON
  category `input`/exit 3; unavailable Docker reports JSON category
  `runtime`/exit 4.
- The real `scripts/smoke-docker.sh` could not run because this verifier image
  has neither `docker` nor `podman`. This is an environment limitation, not a
  substitute for the package, fake-runtime, or classification evidence above.

### Live deployment, privacy, accessibility, and performance

- The live root HTML and every referenced candidate artifact compared byte for
  byte with `dist/site`: hashed JS/CSS, service worker, and 480/720/default
  WebP assets. The page references the same hashes
  `main-CBPTGDHV.js` and `main-CaLKIdS-.css`.
- Response policy is live and correct: root `Cache-Control: public, max-age=0,
  must-revalidate`; hashed JS/CSS and WebP `public, max-age=31536000,
  immutable`; `/sw.js` `no-cache, max-age=0, must-revalidate`. HTTPS headers
  include HSTS, `nosniff`, a self-only CSP (`connect-src 'self'`, no external
  script/style/font/image origin), `frame-ancestors 'none'`, strict referrer
  policy, and disabled camera/microphone/geolocation/payment permissions.
- Fresh Playwright checks against the HTTPS deployment at desktop and 390 × 844
  found the expected title, one H1, one main landmark, no console/page errors,
  and requests only to `https://migration-replay-gate.sociobot.in`. The partial
  recorded replay renders its actionable blocked state. Arrow-key tab selection
  works; the focused tab has a visible `rgb(247, 210, 125) solid 3px` outline.
  With reduced motion it completes immediately. The service worker became
  ready and an offline reload still exposed the replay UI on both viewports.
- Axe against the live landing page reported no serious or critical findings
  on either viewport. The repository suite also covers `/privacy/` and
  `/terms/`; both live pages returned 200. The privacy statement accurately
  declares no analytics, cookies, accounts, forms, advertising, or third-party
  runtime scripts.
- Live mobile Lighthouse 13.4.1: Performance **94**, Accessibility **100**;
  LCP **1.1 s**, CLS **0.005**, TBT **290 ms**, transfer **23 KiB**. Production
  output is 4,768-byte JS (2.11 KiB gzip) and 11,964-byte CSS (3.51 KiB gzip),
  comfortably within the static bundle budgets.

## Scope note

No product source was modified during this verification. This report and the
handoff update are the only intended worktree changes.
