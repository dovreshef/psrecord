use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

use anyhow::{Context, Result, bail};
use sysinfo::{Pid, ProcessesToUpdate, System};

#[derive(Debug, Clone)]
pub struct Sample {
    pub elapsed: Duration,
    pub rss_bytes: u64,
    pub cpu_percent: f32,
}

#[derive(Debug)]
pub struct MonitorResult {
    pub samples: Vec<Sample>,
    pub exit_code: Option<i32>,
    pub command_name: String,
}

pub fn run(command: &[String], interval: Duration) -> Result<MonitorResult> {
    if command.is_empty() {
        bail!("No command provided");
    }

    let command_name = command[0].clone();
    let mut child = spawn_child(command)?;
    let pid = Pid::from_u32(child.id());

    eprintln!("Monitoring PID {} ({command_name})", child.id());

    let mut sys = System::new();
    // Seed initial CPU delta baseline
    sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
    std::thread::sleep(Duration::from_millis(100));

    let start = Instant::now();
    let mut samples = Vec::new();

    loop {
        // Check if child has exited
        match child.try_wait() {
            Ok(Some(status)) => {
                // Child exited — collect one final sample if process still visible
                sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
                if let Some(proc) = sys.process(pid) {
                    samples.push(Sample {
                        elapsed: start.elapsed(),
                        rss_bytes: proc.memory(),
                        cpu_percent: proc.cpu_usage(),
                    });
                }
                eprintln!("Process exited with status: {status}");
                return Ok(MonitorResult {
                    samples,
                    exit_code: status.code(),
                    command_name,
                });
            }
            Ok(None) => {
                // Still running — collect sample
                sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
                if let Some(proc) = sys.process(pid) {
                    samples.push(Sample {
                        elapsed: start.elapsed(),
                        rss_bytes: proc.memory(),
                        cpu_percent: proc.cpu_usage(),
                    });
                } else {
                    // sysinfo lost the process before try_wait caught exit
                    eprintln!("Lost process in sysinfo, waiting for child...");
                    let status = child.wait().context("Failed to wait for child")?;
                    eprintln!("Process exited with status: {status}");
                    return Ok(MonitorResult {
                        samples,
                        exit_code: status.code(),
                        command_name,
                    });
                }
                std::thread::sleep(interval);
            }
            Err(e) => {
                bail!("Error checking child status: {e}");
            }
        }
    }
}

fn spawn_child(command: &[String]) -> Result<Child> {
    Command::new(&command[0])
        .args(&command[1..])
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .with_context(|| format!("Failed to spawn: {}", command[0]))
}
