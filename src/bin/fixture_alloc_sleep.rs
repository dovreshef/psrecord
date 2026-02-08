use std::{env, hint::black_box, process::ExitCode, thread, time::Duration};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: fixture_alloc_sleep <bytes> <sleep_ms> [exit_code]");
        return ExitCode::from(2);
    }

    let Ok(bytes) = args[1].parse::<usize>() else {
        eprintln!("Invalid bytes: {}", args[1]);
        return ExitCode::from(2);
    };

    let Ok(sleep_ms) = args[2].parse::<u64>() else {
        eprintln!("Invalid sleep_ms: {}", args[2]);
        return ExitCode::from(2);
    };

    let requested_exit = if args.len() >= 4 {
        let Ok(parsed_exit) = args[3].parse::<u8>() else {
            eprintln!("Invalid exit_code: {}", args[3]);
            return ExitCode::from(2);
        };
        parsed_exit
    } else {
        0
    };

    let mut buffer = vec![0_u8; bytes];
    for index in (0..buffer.len()).step_by(4096) {
        buffer[index] = 1;
    }
    black_box(&buffer);

    thread::sleep(Duration::from_millis(sleep_ms));
    ExitCode::from(requested_exit)
}
