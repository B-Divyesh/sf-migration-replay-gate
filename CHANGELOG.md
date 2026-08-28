# Changelog

All notable changes follow [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.1.2] - 2026-08-28

### Fixed

- Return exit 3 for Clap-level invalid invocations, preserving exit 2 exclusively for completed unsafe replay outcomes.

## [0.1.1] - 2026-08-28

### Fixed

- Block every PostgreSQL `DROP` statement, plus `TRUNCATE` and table/domain removal clauses, in both migration output and fixtures.
- Deduplicate destructive findings observed in both command output and Postgres logs.
- Add Azure Static Web Apps cache configuration so hashed assets receive immutable caching and the service worker is revalidated.

## [0.1.0] - 2026-08-27

### Added

- Disposable Postgres replay gate for clean, repeat, and partial states.
- Human and JSON CI reports with explicit exit codes.
- Static documentation site with an interactive recorded replay.
