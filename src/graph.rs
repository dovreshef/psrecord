use std::path::Path;

use anyhow::{Context, Result};
use plotters::{coord::Shift, prelude::*};

use crate::{
    memory_scale::MemoryUnit,
    monitor::Sample,
    time_axis::{DEFAULT_TICK_COUNT, TimeAxis},
};

const FONT_FAMILY: &str = "DejaVu Sans";

const CANVAS_BACKGROUND: RGBColor = RGBColor(246, 248, 252);
const PLOT_BACKGROUND: RGBColor = RGBColor(255, 255, 255);
const AXIS_TEXT_COLOR: RGBColor = RGBColor(51, 65, 85);
const GRID_MAJOR_COLOR: RGBColor = RGBColor(203, 213, 225);
const GRID_MINOR_COLOR: RGBColor = RGBColor(226, 232, 240);

const MEMORY_LINE_COLOR: RGBColor = RGBColor(37, 99, 235);
const CPU_LINE_COLOR: RGBColor = RGBColor(220, 38, 38);

pub fn render_memory_png(
    samples: &[Sample],
    output_dir: &Path,
    width: u32,
    height: u32,
    command_name: &str,
) -> Result<()> {
    let path = output_dir.join("memory.png");
    let root = BitMapBackend::new(&path, (width, height)).into_drawing_area();

    render_memory_chart(&root, samples, command_name)?;

    root.present().context("Failed to write memory.png")?;
    eprintln!("Wrote {}", path.display());
    Ok(())
}

pub fn render_memory_svg(
    samples: &[Sample],
    output_dir: &Path,
    width: u32,
    height: u32,
    command_name: &str,
) -> Result<()> {
    let path = output_dir.join("memory.svg");
    let root = SVGBackend::new(&path, (width, height)).into_drawing_area();

    render_memory_chart(&root, samples, command_name)?;

    root.present().context("Failed to write memory.svg")?;
    eprintln!("Wrote {}", path.display());
    Ok(())
}

pub fn render_cpu_png(
    samples: &[Sample],
    output_dir: &Path,
    width: u32,
    height: u32,
    command_name: &str,
) -> Result<()> {
    let path = output_dir.join("cpu.png");
    let root = BitMapBackend::new(&path, (width, height)).into_drawing_area();

    render_cpu_chart(&root, samples, command_name)?;

    root.present().context("Failed to write cpu.png")?;
    eprintln!("Wrote {}", path.display());
    Ok(())
}

pub fn render_cpu_svg(
    samples: &[Sample],
    output_dir: &Path,
    width: u32,
    height: u32,
    command_name: &str,
) -> Result<()> {
    let path = output_dir.join("cpu.svg");
    let root = SVGBackend::new(&path, (width, height)).into_drawing_area();

    render_cpu_chart(&root, samples, command_name)?;

    root.present().context("Failed to write cpu.svg")?;
    eprintln!("Wrote {}", path.display());
    Ok(())
}

fn render_memory_chart<DB: DrawingBackend>(
    root: &DrawingArea<DB, Shift>,
    samples: &[Sample],
    command_name: &str,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let peak_bytes = samples
        .iter()
        .map(|sample| sample.rss_bytes)
        .max()
        .unwrap_or_default();
    let unit = MemoryUnit::for_peak_bytes(peak_bytes);

    let times: Vec<f64> = samples
        .iter()
        .map(|sample| sample.elapsed.as_secs_f64())
        .collect();
    let values: Vec<f64> = samples
        .iter()
        .map(|sample| unit.scale_bytes(sample.rss_bytes))
        .collect();
    let points: Vec<(f64, f64)> = times
        .iter()
        .zip(values.iter())
        .map(|(&time, &value)| (time, value))
        .collect();
    let time_axis = TimeAxis::from_samples(samples);

    let x_max = time_axis.map_or(1.0_f64, |axis| axis.total_seconds().max(0.001_f64));
    let y_max = add_memory_headroom(values.iter().copied().fold(0.0_f64, f64::max));

    root.fill(&CANVAS_BACKGROUND)
        .context("Failed to fill background")?;

    let mut chart = ChartBuilder::on(root)
        .caption(
            format!("Memory Usage - {command_name}"),
            (FONT_FAMILY, 34)
                .into_font()
                .style(FontStyle::Bold)
                .color(&AXIS_TEXT_COLOR),
        )
        .margin(24)
        .x_label_area_size(56)
        .y_label_area_size(84)
        .build_cartesian_2d(0.0_f64..x_max, 0.0_f64..y_max)
        .context("Failed to build chart")?;

    chart
        .plotting_area()
        .fill(&PLOT_BACKGROUND)
        .context("Failed to fill plotting area")?;

    chart
        .configure_mesh()
        .x_desc("Time")
        .axis_desc_style(
            (FONT_FAMILY, 22)
                .into_font()
                .style(FontStyle::Bold)
                .color(&AXIS_TEXT_COLOR),
        )
        .label_style((FONT_FAMILY, 15).into_font().color(&AXIS_TEXT_COLOR))
        .axis_style(ShapeStyle::from(&AXIS_TEXT_COLOR.mix(0.7)).stroke_width(2))
        .bold_line_style(ShapeStyle::from(&GRID_MAJOR_COLOR.mix(0.65)).stroke_width(1))
        .light_line_style(ShapeStyle::from(&GRID_MINOR_COLOR.mix(0.45)).stroke_width(1))
        .x_labels(DEFAULT_TICK_COUNT)
        .x_label_formatter(&|value| {
            time_axis.map_or_else(|| format!("{value:.1}s"), |axis| axis.format_label(*value))
        })
        .y_desc(format!("RSS ({})", unit.label()))
        .y_label_formatter(&|value| format_axis_value(*value, y_max))
        .draw()
        .context("Failed to draw mesh")?;

    chart
        .draw_series(LineSeries::new(
            points.iter().copied(),
            ShapeStyle::from(&MEMORY_LINE_COLOR).stroke_width(4),
        ))
        .context("Failed to draw series")?;

    if let Some((peak_time, peak_value)) = points
        .iter()
        .copied()
        .max_by(|left, right| left.1.total_cmp(&right.1))
    {
        chart
            .draw_series(std::iter::once(Circle::new(
                (peak_time, peak_value),
                5,
                ShapeStyle::from(&MEMORY_LINE_COLOR).filled(),
            )))
            .context("Failed to draw peak marker")?;
    }

    Ok(())
}

