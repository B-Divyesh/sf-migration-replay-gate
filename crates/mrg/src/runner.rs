use crate::{
    GateReport, REPORT_SCHEMA_VERSION, ScenarioKind, classify_scenario, extract_sql,
    fixture_has_destructive_sql, verdict_for,
};
use clap::ValueEnum;
use serde::Serialize;
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum RuntimeChoice {
    Auto,
    Docker,
    Podman,
}

#[derive(Debug, Clone)]
pub struct GateOptions {
    pub command: String,
    pub baseline: Vec<PathBuf>,
    pub partial: Vec<PathBuf>,
    pub allow_destructive_fixtures: bool,
    pub runtime: RuntimeChoice,
    pub image: String,
    pub timeout: Duration,
    pub working_directory: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Input,
    Runtime,
}

#[derive(Debug)]
pub struct GateError {
    pub kind: ErrorKind,
    pub message: String,
}

impl std::fmt::Display for GateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for GateError {}

#[derive(Debug, Serialize)]
pub struct ErrorReport<'a> {
    pub error: &'a str,
    pub category: &'a str,
}

pub fn validate_options(options: &GateOptions) -> Result<(), GateError> {
    if options.command.trim().is_empty() {
        return input_error("--command cannot be empty");
    }
    let lower = options.command.to_ascii_lowercase();
    if ["postgres://", "postgresql://", "database_url=", "pghost="]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        return input_error(
            "the command contains a database connection target; remove it and use the injected disposable DATABASE_URL",
        );
    }
    if options.partial.is_empty() {
        return input_error("at least one explicit --partial fixture is required");
    }
    if !options.working_directory.is_dir() {
        return input_error("--working-directory must name an existing directory");
    }

    for path in options.baseline.iter().chain(options.partial.iter()) {
        let sql = fs::read_to_string(path).map_err(|error| GateError {
            kind: ErrorKind::Input,
            message: format!("could not read fixture {}: {error}", path.display()),
        })?;
        if fixture_has_destructive_sql(&sql) && !options.allow_destructive_fixtures {
            return input_error(format!(
                "fixture {} contains destructive SQL; review it and pass --allow-destructive-fixtures explicitly",
                path.display()
            ));
        }
    }
    Ok(())
}

pub fn run_gate(options: &GateOptions) -> Result<GateReport, GateError> {
    validate_options(options)?;
    let started = Instant::now();
    let engine = resolve_runtime(options.runtime)?;
    let suffix = unique_suffix();
    let container_name = format!("mrg-{suffix}");
    let password = format!("mrg_{suffix}");
    let mut container = Container::start(
        &engine,
        &container_name,
        &password,
        &options.image,
        options.timeout,
    )?;

    container.create_database("gate_clean")?;
    container.create_database("gate_partial")?;
    for fixture in &options.baseline {
        container.apply_fixture("gate_clean", fixture)?;
        container.apply_fixture("gate_partial", fixture)?;
    }
    for fixture in &options.partial {
        container.apply_fixture("gate_partial", fixture)?;
    }

    let mut scenarios = Vec::new();
    scenarios.push(container.run_scenario(
        ScenarioKind::CleanApply,
        "gate_clean",
        options,
        &password,
    )?);
    scenarios.push(container.run_scenario(
        ScenarioKind::RepeatApply,
        "gate_clean",
        options,
        &password,
    )?);
    scenarios.push(container.run_scenario(
        ScenarioKind::PartialApply,
        "gate_partial",
        options,
        &password,
    )?);

    container.stop();
    Ok(GateReport {
        schema_version: REPORT_SCHEMA_VERSION,
        verdict: verdict_for(&scenarios),
        runtime: engine,
        image: options.image.clone(),
        duration_ms: started.elapsed().as_millis(),
        scenarios,
    })
}

struct Container {
    engine: String,
    name: String,
    host_port: u16,
    stopped: bool,
}

