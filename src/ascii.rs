use crate::{memory_scale::MemoryUnit, monitor::Sample};

pub fn print_graphs(samples: &[Sample]) {
    print_memory(samples);
    println!();
    print_cpu(samples);
}

fn print_memory(samples: &[Sample]) {
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
            .with_caption(format!("RSS ({})", unit.label())),
    );
    println!("{graph}");
}

fn print_cpu(samples: &[Sample]) {
    let values: Vec<f64> = samples.iter().map(|s| f64::from(s.cpu_percent)).collect();

    println!("CPU Usage (%):");
    let graph = rasciigraph::plot(
        values,
        rasciigraph::Config::default()
            .with_height(15)
            .with_caption("CPU (%)".to_string()),
    );
    println!("{graph}");
}
