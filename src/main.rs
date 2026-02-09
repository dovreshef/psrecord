mod ascii;
mod graph;
mod memory_scale;
mod monitor;
mod time_axis;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "psrecord", about = "Monitor process memory and CPU usage")]
struct Cli {
    /// Polling interval in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    interval: u64,

    /// Output directory for PNG graphs (default: psr-<command>-<timestamp>)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Suppress ASCII graphs on stdout
    #[arg(long)]
    no_ascii: bool,

    /// PNG width in pixels
    #[arg(long, default_value_t = 1024)]
    width: u32,

    /// PNG height in pixels
    #[arg(long, default_value_t = 768)]
    height: u32,

    /// Command to run (after --)
    #[arg(trailing_var_arg = true, required = true)]
    command: Vec<String>,
}

fn main() -> Result<ExitCode> {
    let cli = Cli::parse();
    let output_dir = resolve_output_dir(cli.output, &cli.command);

    // Create output directory eagerly (fail fast on permission errors)
    std::fs::create_dir_all(&output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            output_dir.display()
        )
    })?;

    let interval = Duration::from_millis(cli.interval);
    let result = monitor::run(&cli.command, interval)?;

    match result.exit_code {
        Some(exit_code) => eprintln!("Command exit code: {exit_code}"),
        None => eprintln!("Command terminated by signal"),
    }

    if result.samples.is_empty() {
        eprintln!("No samples collected (process exited too quickly)");
        return Ok(exit_code_from_child(result.exit_code));
    }

    eprintln!(
        "Collected {} samples over {:.1}s",
        result.samples.len(),
        result
            .samples
            .last()
            .map_or(0.0, |s| s.elapsed.as_secs_f64()),
    );

    if !cli.no_ascii {
        ascii::print_graphs(&result.samples);
    }

    graph::render_memory(
        &result.samples,
        &output_dir,
        cli.width,
        cli.height,
        &result.command_name,
    )?;
    graph::render_cpu(
        &result.samples,
        &output_dir,
        cli.width,
        cli.height,
        &result.command_name,
    )?;

    Ok(exit_code_from_child(result.exit_code))
}

fn exit_code_from_child(exit_code: Option<i32>) -> ExitCode {
    match exit_code {
        Some(code) => match u8::try_from(code) {
            Ok(status) => ExitCode::from(status),
            Err(_) => ExitCode::FAILURE,
        },
        None => ExitCode::FAILURE,
    }
}

fn resolve_output_dir(output: Option<PathBuf>, command: &[String]) -> PathBuf {
    output.unwrap_or_else(|| generated_output_dir(command, current_timestamp_millis()))
}

fn generated_output_dir(command: &[String], timestamp_millis: u128) -> PathBuf {
    let executable = command.first().map_or("cmd", String::as_str);
    let clean_name = sanitize_path_component(executable_name(executable));
    PathBuf::from(format!("psr-{clean_name}-{timestamp_millis}"))
}

fn current_timestamp_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

fn executable_name(command: &str) -> &str {
    Path::new(command)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(command)
}

fn sanitize_path_component(input: &str) -> String {
    let mut sanitized = String::new();
    let mut last_dash = false;

    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            sanitized.push('-');
            last_dash = true;
        }
    }

    let trimmed = sanitized.trim_matches('-');
    if trimmed.is_empty() {
        "cmd".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, process::ExitCode};

    use super::{
        exit_code_from_child, generated_output_dir, resolve_output_dir, sanitize_path_component,
    };

    #[test]
    fn maps_zero_exit_code_to_success() {
        assert_eq!(exit_code_from_child(Some(0)), ExitCode::SUCCESS);
    }

    #[test]
    fn maps_nonzero_exit_code() {
        assert_eq!(exit_code_from_child(Some(17)), ExitCode::from(17));
    }

    #[test]
    fn maps_signaled_process_to_failure() {
        assert_eq!(exit_code_from_child(None), ExitCode::FAILURE);
    }

    #[test]
    fn maps_negative_exit_code_to_failure() {
        assert_eq!(exit_code_from_child(Some(-1)), ExitCode::FAILURE);
    }

    #[test]
    fn maps_out_of_range_exit_code_to_failure() {
        assert_eq!(exit_code_from_child(Some(300)), ExitCode::FAILURE);
    }

    #[test]
    fn generates_default_output_dir_name() {
        let command = vec!["/usr/bin/My Tool".to_string()];
        let output = generated_output_dir(&command, 1_700_000_000_000);

        assert_eq!(output, PathBuf::from("psr-my-tool-1700000000000"));
    }

    #[test]
    fn sanitizes_empty_command_name_to_cmd() {
        assert_eq!(sanitize_path_component("!!!"), "cmd");
    }

    #[test]
    fn keeps_explicit_output_directory() {
        let explicit = PathBuf::from("custom-output");
        let command = vec!["my_command".to_string()];

        assert_eq!(
            resolve_output_dir(Some(explicit.clone()), &command),
            explicit
        );
    }
}
