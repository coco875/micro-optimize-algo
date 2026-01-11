//! Benchmark utilities: data structures and CSV export.

/// Raw timing data for a single variant (used for CSV export)
pub struct RawTimingData {
    pub algo_name: String,
    pub variant_name: String,
    pub input_size: usize,
    pub avg_nanos: u64,
    pub result_sample: Option<f64>,
}

/// Export timing data to CSV file
pub fn export_csv(path: &str, data: &[RawTimingData]) -> std::io::Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;

    writeln!(file, "algorithm,variant,compiler,input_size,avg_time_ns,result")?;

    for entry in data {
        let compiler = crate::utils::C_COMPILER_NAME.unwrap_or(
            if entry.variant_name.starts_with("c-") {
                "Unknown"
            } else {
                ""
            },
        );

        writeln!(
            file,
            "{},{},{},{},{},{}",
            entry.algo_name,
            entry.variant_name,
            compiler,
            entry.input_size,
            entry.avg_nanos,
            entry.result_sample.map(|v| v.to_string()).unwrap_or_default()
        )?;
    }

    Ok(())
}
