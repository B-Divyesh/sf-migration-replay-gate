use std::process::Command;

#[test]
fn rejects_a_production_connection_string_with_documented_exit_code() {
    let fixture = format!(
        "{}/../../fixtures/example/partial.sql",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(env!("CARGO_BIN_EXE_mrg"))
        .args([
            "gate",
            "--command",
            "migrate postgresql://prod.example/app",
            "--partial",
            &fixture,
            "--json",
        ])
        .output()
        .expect("run mrg");

    assert_eq!(output.status.code(), Some(3));
    let stdout = String::from_utf8(output.stdout).expect("utf8 JSON");
    assert!(stdout.contains("\"category\":\"input\""));
    assert!(stdout.contains("disposable DATABASE_URL"));
}

#[test]
fn reports_missing_runtime_with_documented_exit_code() {
    let fixture = format!(
        "{}/../../fixtures/example/partial.sql",
        env!("CARGO_MANIFEST_DIR")
    );
    let output = Command::new(env!("CARGO_BIN_EXE_mrg"))
        .args([
            "gate",
            "--command",
            "true",
            "--partial",
            &fixture,
            "--runtime",
            "docker",
            "--json",
        ])
        .env("PATH", "/path/that/does/not/exist")
        .output()
        .expect("run mrg");

    assert_eq!(output.status.code(), Some(4));
    let stdout = String::from_utf8(output.stdout).expect("utf8 JSON");
    assert!(stdout.contains("\"category\":\"runtime\""));
}
