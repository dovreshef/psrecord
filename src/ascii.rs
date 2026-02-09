use std::{env, io::IsTerminal};

use crate::{
    memory_scale::MemoryUnit,
    monitor::Sample,
    time_axis::{DEFAULT_TICK_COUNT, TimeAxis, scaled_tick_seconds, tick_positions},
};
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
    print_timeline(samples, &graph, graph_width);
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
    print_timeline(samples, &graph, graph_width);
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

fn plot_width(_sample_count: usize, terminal_columns: u16) -> u32 {
    let width = usize::from(terminal_columns.saturating_sub(GRAPH_RESERVED_COLUMNS)).max(1);

    u32::try_from(width).unwrap_or(u32::MAX)
}

fn print_timeline(samples: &[Sample], graph: &str, graph_width: u32) {
    if let Some((ticks, labels)) = timeline_lines(samples, graph, graph_width) {
        println!("{ticks}");
        println!("{labels}");
    }
}

fn timeline_lines(samples: &[Sample], graph: &str, graph_width: u32) -> Option<(String, String)> {
    let width = usize::try_from(graph_width).ok()?.max(1);
    let time_axis = TimeAxis::from_samples(samples)?;

    let axis_column = graph_axis_column(graph).unwrap_or_default();
    let data_start = axis_column.saturating_add(1);

    let mut tick_data = vec![' '; width];
    if width == 1 {
        tick_data[0] = '┬';
    } else {
        tick_data.fill('─');
        for position in tick_positions(width, DEFAULT_TICK_COUNT) {
            if let Some(ch) = tick_data.get_mut(position) {
                *ch = '┬';
            }
        }
    }

    let mut label_data = vec![' '; width];
    let mut occupied = vec![false; width];
    let axis_tick_positions = tick_positions(width, DEFAULT_TICK_COUNT);

    for (index, position) in axis_tick_positions.iter().copied().enumerate() {
        let tick_seconds = scaled_tick_seconds(position, width, time_axis.total_seconds());
        let label = time_axis.format_label(tick_seconds);
        let force = index == 0 || index + 1 == axis_tick_positions.len();
        place_label(&mut label_data, &mut occupied, position, &label, force);
    }

    let mut ticks = vec![' '; data_start + width];
    let mut labels = vec![' '; data_start + width];

    for (index, ch) in tick_data.into_iter().enumerate() {
        ticks[data_start + index] = ch;
    }
    for (index, ch) in label_data.into_iter().enumerate() {
        labels[data_start + index] = ch;
    }

    Some((ticks.into_iter().collect(), labels.into_iter().collect()))
}

fn graph_axis_column(graph: &str) -> Option<usize> {
    graph
        .lines()
        .find_map(|line| line.chars().position(|ch| matches!(ch, '┤' | '┼')))
}

fn place_label(line: &mut [char], occupied: &mut [bool], center: usize, label: &str, force: bool) {
    let label_chars: Vec<char> = label.chars().collect();
    if label_chars.is_empty() || label_chars.len() > line.len() {
        return;
    }

    let half = label_chars.len() / 2;
    let max_start = line.len().saturating_sub(label_chars.len());
    let start = center.saturating_sub(half).min(max_start);
    let end = start + label_chars.len();

    if !force && occupied[start..end].iter().any(|is_used| *is_used) {
        return;
    }

    for (offset, ch) in label_chars.into_iter().enumerate() {
        line[start + offset] = ch;
        occupied[start + offset] = true;
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Sample, plot_width, timeline_lines};

    #[test]
    fn limits_width_to_terminal_space_for_long_runs() {
        assert_eq!(plot_width(1_000, 80), 66);
    }

    #[test]
    fn expands_short_recordings_to_terminal_width() {
        assert_eq!(plot_width(30, 120), 106);
    }

    #[test]
    fn never_returns_zero_width() {
        assert_eq!(plot_width(0, 0), 1);
    }

    #[test]
    fn timeline_includes_start_and_end_time_labels() {
        let samples = vec![
            sample_at(Duration::from_millis(0)),
            sample_at(Duration::from_millis(500)),
            sample_at(Duration::from_millis(1_000)),
        ];
        let graph = " 1.0 ┤....................\n 0.0 ┼....................";

        let Some((ticks, labels)) = timeline_lines(&samples, graph, 20) else {
            panic!("timeline should be generated");
        };

        assert!(ticks.contains('┬'));
        assert!(labels.contains("0s"));
        assert!(labels.contains("1s"));
    }

    fn sample_at(elapsed: Duration) -> Sample {
        Sample {
            elapsed,
            rss_bytes: 0,
            cpu_percent: 0.0,
        }
    }
}
