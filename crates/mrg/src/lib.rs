//! Analysis primitives for Migration Replay Gate.
//!
//! The binary owns container orchestration; this library keeps the stable report
//! types and replay classification available to other Rust tools.

use serde::Serialize;
use std::collections::HashSet;
use std::fmt;

pub mod runner;

pub const REPORT_SCHEMA_VERSION: u8 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Safe,
    Unsafe,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioKind {
    CleanApply,
    RepeatApply,
    PartialApply,
}

impl fmt::Display for ScenarioKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::CleanApply => "Clean apply",
            Self::RepeatApply => "Repeat apply",
            Self::PartialApply => "Partial-state apply",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    CommandFailed,
    NonIdempotent,
    PartialStateFailure,
    DestructiveSql,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Finding {
    pub kind: FindingKind,
    pub message: String,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ScenarioReport {
    pub scenario: ScenarioKind,
    pub status: ScenarioStatus,
    pub exit_code: Option<i32>,
    pub duration_ms: u128,
    pub sql: Vec<String>,
    pub output: String,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct GateReport {
    pub schema_version: u8,
    pub verdict: Verdict,
    pub runtime: String,
    pub image: String,
    pub duration_ms: u128,
    pub scenarios: Vec<ScenarioReport>,
}

/// Finds destructive Postgres statements in captured text.
///
/// Comments are removed before matching. Every PostgreSQL `DROP` statement is
/// considered destructive because it removes a database object. `TRUNCATE`
/// and `ALTER TABLE` / `ALTER DOMAIN` removal clauses are also blocked. This
/// deliberately avoids labeling a normal `ALTER TABLE ... ADD COLUMN` as
/// destructive.
///
/// ```
/// use migration_replay_gate::destructive_statements;
///
/// let findings = destructive_statements("ALTER TABLE users DROP COLUMN legacy_email;");
/// assert_eq!(findings, ["ALTER TABLE users DROP COLUMN legacy_email"]);
/// ```
pub fn destructive_statements(text: &str) -> Vec<String> {
    split_statements(text)
        .into_iter()
        .filter(|statement| {
            let normalized = normalize_sql(statement);
            normalized.starts_with("DROP ")
                || normalized.starts_with("TRUNCATE ")
                || ((normalized.starts_with("ALTER TABLE ")
                    || normalized.starts_with("ALTER DOMAIN "))
                    && normalized.contains(" DROP "))
        })
        .collect()
}

/// Classifies a completed scenario and attaches actionable findings.
pub fn classify_scenario(
    scenario: ScenarioKind,
    exit_code: Option<i32>,
    duration_ms: u128,
    sql: Vec<String>,
    output: String,
) -> ScenarioReport {
    let mut findings = Vec::new();
    let combined = format!("{}\n{}", sql.join("\n"), output);

    if exit_code != Some(0) {
        let kind = match scenario {
            ScenarioKind::RepeatApply => FindingKind::NonIdempotent,
            ScenarioKind::PartialApply => FindingKind::PartialStateFailure,
            ScenarioKind::CleanApply => FindingKind::CommandFailed,
        };
        let message = match scenario {
            ScenarioKind::RepeatApply => {
                "The migration command failed when replayed against its own result."
            }
            ScenarioKind::PartialApply => {
                "The migration command failed against the explicit partial-state fixture."
            }
            ScenarioKind::CleanApply => "The migration command failed on a clean seeded database.",
        };
        findings.push(Finding {
            kind,
            message: message.to_owned(),
            evidence: compact_evidence(&output),
        });
    }

    let mut seen_destructive_sql = HashSet::new();
    for statement in destructive_statements(&combined) {
        let normalized = normalize_sql(&statement);
        if !seen_destructive_sql.insert(normalized) {
            continue;
        }
        findings.push(Finding {
            kind: FindingKind::DestructiveSql,
            message: "Observed destructive DDL during replay.".to_owned(),
            evidence: Some(statement),
        });
    }

    ScenarioReport {
        scenario,
        status: if findings.is_empty() {
            ScenarioStatus::Pass
        } else {
            ScenarioStatus::Fail
        },
        exit_code,
        duration_ms,
        sql,
        output,
        findings,
    }
}

pub fn verdict_for(scenarios: &[ScenarioReport]) -> Verdict {
    if scenarios
        .iter()
        .all(|scenario| scenario.status == ScenarioStatus::Pass)
    {
        Verdict::Safe
    } else {
        Verdict::Unsafe
    }
}

pub fn extract_sql(database_logs: &str, process_output: &str) -> Vec<String> {
    let mut statements = Vec::new();
    for line in database_logs.lines().chain(process_output.lines()) {
        let trimmed = line.trim();
        let candidate = trimmed
            .split_once("statement:")
            .map(|(_, value)| value.trim())
            .unwrap_or(trimmed);
        let upper = normalize_sql(candidate);
        if [
            "CREATE ",
            "ALTER ",
            "DROP ",
            "TRUNCATE ",
            "COMMENT ",
            "GRANT ",
            "REVOKE ",
        ]
        .iter()
        .any(|prefix| upper.starts_with(prefix))
        {
            let value = candidate.trim_end_matches(';').trim().to_owned();
            if !value.is_empty() && !statements.contains(&value) {
                statements.push(value);
            }
        }
    }
    statements
}

pub fn fixture_has_destructive_sql(text: &str) -> bool {
    !destructive_statements(text).is_empty()
}

fn compact_evidence(output: &str) -> Option<String> {
    output
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(|line| line.chars().take(320).collect())
}

fn normalize_sql(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_uppercase()
}

fn split_statements(text: &str) -> Vec<String> {
    let uncommented = text
        .lines()
        .map(|line| line.split_once("--").map(|(left, _)| left).unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");
    uncommented
        .split(';')
        .flat_map(|chunk| chunk.lines())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn destructive_detection_is_high_confidence() {
        let sql = "ALTER TABLE users ADD COLUMN name text;\nALTER TABLE users DROP COLUMN email;\nDROP TABLE audit;";
        let found = destructive_statements(sql);
        assert_eq!(found.len(), 2);
        assert!(found[0].contains("DROP COLUMN"));
    }

    #[test]
    fn destructive_detection_covers_postgres_object_and_integrity_removals() {
        let sql = "
            DROP INDEX IF EXISTS idx_accounts;
            DROP VIEW account_summary;
            DROP SEQUENCE account_ids;
            ALTER TABLE accounts DROP CONSTRAINT accounts_owner_id_fkey;
            ALTER DOMAIN account_status DROP CONSTRAINT account_status_check;
            -- DROP TABLE commented_out;
            ALTER TABLE accounts ADD COLUMN handle text;
        ";

        let found = destructive_statements(sql);
        assert_eq!(found.len(), 5);
        assert!(
            found
                .iter()
                .any(|statement| statement.contains("DROP INDEX"))
        );
        assert!(
            found
                .iter()
                .any(|statement| statement.contains("DROP VIEW"))
        );
        assert!(
            found
                .iter()
                .any(|statement| statement.contains("DROP SEQUENCE"))
        );
        assert!(
            found
                .iter()
                .any(|statement| statement.contains("DROP CONSTRAINT"))
        );
    }

    #[test]
    fn destructive_findings_are_deduplicated_between_logs_and_command_output() {
        let report = classify_scenario(
            ScenarioKind::CleanApply,
            Some(0),
            5,
            vec!["DROP INDEX idx_accounts".into()],
            "DROP INDEX idx_accounts;".into(),
        );

        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].kind, FindingKind::DestructiveSql);
    }

    #[test]
    fn extracts_postgres_log_statements() {
        let logs = "2026-01-01 LOG:  statement: CREATE TABLE users(id bigint);";
        assert_eq!(extract_sql(logs, ""), vec!["CREATE TABLE users(id bigint)"]);
    }
}
