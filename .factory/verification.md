# Independent verification — FAIL

Verified on 2026-08-27 against candidate commit
`5411c206a8cd80cdabea6495ae2b5e8abf80f718`.

Live URL: <https://migration-replay-gate.sociobot.in/>

## Decision

**FAIL.** The CLI returns a safe CI verdict for a destructive `DROP INDEX`
operation and also permits that operation in a fixture without the required
explicit acknowledgement. This contradicts the product contract: identify and
block destructive outcomes, and require destructive test fixtures to be
explicit.

## Release-blocking defect

### P1 — destructive `DROP INDEX` is classified safe

With the compiled candidate binary and a deterministic fake Docker runtime
(the same no-container integration strategy described by the repository), this
command completed all three scenarios with `"verdict":"safe"` and process
exit `0`:

```sh
PATH="$PWD/.qa-runtime:$PATH" target/release/mrg gate \
  --command 'printf "DROP INDEX idx_accounts;\\n"' \
  --partial fixtures/example/partial.sql --runtime docker --json
```

Each scenario captured `DROP INDEX idx_accounts` in `sql` but supplied no
finding. The public contract says destructive DDL blocks the gate. `DROP
INDEX` removes a schema object and must not receive a safe CI verdict.

The fixture safety boundary fails in the same way. A partial fixture containing
only `DROP INDEX idx_accounts;` was accepted without
`--allow-destructive-fixtures`; the safe migration command then returned exit
`0`. By contrast, the existing `DROP TABLE` fixture test correctly returns
input exit `3`. This demonstrates an incomplete destructive-statement detector,
not a runtime availability problem.

Recommended remediation: explicitly cover destructive PostgreSQL object and
integrity removals (at least `DROP INDEX`, `DROP VIEW`, `DROP SEQUENCE`,
`ALTER TABLE ... DROP CONSTRAINT`; decide/document the intended policy for
other destructive forms), deduplicate findings collected from command output
and database logs, and add black-box CLI tests for both command and fixture
paths.

## Secondary deployment defect

### P2 — live immutable asset caching is not effective

The built `site/public/_headers` declares one-year immutable caching for
`/assets/*` and `/*.webp`, but the live server returned this for the hashed JS
asset and service worker on 2026-08-27:

```text
cache-control: public, must-revalidate, max-age=30
```

This misses the supplied production caching policy for hashed static assets.
It is not the reason for the FAIL decision, but needs deployment configuration
that actually applies the checked-in headers.

## Checks that passed

- Clean candidate checkout confirmed: `git rev-parse HEAD` was
  `5411c206a8cd80cdabea6495ae2b5e8abf80f718` before verification changes.
- `npm ci`, `cargo test --workspace`, `npm test`, `cargo build --release
  --workspace`, and `npm run build` passed. `npm test` passed all Rust unit,
  integration, and doctests; TypeScript; Node tests; production build; and 14
  Playwright desktop/390px cases. The generated site is `dist/site`.
- `cargo package -p migration-replay-gate --allow-dirty` verified and produced
  `target/package/migration-replay-gate-0.1.0.crate` (12,955 bytes). A separate
  `cargo install --path crates/mrg --root /tmp/mrg-verify-install --force`
  installed `mrg 0.1.0`; `--help` and `gate --help` worked.
- Compiled-CLI end-to-end fake-runtime cases produced the documented results:
  safe migration exit 0; duplicate repeat exit 2 with `non_idempotent`;
  partial-state failure exit 2 with `partial_state_failure`; destructive
  `ALTER TABLE ... DROP COLUMN` exit 2; literal production URL rejected with
  JSON input error/exit 3; destructive `DROP TABLE` fixture rejected with exit
  3. CLI timeout lower bound (`--timeout 0`) was rejected by clap.
- Docker and Podman are not installed in this verifier image, so the actual
  Postgres-container smoke test could not be run. This is recorded as an
  environment limitation, not substituted for the classification evidence
  above.
- Production deployment matches the candidate: SHA-256 values matched for
  `index.html`, CSS, JS, service worker, offline page, and all three hero WebP
  variants. Live browser checks on desktop and 390px found no console/page
  errors, no outbound runtime requests, no axe serious/critical violations,
  one H1, `lang=en`, and no horizontal overflow. Keyboard Tab exposed a visible
  `rgb(247, 210, 125) solid 3px` focus outline; arrow tab selection and the
  blocked partial replay worked. Reduced motion rendered the replay result
  immediately. Service-worker offline reload retained the usable replay UI.
- Lighthouse 12.8.2 mobile against the live URL: Performance 0.97,
  Accessibility 1.00; LCP 2.1 s, CLS 0, maximum potential FID 57 ms, and
  total transferred bytes 22 KiB. Built payloads: JS 4,692 bytes, CSS 11,964
  bytes, and mobile hero 12,476 bytes; all are within the stated size budgets.

## Informational observation

`ALTER TABLE ... DROP COLUMN` emitted once by a migration results in two
identical `destructive_sql` findings per scenario because the classifier scans
both captured SQL and command output without deduplication. It does block
correctly; treat this as a report-quality fix alongside the P1 remediation.