impl Container {
    fn start(
        engine: &str,
        name: &str,
        password: &str,
        image: &str,
        timeout: Duration,
    ) -> Result<Self, GateError> {
        let output = Command::new(engine)
            .args([
                "run",
                "-d",
                "--rm",
                "--name",
                name,
                "--label",
                "dev.sociobot.mrg.disposable=true",
                "-e",
                &format!("POSTGRES_PASSWORD={password}"),
                "-e",
                "POSTGRES_USER=postgres",
                "-p",
                "127.0.0.1::5432",
                image,
                "postgres",
                "-c",
                "log_statement=ddl",
                "-c",
                "log_min_messages=info",
            ])
            .output()
            .map_err(|error| runtime_error(format!("could not start {engine}: {error}")))?;
        if !output.status.success() {
            return Err(runtime_error(format!(
                "{engine} could not start disposable Postgres: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )));
        }

        let mut container = Self {
            engine: engine.to_owned(),
            name: name.to_owned(),
            host_port: 0,
            stopped: false,
        };

        let result = (|| {
            container.host_port = container.discover_port()?;
            let deadline = Instant::now() + timeout.min(Duration::from_secs(120));
            loop {
                let ready = Command::new(&container.engine)
                    .args(["exec", &container.name, "pg_isready", "-U", "postgres"])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status()
                    .map(|status| status.success())
                    .unwrap_or(false);
                if ready {
                    break;
                }
                if Instant::now() >= deadline {
                    return Err(runtime_error(
                        "disposable Postgres did not become ready before timeout",
                    ));
                }
                thread::sleep(Duration::from_millis(250));
            }
            Ok(())
        })();
        if result.is_err() {
            container.stop();
        }
        result.map(|_| container)
    }

    fn discover_port(&self) -> Result<u16, GateError> {
        for _ in 0..20 {
            let output = Command::new(&self.engine)
                .args(["port", &self.name, "5432/tcp"])
                .output()
                .map_err(|error| {
                    runtime_error(format!("could not inspect container port: {error}"))
                })?;
            let text = String::from_utf8_lossy(&output.stdout);
            if let Some(port) = parse_port(&text) {
                return Ok(port);
            }
            thread::sleep(Duration::from_millis(100));
        }
        Err(runtime_error(
            "container runtime did not publish a loopback Postgres port",
        ))
    }

    fn create_database(&self, database: &str) -> Result<(), GateError> {
        let output = Command::new(&self.engine)
            .args(["exec", &self.name, "createdb", "-U", "postgres", database])
            .output()
            .map_err(|error| runtime_error(format!("could not create test database: {error}")))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(runtime_error(format!(
                "could not create test database: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            )))
        }
    }

    fn apply_fixture(&self, database: &str, path: &Path) -> Result<(), GateError> {
        let sql = fs::read(path).map_err(|error| GateError {
            kind: ErrorKind::Input,
            message: format!("could not read fixture {}: {error}", path.display()),
        })?;
        let mut child = Command::new(&self.engine)
            .args([
                "exec",
                "-i",
                &self.name,
                "psql",
                "-v",
                "ON_ERROR_STOP=1",
                "-U",
                "postgres",
                "-d",
                database,
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| runtime_error(format!("could not seed fixture: {error}")))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(&sql)
                .map_err(|error| runtime_error(format!("could not send fixture SQL: {error}")))?;
        }
        let output = child
            .wait_with_output()
            .map_err(|error| runtime_error(format!("could not finish fixture: {error}")))?;
        if output.status.success() {
            Ok(())
        } else {
            Err(GateError {
                kind: ErrorKind::Input,
                message: format!(
                    "fixture {} failed in its disposable database: {}",
                    path.display(),
                    String::from_utf8_lossy(&output.stderr).trim()
                ),
            })
        }
    }

    fn run_scenario(
        &self,
        kind: ScenarioKind,
        database: &str,
        options: &GateOptions,
        password: &str,
    ) -> Result<crate::ScenarioReport, GateError> {
        let logs_before = self.logs()?;
        let started = Instant::now();
        let command_result = run_user_command(
            &options.command,
            &options.working_directory,
            self.host_port,
            database,
            password,
            options.timeout,
        )?;
        let duration_ms = started.elapsed().as_millis();
        let logs_after = self.logs()?;
        let new_logs = logs_after
            .strip_prefix(&logs_before)
            .unwrap_or(&logs_after)
            .to_owned();
        let redacted_output = redact(&command_result.output, self.host_port, password);
        let sql = extract_sql(&new_logs, &redacted_output);
        Ok(classify_scenario(
            kind,
            command_result.status.code(),
            duration_ms,
            sql,
            redacted_output,
        ))
    }

    fn logs(&self) -> Result<String, GateError> {
        let output = Command::new(&self.engine)
            .args(["logs", &self.name])
            .output()
            .map_err(|error| runtime_error(format!("could not read Postgres logs: {error}")))?;
        let mut text = String::from_utf8_lossy(&output.stdout).into_owned();
        text.push_str(&String::from_utf8_lossy(&output.stderr));
        Ok(text)
    }

    fn stop(&mut self) {
        if self.stopped {
            return;
        }
        let _ = Command::new(&self.engine)
            .args(["rm", "-f", &self.name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        self.stopped = true;
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        self.stop();
    }
}

struct CommandResult {
    status: ExitStatus,
    output: String,
}

fn run_user_command(
    command: &str,
    directory: &Path,
    port: u16,
    database: &str,
    password: &str,
    timeout: Duration,
) -> Result<CommandResult, GateError> {
    let suffix = unique_suffix();
    let stdout_path = env::temp_dir().join(format!("mrg-{suffix}-stdout.log"));
    let stderr_path = env::temp_dir().join(format!("mrg-{suffix}-stderr.log"));
    let stdout_file = File::create(&stdout_path)
        .map_err(|error| runtime_error(format!("could not create output capture: {error}")))?;
    let stderr_file = File::create(&stderr_path)
        .map_err(|error| runtime_error(format!("could not create error capture: {error}")))?;

    let url = format!("postgresql://postgres:{password}@127.0.0.1:{port}/{database}");
    let mut process = shell_command(command);
    process
        .current_dir(directory)
        .env_remove("PGSERVICE")
        .env_remove("PGSERVICEFILE")
        .env_remove("DIRECT_URL")
        .env_remove("SHADOW_DATABASE_URL")
        .env_remove("DB_URL")
        .env("DATABASE_URL", &url)
        .env("PGHOST", "127.0.0.1")
        .env("PGPORT", port.to_string())
        .env("PGDATABASE", database)
        .env("PGUSER", "postgres")
        .env("PGPASSWORD", password)
        .env("MRG_SCENARIO", database)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));
    let mut child = process
        .spawn()
        .map_err(|error| runtime_error(format!("could not run migration command: {error}")))?;
    let deadline = Instant::now() + timeout;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|error| {
            runtime_error(format!("could not wait for migration command: {error}"))
        })? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let status = child.wait().map_err(|error| {
                runtime_error(format!("could not stop timed-out command: {error}"))
            })?;
            let _ = append_timeout(&stderr_path, timeout);
            break status;
        }
        thread::sleep(Duration::from_millis(50));
    };

    let mut output = read_capture(&stdout_path);
    let errors = read_capture(&stderr_path);
    if !errors.trim().is_empty() {
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(&errors);
    }
    let _ = fs::remove_file(stdout_path);
    let _ = fs::remove_file(stderr_path);
    Ok(CommandResult { status, output })
}

#[cfg(unix)]
fn shell_command(command: &str) -> Command {
    let mut process = Command::new("sh");
    process.args(["-c", command]);
    process
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut process = Command::new("cmd");
    process.args(["/C", command]);
    process
}

fn append_timeout(path: &Path, timeout: Duration) -> std::io::Result<()> {
    let mut file = fs::OpenOptions::new().append(true).open(path)?;
    writeln!(
        file,
        "Migration Replay Gate stopped the command after {} seconds.",
        timeout.as_secs()
    )
}

fn read_capture(path: &Path) -> String {
    let mut output = String::new();
    if let Ok(file) = File::open(path) {
        let _ = file.take(1_000_000).read_to_string(&mut output);
    }
    output
}

fn resolve_runtime(choice: RuntimeChoice) -> Result<String, GateError> {
    let candidates: &[&str] = match choice {
        RuntimeChoice::Auto => &["docker", "podman"],
        RuntimeChoice::Docker => &["docker"],
        RuntimeChoice::Podman => &["podman"],
    };
    candidates
        .iter()
        .find(|candidate| {
            Command::new(candidate)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .map(|status| status.success())
                .unwrap_or(false)
        })
        .map(|value| (*value).to_owned())
        .ok_or_else(|| GateError {
            kind: ErrorKind::Runtime,
            message: "no supported container runtime found; install Docker or Podman".to_owned(),
        })
}

fn parse_port(value: &str) -> Option<u16> {
    value
        .lines()
        .find_map(|line| line.rsplit(':').next()?.trim().parse().ok())
}

fn redact(value: &str, port: u16, password: &str) -> String {
    value
        .replace(password, "[ephemeral-password]")
        .replace(&format!("127.0.0.1:{port}"), "127.0.0.1:[ephemeral-port]")
}

fn unique_suffix() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("{}-{millis}", std::process::id())
}

fn input_error<T>(message: impl Into<String>) -> Result<T, GateError> {
    Err(GateError {
        kind: ErrorKind::Input,
        message: message.into(),
    })
}

fn runtime_error(message: impl Into<String>) -> GateError {
    GateError {
        kind: ErrorKind::Runtime,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn options(command: &str, partial: Vec<PathBuf>) -> GateOptions {
        GateOptions {
            command: command.to_owned(),
            baseline: Vec::new(),
            partial,
            allow_destructive_fixtures: false,
            runtime: RuntimeChoice::Auto,
            image: "postgres:16-alpine".to_owned(),
            timeout: Duration::from_secs(1),
            working_directory: env::current_dir().unwrap(),
        }
    }

    #[test]
    fn parses_docker_and_podman_ports() {
        assert_eq!(parse_port("127.0.0.1:49152\n"), Some(49152));
        assert_eq!(parse_port("0.0.0.0:38211\n[::]:38211"), Some(38211));
    }

    #[test]
    fn refuses_embedded_database_target() {
        let result = validate_options(&options(
            "tool --database postgresql://prod.example/db",
            vec![PathBuf::from("missing")],
        ));
        assert_eq!(result.unwrap_err().kind, ErrorKind::Input);
    }

    #[test]
    fn requires_partial_fixture() {
        let result = validate_options(&options("tool migrate", vec![]));
        assert!(result.unwrap_err().message.contains("--partial"));
    }
}
