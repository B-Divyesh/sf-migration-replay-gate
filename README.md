# Migration Replay Gate

Migration Replay Gate (`mrg`) is a CI gate for teams whose schema migration or snapshot tool needs to behave safely when reality is messy. It starts a disposable Postgres container, seeds a baseline and an explicit partial-state fixture, runs your own migration command against clean, repeated, and partial states, captures emitted DDL and errors, then exits non-zero for destructive, non-idempotent, or failed outcomes.

Landing page and interactive replay: <https://migration-replay-gate.sociobot.in>

It never accepts a database URL and never connects to staging or production. The tested command receives an ephemeral loopback `DATABASE_URL` plus standard `PG*` variables.

## Install

Download a release binary, or build locally:

```sh
cargo install --path crates/mrg
mrg --help
```

Docker or Podman and access to the `postgres:16-alpine` image are required at runtime.

## Usage

Create a baseline fixture and an intentionally partial state:

```sql
-- fixtures/baseline.sql
CREATE TABLE accounts (id bigint PRIMARY KEY);
```

```sql
-- fixtures/partial.sql
-- The target table already exists, but only some columns landed.
CREATE TABLE audit_events (id bigint PRIMARY KEY);
```

Gate your existing migration command:

```sh
mrg gate \
  --command "npm run migrate" \
  --baseline fixtures/baseline.sql \
  --partial fixtures/partial.sql
```

For CI, add `--json` and inspect the stable report on stdout:

```sh
mrg gate --command "npm run migrate" \
  --baseline fixtures/baseline.sql \
  --partial fixtures/partial.sql \
  --json > replay-report.json
```

Exit codes are `0` safe, `2` unsafe replay outcome, `3` invalid input, and `4` container/runtime failure. Fixtures containing destructive SQL are rejected unless `--allow-destructive-fixtures` is present; destructive SQL observed from the migration command always blocks the gate.

## What gets tested

1. **Clean apply** — seed baseline, run the command once.
2. **Repeat apply** — run it again against the already-migrated clean database.
3. **Partial-state apply** — seed baseline plus every `--partial` fixture in a fresh database, then run the command.

The report distinguishes command errors, repeat failures, partial-state failures, and destructive DDL. SQL is captured from Postgres statement logging as well as command output.

## Develop and verify

```sh
cargo test --workspace
cargo build --release --workspace
cargo package -p migration-replay-gate --allow-dirty
npm ci
npm test
npm run build
```

`npm run build` writes the static landing/docs site to `dist/site`. The Rust binary is written to `target/release/mrg`.

The fixture integration suite uses a fake container runtime and does not require Docker. For a real smoke test, run `scripts/smoke-docker.sh` on a machine with Docker or Podman.

## Scope

Migration Replay Gate does not generate migrations, synchronize row data, host a database, or touch shared environments. It is free, local-first, contains no telemetry, and makes no network requests except those your container runtime needs to obtain the Postgres image.

## License

MIT. See [LICENSE](LICENSE).
