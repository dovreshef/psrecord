mod ascii;
mod graph;
mod memory_scale;
mod monitor;

use std::{path::PathBuf, process::ExitCode, time::Duration};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "psrecord", about = "Monitor process memory and CPU usage")]
struct Cli {
    /// Polling interval in milliseconds
    #[arg(short, long, default_value_t = 1000)]
    interval: u64,

    /// Output directory for PNG graphs
    #[arg(short, long, default_value = "psrecord-output")]
    output: PathBuf,

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

    // Create output directory eagerly (fail fast on permission errors)
    std::fs::create_dir_all(&cli.output).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            cli.output.display()
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
        &cli.output,
        cli.width,
        cli.height,
        &result.command_name,
    )?;
    graph::render_cpu(
        &result.samples,
        &cli.output,
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

#[cfg(test)]
mod tests {
    use std::process::ExitCode;

    use super::exit_code_from_child;

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
}
