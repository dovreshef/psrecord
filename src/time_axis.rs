use std::time::Duration;

use crate::monitor::Sample;

pub const DEFAULT_TICK_COUNT: usize = 5;

#[derive(Debug, Clone, Copy)]
pub struct TimeAxis {
    total_seconds: f64,
    average_step_seconds: f64,
}

impl TimeAxis {
    pub fn from_samples(samples: &[Sample]) -> Option<Self> {
        let total_seconds = samples.last()?.elapsed.as_secs_f64();
        let average_step_seconds = average_sample_interval_seconds(samples);

        Some(Self {
            total_seconds: total_seconds.max(0.0),
            average_step_seconds,
        })
    }

    pub fn total_seconds(self) -> f64 {
        self.total_seconds
    }

    pub fn format_label(self, seconds: f64) -> String {
        format_duration_label(seconds, self.total_seconds, self.average_step_seconds)
    }
}

pub fn tick_positions(width: usize, tick_count: usize) -> Vec<usize> {
    if width <= 1 {
        return vec![0];
    }

    let mut positions = Vec::with_capacity(tick_count);
    let last = width.saturating_sub(1);
    let denominator = tick_count.saturating_sub(1).max(1);
    for index in 0..tick_count {
        positions.push(index.saturating_mul(last) / denominator);
    }
    positions.dedup();
    positions
}

pub fn scaled_tick_seconds(position: usize, width: usize, total_seconds: f64) -> f64 {
    if width <= 1 {
        return total_seconds.max(0.0);
    }

    let position_f64 = u32::try_from(position).map_or(f64::from(u32::MAX), f64::from);
    let denominator = u32::try_from(width.saturating_sub(1)).map_or(1.0, f64::from);

    (total_seconds.max(0.0) * position_f64) / denominator
}

fn average_sample_interval_seconds(samples: &[Sample]) -> f64 {
    let mut total_delta = 0.0_f64;
    let mut count = 0_u32;

    for window in samples.windows(2) {
        let delta = window[1]
            .elapsed
            .saturating_sub(window[0].elapsed)
            .as_secs_f64();
        if delta > 0.0 {
            total_delta += delta;
            count = count.saturating_add(1);
        }
    }

    if count == 0 {
        return samples
            .first()
            .map_or(1.0, |sample| sample.elapsed.as_secs_f64().max(0.001));
    }

    total_delta / f64::from(count)
}

fn format_duration_label(seconds: f64, total_seconds: f64, average_step_seconds: f64) -> String {
    let clamped_seconds = seconds.max(0.0);
    if total_seconds < 1.0 {
        let precision = if average_step_seconds < 0.001 {
            2
        } else {
            usize::from(average_step_seconds < 0.01)
        };
        let millis = clamped_seconds * 1000.0;
        return format!("{millis:.precision$}ms");
    }

    if total_seconds < 120.0 {
        let precision = if average_step_seconds < 0.01 {
            2
        } else {
            usize::from(average_step_seconds < 0.1)
        };
        return format!("{clamped_seconds:.precision$}s");
    }

    if total_seconds < 3600.0 {
        return format_minutes_and_seconds(clamped_seconds);
    }

    format_hours_minutes_and_seconds(clamped_seconds)
}

fn format_minutes_and_seconds(seconds: f64) -> String {
    let rounded = rounded_seconds(seconds);
    let minutes = rounded / 60;
    let seconds_only = rounded % 60;
    format!("{minutes}:{seconds_only:02}")
}

fn format_hours_minutes_and_seconds(seconds: f64) -> String {
    let rounded = rounded_seconds(seconds);
    let hours = rounded / 3600;
    let minutes = (rounded % 3600) / 60;
    let seconds_only = rounded % 60;
    format!("{hours}:{minutes:02}:{seconds_only:02}")
}

fn rounded_seconds(seconds: f64) -> u64 {
    Duration::from_secs_f64((seconds + 0.5).max(0.0)).as_secs()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::{Sample, TimeAxis, tick_positions};

    #[test]
    fn formats_subsecond_timeline_labels_in_milliseconds() {
        let axis = axis_for_total(Duration::from_millis(500));
        assert_eq!(axis.format_label(0.125), "125ms");
    }

    #[test]
    fn formats_short_timeline_labels_in_seconds() {
        let axis = axis_for_total(Duration::from_secs(10));
        assert_eq!(axis.format_label(1.25), "1s");
    }

    #[test]
    fn formats_medium_timeline_labels_as_minute_second() {
        let axis = axis_for_total(Duration::from_secs(600));
        assert_eq!(axis.format_label(125.0), "2:05");
    }

    #[test]
    fn formats_long_timeline_labels_as_hour_minute_second() {
        let axis = axis_for_total(Duration::from_secs(7_200));
        assert_eq!(axis.format_label(3_723.0), "1:02:03");
    }

    #[test]
    fn creates_start_and_end_tick_positions() {
        assert_eq!(tick_positions(20, 5), vec![0, 4, 9, 14, 19]);
    }

    fn axis_for_total(total: Duration) -> TimeAxis {
        let samples = vec![
            Sample {
                elapsed: Duration::from_millis(0),
                rss_bytes: 0,
                cpu_percent: 0.0,
            },
            Sample {
                elapsed: total,
                rss_bytes: 0,
                cpu_percent: 0.0,
            },
        ];

        match TimeAxis::from_samples(&samples) {
            Some(axis) => axis,
            None => panic!("samples should produce an axis"),
        }
    }
}
