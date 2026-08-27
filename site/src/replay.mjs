export const scenarios = Object.freeze({
  clean: {
    label: "Clean apply",
    status: "safe",
    verdict: "PASS / CLEAN APPLY",
    meta: "3 statements · exit 0 · 842 ms",
    lines: [
      ["muted", "[mrg] disposable postgres ready on loopback"],
      ["cyan", "[01]  seeding fixtures/baseline.sql"],
      ["plain", "$ npm run migrate"],
      ["sql", "ALTER TABLE accounts ADD COLUMN handle text;"],
      ["sql", "CREATE UNIQUE INDEX accounts_handle_idx …;"],
      ["green", "✓ migration command exited 0"]
    ]
  },
  repeat: {
    label: "Repeat apply",
    status: "safe",
    verdict: "PASS / IDEMPOTENT",
    meta: "0 new statements · exit 0 · 311 ms",
    lines: [
      ["muted", "[mrg] replaying against the migrated database"],
      ["cyan", "[02]  same command, same target"],
      ["plain", "$ npm run migrate"],
      ["plain", "schema already current"],
      ["green", "✓ no new DDL observed"],
      ["green", "✓ migration command exited 0"]
    ]
  },
  partial: {
    label: "Partial state",
    status: "unsafe",
    verdict: "BLOCKED / PARTIAL-STATE FAILURE",
    meta: "duplicate_table · exit 1 · 476 ms",
    lines: [
      ["muted", "[mrg] seeded baseline + partial.sql"],
      ["cyan", "[03]  replaying against partial state"],
      ["plain", "$ npm run migrate"],
      ["sql", "CREATE TABLE audit_events ( … );"],
      ["red", "ERROR: relation \"audit_events\" already exists"],
      ["amber", "! unsafe: command assumes an untouched environment"]
    ]
  }
});

export function scenarioFor(key) {
  return scenarios[key] ?? scenarios.clean;
}
