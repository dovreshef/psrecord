use std::{env, io::IsTerminal};

use crate::{memory_scale::MemoryUnit, monitor::Sample};
use terminal_size::{Width, terminal_size};

const DEFAULT_NON_TTY_COLUMNS: u16 = 100;
const GRAPH_RESERVED_COLUMNS: u16 = 14;

pub fn print_graphs(samples: &[Sample]) {
    let columns = detect_terminal_columns();
    let graph_width = plot_width(samples.len(), columns);

    print_memory(samples, graph_width);
    println!();
    print_cpu(samples, graph_width);
}

fn print_memory(samples: &[Sample], graph_width: u32) {
    let peak_bytes = samples
        .iter()
        .map(|s| s.rss_bytes)
        .max()
        .unwrap_or_default();
    let unit = MemoryUnit::for_peak_bytes(peak_bytes);
    let values: Vec<f64> = samples
        .iter()
        .map(|s| unit.scale_bytes(s.rss_bytes))
        .collect();

    println!("Memory Usage ({}):", unit.label());
    let graph = rasciigraph::plot(
        values,
        rasciigraph::Config::default()
            .with_height(15)
            .with_width(graph_width)
            .with_caption(format!("RSS ({})", unit.label())),
    );
    println!("{graph}");
}

fn print_cpu(samples: &[Sample], graph_width: u32) {
    let values: Vec<f64> = samples.iter().map(|s| f64::from(s.cpu_percent)).collect();

    println!("CPU Usage (%):");
    let graph = rasciigraph::plot(
        values,
        rasciigraph::Config::default()
            .with_height(15)
            .with_width(graph_width)
            .with_caption("CPU (%)".to_string()),
    );
    println!("{graph}");
}

fn detect_terminal_columns() -> u16 {
    if std::io::stdout().is_terminal() {
        if let Some((Width(columns), _)) = terminal_size() {
            return columns;
        }
        if let Some(columns) = columns_from_env() {
            return columns;
        }
    }

    DEFAULT_NON_TTY_COLUMNS
}

fn columns_from_env() -> Option<u16> {
    env::var("COLUMNS")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .filter(|columns| *columns > 0)
}

fn plot_width(sample_count: usize, terminal_columns: u16) -> u32 {
    let available_columns = usize::from(terminal_columns.saturating_sub(GRAPH_RESERVED_COLUMNS));
    let width = available_columns.max(1).min(sample_count.max(1));

    u32::try_from(width).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::plot_width;

    #[test]
    fn limits_width_to_terminal_space_for_long_runs() {
        assert_eq!(plot_width(1_000, 80), 66);
    }

    #[test]
    fn keeps_full_sample_width_when_recording_is_short() {
        assert_eq!(plot_width(30, 120), 30);
    }

    #[test]
    fn never_returns_zero_width() {
        assert_eq!(plot_width(0, 0), 1);
    }
}
