use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

fn psrecord_bin() -> &'static str {
    env!("CARGO_BIN_EXE_psrecord")
}

fn fixture_bin() -> &'static str {
    env!("CARGO_BIN_EXE_fixture_alloc_sleep")
}

fn unique_output_dir(test_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "psrecord-test-{test_name}-{timestamp}-{}",
        std::process::id()
    ))
}

fn run_psrecord<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(psrecord_bin())
        .args(args)
        .output()
        .expect("failed to run psrecord")
}

fn run_with_fixture(
    output_dir: &Path,
    extra_psrecord_args: &[&str],
    fixture_args: &[&str],
) -> Output {
    let mut cmd = Command::new(psrecord_bin());
    cmd.args(extra_psrecord_args)
        .arg("--output")
        .arg(output_dir)
        .arg("--")
        .arg(fixture_bin())
        .args(fixture_args);
    cmd.output().expect("failed to run psrecord with fixture")
}

#[test]
fn help_lists_main_options() {
    let output = run_psrecord(["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--interval"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--no-ascii"));
    assert!(stdout.contains("--width"));
    assert!(stdout.contains("--height"));
}

#[test]
fn propagates_wrapped_command_exit_code() {
    let output_dir = unique_output_dir("propagates-exit-code");
    let output = run_with_fixture(&output_dir, &["--no-ascii"], &["0", "0", "7"]);

    assert_eq!(output.status.code(), Some(7));
    let _ = fs::remove_dir_all(&output_dir);
}

#[test]
fn no_ascii_flag_suppresses_ascii_graph_output() {
    let output_dir = unique_output_dir("no-ascii");
    let output = run_with_fixture(
        &output_dir,
        &["--no-ascii", "--interval", "50"],
        &["20000000", "400", "0"],
    );

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Memory Usage ("));
    assert!(!stdout.contains("CPU Usage (%)"));

    let _ = fs::remove_dir_all(&output_dir);
}

#[test]
fn default_mode_prints_ascii_graphs_to_stdout() {
    let output_dir = unique_output_dir("ascii-default");
    let output = run_with_fixture(
        &output_dir,
        &["--interval", "50"],
        &["20000000", "400", "0"],
    );

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Memory Usage ("));
    assert!(stdout.contains("CPU Usage (%)"));

    let _ = fs::remove_dir_all(&output_dir);
}

#[test]
fn writes_png_outputs_when_samples_exist() {
    let output_dir = unique_output_dir("png-output");
    let output = run_with_fixture(
        &output_dir,
        &["--no-ascii", "--interval", "50"],
        &["20000000", "400", "0"],
    );
    assert!(output.status.success());

    let memory_png = output_dir.join("memory.png");
    let cpu_png = output_dir.join("cpu.png");

    let memory_size = fs::metadata(&memory_png)
        .expect("memory.png should exist")
        .len();
    let cpu_size = fs::metadata(&cpu_png).expect("cpu.png should exist").len();

    assert!(memory_size > 0, "memory.png should be non-empty");
    assert!(cpu_size > 0, "cpu.png should be non-empty");

    let _ = fs::remove_dir_all(&output_dir);
}

#[test]
fn short_lived_command_reports_no_samples() {
    let output_dir = unique_output_dir("no-samples");
    let output = run_with_fixture(&output_dir, &["--no-ascii"], &["0", "0", "0"]);
    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No samples collected"));

    let _ = fs::remove_dir_all(&output_dir);
}

#[test]
fn memory_ascii_uses_mb_scale_for_medium_usage() {
    let output_dir = unique_output_dir("mb-scale");
    let output = run_with_fixture(
        &output_dir,
        &["--interval", "50"],
        &["200000000", "400", "0"],
    );

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Memory Usage (MB):"));

    let _ = fs::remove_dir_all(&output_dir);
}
