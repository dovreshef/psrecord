use std::path::Path;

use anyhow::{Context, Result};
use plotters::prelude::*;

use crate::{memory_scale::MemoryUnit, monitor::Sample};

pub fn render_memory(
    samples: &[Sample],
    output_dir: &Path,
    width: u32,
    height: u32,
    command_name: &str,
) -> Result<()> {
    let path = output_dir.join("memory.png");

    let peak_bytes = samples
        .iter()
        .map(|s| s.rss_bytes)
        .max()
        .unwrap_or_default();
    let unit = MemoryUnit::for_peak_bytes(peak_bytes);

    let times: Vec<f64> = samples.iter().map(|s| s.elapsed.as_secs_f64()).collect();
    let values: Vec<f64> = samples
        .iter()
        .map(|s| unit.scale_bytes(s.rss_bytes))
        .collect();

    let x_max = times.last().copied().unwrap_or(1.0_f64);
    let y_max = values.iter().copied().fold(0.0_f64, f64::max).max(1.0_f64);

    let root = BitMapBackend::new(&path, (width, height)).into_drawing_area();
    root.fill(&WHITE).context("Failed to fill background")?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("Memory Usage — {command_name}"), ("sans-serif", 24))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(70)
        .build_cartesian_2d(0.0_f64..x_max, 0.0_f64..y_max)
        .context("Failed to build chart")?;

    chart
        .configure_mesh()
        .x_desc("Time (s)")
        .y_desc(format!("RSS ({})", unit.label()))
        .y_label_formatter(&|v| format_axis_value(*v, y_max))
        .draw()
        .context("Failed to draw mesh")?;

    chart
        .draw_series(LineSeries::new(
            times.iter().zip(values.iter()).map(|(&t, &v)| (t, v)),
            &BLUE,
        ))
        .context("Failed to draw series")?;

    root.present().context("Failed to write memory.png")?;
    eprintln!("Wrote {}", path.display());
    Ok(())
}

pub fn render_cpu(
    samples: &[Sample],
    output_dir: &Path,
    width: u32,
    height: u32,
    command_name: &str,
) -> Result<()> {
    let path = output_dir.join("cpu.png");

    let times: Vec<f32> = samples.iter().map(|s| s.elapsed.as_secs_f32()).collect();
    let values: Vec<f32> = samples.iter().map(|s| s.cpu_percent).collect();

    let x_max = times.last().copied().unwrap_or(1.0_f32);
    let y_max = values
        .iter()
        .copied()
        .fold(0.0_f32, f32::max)
        .max(100.0_f32);

    let root = BitMapBackend::new(&path, (width, height)).into_drawing_area();
    root.fill(&WHITE).context("Failed to fill background")?;

    let mut chart = ChartBuilder::on(&root)
        .caption(format!("CPU Usage — {command_name}"), ("sans-serif", 24))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(60)
        .build_cartesian_2d(0.0_f32..x_max, 0.0_f32..y_max)
        .context("Failed to build chart")?;

    chart
        .configure_mesh()
        .x_desc("Time (s)")
        .y_desc("CPU %")
        .y_label_formatter(&|v| format!("{v:.0}%"))
        .draw()
        .context("Failed to draw mesh")?;

    chart
        .draw_series(LineSeries::new(
            times.iter().zip(values.iter()).map(|(&t, &v)| (t, v)),
            &RED,
        ))
        .context("Failed to draw series")?;

    root.present().context("Failed to write cpu.png")?;
    eprintln!("Wrote {}", path.display());
    Ok(())
}

fn format_axis_value(value: f64, axis_max: f64) -> String {
    if axis_max < 10.0 {
        format!("{value:.2}")
    } else if axis_max < 100.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.0}")
    }
}

#[cfg(test)]
mod tests {
    use super::format_axis_value;

    #[test]
    fn uses_two_decimals_for_small_axis() {
        assert_eq!(format_axis_value(3.87654, 9.0), "3.88");
    }

    #[test]
    fn uses_one_decimal_for_medium_axis() {
        assert_eq!(format_axis_value(12.34, 50.0), "12.3");
    }

    #[test]
    fn uses_integer_labels_for_large_axis() {
        assert_eq!(format_axis_value(456.78, 100.0), "457");
    }
}
