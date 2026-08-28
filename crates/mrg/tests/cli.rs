use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

#[test]
fn blocks_drop_index_emitted_by_the_migration_command_in_every_scenario() {
    let temp = TestDirectory::new("drop-index-command");
    let runtime = temp.create_fake_runtime();
    let partial = temp.write_fixture("partial.sql", "CREATE TABLE accounts (id bigint);\n");

    let output = Command::new(env!("CARGO_BIN_EXE_mrg"))
        .args([
            "gate",
            "--command",
            "printf 'DROP INDEX idx_accounts;\\n'",
            "--partial",
            partial.to_str().expect("utf8 path"),
            "--runtime",
            "docker",
            "--json",
        ])
        .env("PATH", runtime.path())
        .output()
        .expect("run mrg against fake runtime");

    assert_eq!(output.status.code(), Some(2));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON report");
    assert_eq!(report["verdict"], "unsafe");
    let scenarios = report["scenarios"].as_array().expect("scenarios array");
    assert_eq!(scenarios.len(), 3);
    for scenario in scenarios {
        let findings = scenario["findings"].as_array().expect("findings array");
        assert_eq!(
            findings.len(),
            1,
            "findings must be deduplicated: {scenario}"
        );
        assert_eq!(findings[0]["kind"], "destructive_sql");
        assert_eq!(findings[0]["evidence"], "DROP INDEX idx_accounts");
    }
}

#[test]
fn rejects_drop_index_fixture_without_explicit_acknowledgement() {
    let temp = TestDirectory::new("drop-index-fixture");
    let partial = temp.write_fixture("partial.sql", "DROP INDEX idx_accounts;\n");

    let output = Command::new(env!("CARGO_BIN_EXE_mrg"))
        .args([
            "gate",
            "--command",
            "true",
            "--partial",
            partial.to_str().expect("utf8 path"),
            "--runtime",
            "docker",
            "--json",
        ])
        .output()
        .expect("run mrg");

    assert_eq!(output.status.code(), Some(3));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("JSON error");
    assert_eq!(report["category"], "input");
    assert!(
        report["error"]
            .as_str()
            .expect("error message")
            .contains("--allow-destructive-fixtures")
    );
}

struct TestDirectory {
    path: PathBuf,
}

impl TestDirectory {
    fn new(label: &str) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("mrg-cli-{label}-{}-{nonce}", std::process::id()));
        fs::create_dir_all(&path).expect("create temporary test directory");
        Self { path }
    }

    fn write_fixture(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.path.join(name);
        fs::write(&path, contents).expect("write fixture");
        path
    }

    fn create_fake_runtime(&self) -> FakeRuntimePath {
        let bin = self.path.join("bin");
        fs::create_dir_all(&bin).expect("create fake runtime directory");
        let docker = bin.join("docker");
        fs::write(
            &docker,
            r#"#!/bin/sh
case "$1" in
  --version) exit 0 ;;
  run) printf 'fake-container\n'; exit 0 ;;
  port) printf '127.0.0.1:54321\n'; exit 0 ;;
  exec)
    case " $* " in
      *" psql "*) cat >/dev/null; exit 0 ;;
    esac
    exit 0 ;;
  logs|rm) exit 0 ;;
esac
exit 0
"#,
        )
        .expect("write fake docker");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&docker)
                .expect("fake docker metadata")
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&docker, permissions).expect("make fake docker executable");
        }
        FakeRuntimePath { bin }
    }
}

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct FakeRuntimePath {
    bin: PathBuf,
}

impl FakeRuntimePath {
    fn path(&self) -> String {
        let system_path = std::env::var("PATH").expect("system PATH");
        format!("{}:{system_path}", self.bin.display())
    }
}
