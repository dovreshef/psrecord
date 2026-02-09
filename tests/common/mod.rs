use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::{SystemTime, UNIX_EPOCH},
};

pub fn psrecord_bin() -> &'static str {
    env!("CARGO_BIN_EXE_psrecord")
}

pub fn fixture_bin() -> &'static str {
    env!("CARGO_BIN_EXE_fixture_alloc_sleep")
}

pub fn unique_output_dir(test_name: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "psrecord-test-{test_name}-{timestamp}-{}",
        std::process::id()
    ))
}

pub fn run_psrecord<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(psrecord_bin())
        .args(args)
        .output()
        .expect("failed to run psrecord")
}

pub fn run_with_fixture(
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

pub fn cleanup_output_dir(output_dir: &Path) {
    let _ = fs::remove_dir_all(output_dir);
}
