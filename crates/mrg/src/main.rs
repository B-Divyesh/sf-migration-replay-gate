use clap::{Parser, Subcommand};
use migration_replay_gate::runner::{ErrorKind, GateOptions, RuntimeChoice, run_gate};
use migration_replay_gate::{FindingKind, ScenarioStatus, Verdict};
use serde_json::json;
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::Duration;

#[derive(Debug, Parser)]
#[command(
    name = "mrg",
    version,
    about = "Replay a migration command against disposable Postgres states",
    long_about = "Migration Replay Gate starts one disposable Postgres container, seeds isolated clean and partial databases, runs your migration command in clean, repeat, and partial-state scenarios, then blocks unsafe outcomes. It never accepts an external database URL."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Run the clean, repeat, and partial-state replay gate
    Gate {
        /// Migration or schema-apply command to test
        #[arg(long, value_name = "COMMAND")]
        command: String,

        /// SQL fixture applied to both clean and partial databases (repeatable)
        #[arg(long, value_name = "FILE")]
        baseline: Vec<PathBuf>,

        /// SQL fixture describing an already-partially-applied state (repeatable, required)
        #[arg(long, value_name = "FILE", required = true)]
        partial: Vec<PathBuf>,

        /// Acknowledge and allow destructive SQL inside fixture files
        #[arg(long)]
        allow_destructive_fixtures: bool,

        /// Container runtime; auto tries Docker, then Podman
        #[arg(long, value_enum, default_value_t = RuntimeChoice::Auto)]
        runtime: RuntimeChoice,

        /// Postgres container image used only for the disposable database
        #[arg(long, default_value = "postgres:16-alpine")]
        image: String,

        /// Maximum seconds for startup and each migration command
        #[arg(long, default_value_t = 120, value_parser = clap::value_parser!(u64).range(1..=3600))]
        timeout: u64,

        /// Directory in which to run the migration command
        #[arg(long, default_value = ".")]
        working_directory: PathBuf,

        /// Emit the stable JSON report for CI
        #[arg(long)]
        json: bool,
    },
}

fn main() -> ExitCode {
    let Cli { command } = Cli::parse();
    match command {
        Commands::Gate {
            command,
            baseline,
            partial,
            allow_destructive_fixtures,
            runtime,
            image,
            timeout,
            working_directory,
            json: json_output,
        } => {
            let options = GateOptions {
                command,
                baseline,
                partial,
                allow_destructive_fixtures,
                runtime,
                image,
                timeout: Duration::from_secs(timeout),
                working_directory,
            };
            match run_gate(&options) {
                Ok(report) => {
                    if json_output {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&report).expect("serialize report")
                        );
                    } else {
                        print_human_report(&report);
                    }
                    match report.verdict {
                        Verdict::Safe => ExitCode::SUCCESS,
                        Verdict::Unsafe => ExitCode::from(2),
                    }
                }
                Err(error) => {
                    if json_output {
                        let category = match error.kind {
                            ErrorKind::Input => "input",
                            ErrorKind::Runtime => "runtime",
                        };
                        println!(
                            "{}",
                            json!({ "error": error.message, "category": category })
                        );
                    } else {
                        eprintln!("mrg: {}", error.message);
                        eprintln!("Run `mrg gate --help` for usage.");
                    }
                    match error.kind {
                        ErrorKind::Input => ExitCode::from(3),
                        ErrorKind::Runtime => ExitCode::from(4),
                    }
                }
            }
        }
    }
}

fn print_human_report(report: &migration_replay_gate::GateReport) {
    println!("Migration Replay Gate");
    println!("Runtime: {} · Image: {}", report.runtime, report.image);
    println!();
    for scenario in &report.scenarios {
        let mark = match scenario.status {
            ScenarioStatus::Pass => "PASS",
            ScenarioStatus::Fail => "FAIL",
        };
        println!(
            "[{mark}] {} ({} ms)",
            scenario.scenario, scenario.duration_ms
        );
        for finding in &scenario.findings {
            let label = match finding.kind {
                FindingKind::CommandFailed => "command failed",
                FindingKind::NonIdempotent => "non-idempotent",
                FindingKind::PartialStateFailure => "partial-state failure",
                FindingKind::DestructiveSql => "destructive SQL",
            };
            println!("       {label}: {}", finding.message);
            if let Some(evidence) = &finding.evidence {
                println!("       ↳ {evidence}");
            }
        }
        if scenario.findings.is_empty() {
            println!("       no unsafe behavior observed");
        }
    }
    println!();
    match report.verdict {
        Verdict::Safe => println!("SAFE — all replay scenarios passed"),
        Verdict::Unsafe => println!("UNSAFE — block this migration before environment apply"),
    }
}