fn render_cpu_chart<DB: DrawingBackend>(
    root: &DrawingArea<DB, Shift>,
    samples: &[Sample],
    command_name: &str,
) -> Result<()>
where
    DB::ErrorType: 'static,
{
    let times: Vec<f64> = samples
        .iter()
        .map(|sample| sample.elapsed.as_secs_f64())
        .collect();
    let values: Vec<f32> = samples.iter().map(|sample| sample.cpu_percent).collect();
    let points: Vec<(f64, f32)> = times
        .iter()
        .zip(values.iter())
        .map(|(&time, &value)| (time, value))
        .collect();
    let time_axis = TimeAxis::from_samples(samples);

    let x_max = time_axis.map_or(1.0_f64, |axis| axis.total_seconds().max(0.001_f64));
    let y_max = cpu_upper_bound(values.iter().copied().fold(0.0_f32, f32::max));

    root.fill(&CANVAS_BACKGROUND)
        .context("Failed to fill background")?;

    let mut chart = ChartBuilder::on(root)
        .caption(
            format!("CPU Usage - {command_name}"),
            (FONT_FAMILY, 34)
                .into_font()
                .style(FontStyle::Bold)
                .color(&AXIS_TEXT_COLOR),
        )
        .margin(24)
        .x_label_area_size(56)
        .y_label_area_size(72)
        .build_cartesian_2d(0.0_f64..x_max, 0.0_f32..y_max)
        .context("Failed to build chart")?;

    chart
        .plotting_area()
        .fill(&PLOT_BACKGROUND)
        .context("Failed to fill plotting area")?;

    chart
        .configure_mesh()
        .x_desc("Time")
        .axis_desc_style(
            (FONT_FAMILY, 22)
                .into_font()
                .style(FontStyle::Bold)
                .color(&AXIS_TEXT_COLOR),
        )
        .label_style((FONT_FAMILY, 15).into_font().color(&AXIS_TEXT_COLOR))
        .axis_style(ShapeStyle::from(&AXIS_TEXT_COLOR.mix(0.7)).stroke_width(2))
        .bold_line_style(ShapeStyle::from(&GRID_MAJOR_COLOR.mix(0.65)).stroke_width(1))
        .light_line_style(ShapeStyle::from(&GRID_MINOR_COLOR.mix(0.45)).stroke_width(1))
        .x_labels(DEFAULT_TICK_COUNT)
        .x_label_formatter(&|value| {
            time_axis.map_or_else(|| format!("{value:.1}s"), |axis| axis.format_label(*value))
        })
        .y_desc("CPU %")
        .y_label_formatter(&|value| format!("{value:.0}%"))
        .draw()
        .context("Failed to draw mesh")?;

    chart
        .draw_series(LineSeries::new(
            points.iter().copied(),
            ShapeStyle::from(&CPU_LINE_COLOR).stroke_width(4),
        ))
        .context("Failed to draw series")?;

    if let Some((peak_time, peak_value)) = points
        .iter()
        .copied()
        .max_by(|left, right| left.1.total_cmp(&right.1))
    {
        chart
            .draw_series(std::iter::once(Circle::new(
                (peak_time, peak_value),
                5,
                ShapeStyle::from(&CPU_LINE_COLOR).filled(),
            )))
            .context("Failed to draw peak marker")?;
    }

    Ok(())
}

fn add_memory_headroom(max_value: f64) -> f64 {
    if max_value <= 1.0_f64 {
        1.0_f64
    } else {
        max_value * 1.08_f64
    }
}

fn cpu_upper_bound(max_value: f32) -> f32 {
    if max_value <= 100.0_f32 {
        100.0_f32
    } else {
        max_value * 1.08_f32
    }
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
    use super::{add_memory_headroom, cpu_upper_bound, format_axis_value};

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

    #[test]
    fn adds_headroom_for_non_trivial_memory_axes() {
        let actual = add_memory_headroom(128.0_f64);
        let expected = 138.24_f64;
        assert!((actual - expected).abs() < 1e-9_f64);
    }

    #[test]
    fn keeps_cpu_axis_at_100_for_single_core_usage() {
        let actual = cpu_upper_bound(72.0_f32);
        let expected = 100.0_f32;
        assert!((actual - expected).abs() < 1e-6_f32);
    }
}
