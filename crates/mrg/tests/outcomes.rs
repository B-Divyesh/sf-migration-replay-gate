use migration_replay_gate::{
    FindingKind, ScenarioKind, ScenarioStatus, classify_scenario, verdict_for,
};

#[test]
fn known_clean_change_passes() {
    let clean = classify_scenario(
        ScenarioKind::CleanApply,
        Some(0),
        12,
        vec!["ALTER TABLE accounts ADD COLUMN handle text".into()],
        "migration complete".into(),
    );
    let repeat = classify_scenario(
        ScenarioKind::RepeatApply,
        Some(0),
        7,
        vec![],
        "already up to date".into(),
    );
    let partial = classify_scenario(
        ScenarioKind::PartialApply,
        Some(0),
        8,
        vec!["ALTER TABLE accounts ADD COLUMN handle text".into()],
        "migration complete".into(),
    );
    assert_eq!(clean.status, ScenarioStatus::Pass);
    assert_eq!(
        verdict_for(&[clean, repeat, partial]),
        migration_replay_gate::Verdict::Safe
    );
}

#[test]
fn known_duplicate_table_replay_is_non_idempotent() {
    let report = classify_scenario(
        ScenarioKind::RepeatApply,
        Some(1),
        9,
        vec!["CREATE TABLE widgets(id bigint)".into()],
        "ERROR: relation \"widgets\" already exists".into(),
    );
    assert_eq!(report.status, ScenarioStatus::Fail);
    assert_eq!(report.findings[0].kind, FindingKind::NonIdempotent);
}

#[test]
fn known_destructive_change_is_blocked_even_when_command_succeeds() {
    let report = classify_scenario(
        ScenarioKind::CleanApply,
        Some(0),
        5,
        vec!["ALTER TABLE users DROP COLUMN legacy_email".into()],
        String::new(),
    );
    assert_eq!(report.status, ScenarioStatus::Fail);
    assert!(
        report
            .findings
            .iter()
            .any(|finding| finding.kind == FindingKind::DestructiveSql)
    );
}
