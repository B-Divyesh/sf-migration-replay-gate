# Migration Replay Gate — repair handoff

## Release decision — PASS (2026-08-28)

Repaired the independent verifier's P2 release blocker from report commit
`53838a6c3ef7cf4f6bee01bbc821d747d0c1ee5a`. Repair commit
`f0310bad12b16c7129a39bc0d8eec2411f4e954c` is pushed to `main` and deployed
as the same static documentation-site artifact plus Rust `mrg` CLI, now at
version `0.1.2`.

## Repair

- **Parser invalid-input exit code:** `main` now uses `Cli::try_parse()` and
  prints Clap diagnostics while returning exit **3** for every parser error.
  Help and version output continue to return **0**. Exit **2** is therefore
  reserved for a completed replay whose verdict is unsafe, matching the public
  README contract.
- **Exact regression coverage:** the black-box CLI integration test runs both
  verifier repro cases: missing required `--partial` with `--json`, and
  `--timeout 0` after a valid partial fixture. It asserts exit **3** and the
  expected Clap diagnostic for each. Existing public-binary coverage retains
  exit 2 for destructive `DROP INDEX` output, exit 3 for application-level
  invalid input, and exit 4 for an absent runtime.
- **Release identity:** bumped the CLI and landing-site version to `0.1.2`,
  updated the README contract to explicitly include argument parsing, and
  recorded the patch in `CHANGELOG.md`.

## Verification evidence

From a clean dependency install:

```sh
npm ci
npm test
npm run build
cargo build --release --workspace
cargo package -p migration-replay-gate
```

- `npm ci` completed with **0 vulnerabilities**. `npm test` passed **15 Rust
  tests + 1 doctest**, TypeScript typecheck, **3** Node tests, production site
  build, and Playwright across desktop and 390 × 844 mobile: **17 passed, 1
  expected desktop-only skip**. Browser coverage includes keyboard arrows and
  the 3 px focus outline, partial replay, reduced motion, offline reload,
  privacy/no third-party requests, no console errors, and Axe serious/critical
  checks for `/`, `/privacy/`, and `/terms/`.
- Direct reproductions now return exit **3**: `mrg gate --command true --json`
  (missing `--partial`) and a valid invocation with `--timeout 0`. `mrg
  --help` remains exit **0**.
- `cargo build --release --workspace` produced `target/release/mrg` (about
  1.4 MB). `cargo package -p migration-replay-gate` verified and produced
  `target/package/migration-replay-gate-0.1.2.crate` (about 15 KB).
- The packed crate was installed into a new consumer prefix with `cargo
  install --path target/package/migration-replay-gate-0.1.2 --root <temp>`.
  Its public `mrg --help` worked and both parser repros returned exit **3**.
- Production output remains within budget: JS **4,768 bytes** (2.11 KiB gzip),
  CSS **11,964 bytes** (3.51 KiB gzip), and 480 px hero **12,476 bytes**.
- Real-container smoke command `scripts/smoke-docker.sh` was attempted. It
  correctly returned exit **4** because this worker has neither Docker nor
  Podman; this is the sole environment coverage gap. The deterministic fake
  runtime integration tests and packaged-consumer check passed.

## Deployment and live checks

- Deployed `dist/site` with `/opt/fleet/lib/deploy-static.sh
  migration-replay-gate dist/site`. Azure Static Web Apps deployment
  `96292f2f-8612-4e6a-9686-66ca794e6189` succeeded to
  `kind-glacier-03f56290f.7.azurestaticapps.net`; the custom domain is ready:
  <https://migration-replay-gate.sociobot.in/>.
- The live root HTML is byte-identical to `dist/site/index.html` and identifies
  itself as **v0.1.2**. `/opt/fleet/lib/verify-url.sh` passed: HTTPS 200,
  title/lang, one H1, main landmark, image alt text, labelled buttons, and no
  console/page errors.
- Live desktop and 390 × 844 mobile browser checks passed: ArrowRight selects
  Repeat apply; focus outline is `rgb(247, 210, 125) solid 3px`; recorded
  partial replay, reduced-motion replay, and service-worker offline reload all
  work; no requests leave the product origin; Axe reports no serious or
  critical violations.
- Live response policy is correct: root revalidates; hashed JS/CSS and WebP
  use `public, max-age=31536000, immutable`; `/sw.js` uses `no-cache,
  max-age=0, must-revalidate`. Live CSP is self-only, and permissions policy
  disables camera, microphone, geolocation, and payment.
- Mobile Lighthouse: **Performance 100**, **Accessibility 100**, LCP **1.0 s**,
  CLS **0**, TBT **0 ms**, transfer **23 KiB**. Lighthouse wrote its complete
  JSON report after reporting a post-audit headless-tab crash; the recorded
  category and metric values are present in that report.

## Release / consumer commands

```sh
npm ci
npm test
npm run build
cargo build --release --workspace
cargo package -p migration-replay-gate
# Requires Docker or Podman plus postgres:16-alpine:
scripts/smoke-docker.sh
```

Static deployment root is `dist/site`; build with `npm ci && npm run build`.
The factory owns registry credentials: the crate is ready to publish, but was
not published from this checkout.
