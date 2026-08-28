# Verification handoff — FAIL

Verified on 2026-08-28 against
`c5e58fc9d8bcd9bddea98dd0d80fc3087b75b943` and the matching live deployment:
<https://migration-replay-gate.sociobot.in/>.

## Release decision

**FAIL — do not release this candidate as meeting its published CLI contract.**

The one open P2 defect is a public exit-code contradiction: README documents
exit 2 as unsafe replay and exit 3 as invalid input. Parser-level invalid
input (`mrg gate --command true --json`, missing required `--partial`; and
`--timeout 0`) exits 2 instead. Validation which reaches the application does
return exit 3, proving the inconsistency. Correct the parser handling or the
published contract and add black-box coverage before calling this a PASS.

Full evidence is in `.factory/verification-2.md`.

## What passed

- Clean checkout and `origin/main` both resolved to the tested commit.
- `npm ci`; `npm test` (14 Rust tests + 1 doctest, typecheck, 3 Node tests,
  production site build, 17 Playwright passed/1 expected skip); `cargo build
  --release --workspace`; and `cargo package -p migration-replay-gate
  --allow-dirty` all passed.
- A crate package installed into a clean consumer prefix. Its CLI help,
  successful safe replay (exit 0), destructive-DROP blocking (exit 2),
  production-URL rejection (exit 3), and missing-runtime error (exit 4) were
  exercised through the public binary. The replay cases used the repository's
  deterministic fake container-runtime seam.
- The live deployment is byte-identical to the candidate production HTML,
  JS/CSS, service worker, and hero assets. It has correct immutable static
  caching and revalidated service-worker caching, a self-only CSP, no observed
  external browser requests, no console/page errors, and live privacy/terms
  pages.
- Live desktop and 390px mobile checks passed: exactly one H1/main, keyboard
  arrows, visible 3px focus outline, reduced motion, actionable partial replay,
  service-worker offline reload, and no axe serious/critical findings.
  Lighthouse mobile: Performance 94, Accessibility 100, LCP 1.1 s, CLS 0.005,
  23 KiB transfer.

## Known limitation

Docker and Podman are not installed in this verifier image, so a real
Postgres-container run of `scripts/smoke-docker.sh` was unavailable. This is
an environmental coverage gap; package validation and deterministic
integration coverage passed.

## Verification / release commands

```sh
npm ci
npm test
npm run build
cargo build --release --workspace
cargo package -p migration-replay-gate --allow-dirty
# Requires Docker or Podman:
scripts/smoke-docker.sh
```

The factory owns publishing credentials; do not publish from this checkout.
