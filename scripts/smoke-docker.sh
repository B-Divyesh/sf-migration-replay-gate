#!/usr/bin/env sh
set -eu

runtime="${MRG_RUNTIME:-docker}"
work_dir="$(mktemp -d)"
trap 'rm -rf "$work_dir"' EXIT

cat > "$work_dir/migrate.sh" <<'SCRIPT'
#!/usr/bin/env sh
set -eu
psql "$DATABASE_URL" -v ON_ERROR_STOP=1 <<'SQL'
CREATE TABLE IF NOT EXISTS audit_events (
  id bigint PRIMARY KEY,
  account_id bigint NOT NULL REFERENCES accounts(id)
);
CREATE INDEX IF NOT EXISTS audit_events_account_id_idx ON audit_events(account_id);
SQL
SCRIPT
chmod +x "$work_dir/migrate.sh"

cargo run -q -p migration-replay-gate -- gate \
  --runtime "$runtime" \
  --command "$work_dir/migrate.sh" \
  --baseline fixtures/example/baseline.sql \
  --partial fixtures/example/partial.sql
