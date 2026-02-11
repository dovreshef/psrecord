use std::fs;

use crate::common::{
    cleanup_output_dir, fixture_bin, run_psrecord, run_with_fixture, unique_output_dir,
};

#[test]
fn help_lists_main_options() {
    let output = run_psrecord(["--help"]);
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("--interval"));
    assert!(stdout.contains("--output"));
    assert!(stdout.contains("--mode"));
    assert!(stdout.contains("--width"));
    assert!(stdout.contains("--height"));
    assert!(stdout.contains("terminal"));
    assert!(stdout.contains("png"));
    assert!(stdout.contains("svg"));
}

#[test]
fn propagates_wrapped_command_exit_code() {
    let output = run_psrecord(["--mode", "terminal", "--", fixture_bin(), "0", "0", "7"]);
    assert_eq!(output.status.code(), Some(7));
}

#[test]
fn default_mode_prints_terminal_graphs_to_stdout() {
    let output = run_psrecord([
        "--interval",
        "50",
        "--",
        fixture_bin(),
        "20000000",
        "400",
        "0",
    ]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Memory Usage ("));
    assert!(stdout.contains("CPU Usage (%)"));
}

#[test]
fn png_mode_suppresses_terminal_graph_output() {
    let output_dir = unique_output_dir("png-no-terminal");
    let output = run_with_fixture(
        &output_dir,
        &["--mode", "png", "--interval", "50"],
        &["20000000", "400", "0"],
    );

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Memory Usage ("));
    assert!(!stdout.contains("CPU Usage (%)"));

    cleanup_output_dir(&output_dir);
}

#[test]
fn writes_png_outputs_when_samples_exist() {
    let output_dir = unique_output_dir("png-output");
    let output = run_with_fixture(
        &output_dir,
        &["--mode", "png", "--interval", "50"],
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

    cleanup_output_dir(&output_dir);
}

#[test]
fn writes_svg_outputs_when_samples_exist() {
    let output_dir = unique_output_dir("svg-output");
    let output = run_with_fixture(
        &output_dir,
        &["--mode", "svg", "--interval", "50"],
        &["20000000", "400", "0"],
    );
    assert!(output.status.success());

    let memory_svg = output_dir.join("memory.svg");
    let cpu_svg = output_dir.join("cpu.svg");

    let memory_contents = fs::read_to_string(&memory_svg).expect("memory.svg should exist");
    let cpu_contents = fs::read_to_string(&cpu_svg).expect("cpu.svg should exist");

    assert!(memory_contents.contains("<svg"), "memory.svg should be SVG");
    assert!(cpu_contents.contains("<svg"), "cpu.svg should be SVG");

    cleanup_output_dir(&output_dir);
}

#[test]
fn combined_modes_generate_terminal_png_and_svg_outputs() {
    let output_dir = unique_output_dir("combined-modes");
    let output = run_with_fixture(
        &output_dir,
        &[
            "--mode",
            "term",
            "--mode",
            "png",
            "--mode",
            "svg",
            "--interval",
            "50",
        ],
        &["20000000", "400", "0"],
    );
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Memory Usage ("));
    assert!(stdout.contains("CPU Usage (%)"));

    assert!(
        output_dir.join("memory.png").exists(),
        "memory.png should exist"
    );
    assert!(output_dir.join("cpu.png").exists(), "cpu.png should exist");
    assert!(
        output_dir.join("memory.svg").exists(),
        "memory.svg should exist"
    );
    assert!(output_dir.join("cpu.svg").exists(), "cpu.svg should exist");

    cleanup_output_dir(&output_dir);
}

#[test]
fn short_lived_command_reports_no_samples() {
    let output_dir = unique_output_dir("no-samples");
    let output = run_with_fixture(&output_dir, &["--mode", "png"], &["0", "0", "0"]);
    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("No samples collected"));
    assert!(!output_dir.join("memory.png").exists());
    assert!(!output_dir.join("cpu.png").exists());

    cleanup_output_dir(&output_dir);
}

#[test]
fn memory_ascii_uses_mb_scale_for_medium_usage() {
    let output = run_psrecord([
        "--interval",
        "50",
        "--",
        fixture_bin(),
        "200000000",
        "400",
        "0",
    ]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Memory Usage (MB):"));
}

#[test]
fn default_output_uses_generated_psr_directory_name_for_image_modes() {
    let output = run_psrecord([
        "--mode",
        "png",
        "--interval",
        "50",
        "--",
        fixture_bin(),
        "20000000",
        "400",
        "0",
    ]);
    assert!(output.status.success());

    let stderr = String::from_utf8_lossy(&output.stderr);
    let memory_output = stderr
        .lines()
        .find(|line| line.contains("Wrote ") && line.contains("memory.png"))
        .expect("stderr should include memory output path");
    let memory_png = memory_output
        .strip_prefix("Wrote ")
        .expect("stderr output should start with Wrote");
    let output_dir = std::path::Path::new(memory_png)
        .parent()
        .expect("memory output path should have a parent directory");

    let dir_name = output_dir
        .file_name()
        .and_then(|name| name.to_str())
        .expect("generated output directory should be valid utf-8");
    assert!(dir_name.starts_with("psr-fixture-alloc-sleep-"));

    cleanup_output_dir(output_dir);
}
