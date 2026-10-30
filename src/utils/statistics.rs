//! Descriptive timing statistics and an approximate confidence interval for the mean.

#[derive(Clone, Debug)]
pub struct TimingStatistics {
    pub samples: usize,
    pub mean: f64,
    pub median: f64,
    pub p95: f64,
    pub min: u64,
    pub max: u64,
    pub stddev: Option<f64>,
    pub cv: Option<f64>,
    pub mean_ci95: Option<(f64, f64)>,
}

impl TimingStatistics {
    pub fn new(samples: &[u64]) -> Option<Self> {
        if samples.is_empty() {
            return None;
        }
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        // Accumulate in floating point so even large u64 samples cannot overflow a u64 sum.
        let mean = sorted.iter().map(|&value| value as f64).sum::<f64>() / sorted.len() as f64;
        let stddev = (sorted.len() > 1).then(|| {
            (sorted
                .iter()
                .map(|&value| (value as f64 - mean).powi(2))
                .sum::<f64>()
                / (sorted.len() - 1) as f64)
                .sqrt()
        });
        let mean_ci95 = stddev.map(|stddev| {
            let margin = critical_value(sorted.len() - 1) * stddev / (sorted.len() as f64).sqrt();
            (mean - margin, mean + margin)
        });
        Some(Self {
            samples: sorted.len(),
            mean,
            median: percentile(&sorted, 0.5),
            p95: percentile(&sorted, 0.95),
            min: sorted[0],
            max: sorted[sorted.len() - 1],
            stddev,
            cv: stddev.filter(|_| mean > 0.0).map(|stddev| stddev / mean),
            mean_ci95,
        })
    }
}

// Linear interpolation at (n - 1) * p (the R-7 quantile convention).
fn percentile(sorted: &[u64], p: f64) -> f64 {
    let position = (sorted.len() - 1) as f64 * p;
    let lower = position.floor() as usize;
    let upper = position.ceil() as usize;
    sorted[lower] as f64 + (sorted[upper] as f64 - sorted[lower] as f64) * position.fract()
}

fn critical_value(degrees_of_freedom: usize) -> f64 {
    // Two-sided 95% Student-t critical values, rounded upward. For degrees of
    // freedom between entries, use the smaller df for a conservative margin.
    const VALUES: &[(usize, f64)] = &[
        (1, 12.707),
        (2, 4.303),
        (3, 3.183),
        (4, 2.777),
        (5, 2.571),
        (6, 2.447),
        (7, 2.365),
        (8, 2.307),
        (9, 2.263),
        (10, 2.229),
        (11, 2.201),
        (12, 2.179),
        (13, 2.161),
        (14, 2.145),
        (15, 2.132),
        (16, 2.120),
        (17, 2.110),
        (18, 2.101),
        (19, 2.094),
        (20, 2.086),
        (21, 2.080),
        (22, 2.074),
        (23, 2.069),
        (24, 2.064),
        (25, 2.060),
        (26, 2.056),
        (27, 2.052),
        (28, 2.049),
        (29, 2.046),
        (30, 2.043),
        (40, 2.022),
        (60, 2.001),
        (120, 1.980),
    ];
    VALUES
        .iter()
        .rev()
        .find(|&&(df, _)| df <= degrees_of_freedom)
        .unwrap()
        .1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_distribution_keeps_fractional_statistics() {
        let stats = TimingStatistics::new(&[4, 1, 3, 2]).unwrap();
        assert_eq!(stats.samples, 4);
        assert_eq!((stats.mean, stats.median), (2.5, 2.5));
        assert!((stats.p95 - 3.85).abs() < 1e-12);
        assert_eq!((stats.min, stats.max), (1, 4));
        let stddev = (5.0_f64 / 3.0).sqrt();
        assert!((stats.stddev.unwrap() - stddev).abs() < 1e-12);
        assert!((stats.cv.unwrap() - stddev / 2.5).abs() < 1e-12);
        let (lower, upper) = stats.mean_ci95.unwrap();
        assert!((lower - (2.5 - 3.183 * stddev / 2.0)).abs() < 1e-12);
        assert!((upper - (2.5 + 3.183 * stddev / 2.0)).abs() < 1e-12);
        let fractional = TimingStatistics::new(&[1, 2]).unwrap();
        assert!(fractional.cv.unwrap() > 0.0);
        assert!(fractional.stddev.unwrap() < 1.0);
    }

    #[test]
    fn handles_empty_single_constant_and_large_samples() {
        assert!(TimingStatistics::new(&[]).is_none());
        let single = TimingStatistics::new(&[7]).unwrap();
        assert_eq!((single.mean, single.median, single.p95), (7.0, 7.0, 7.0));
        assert!(single.stddev.is_none() && single.cv.is_none() && single.mean_ci95.is_none());
        let zero = TimingStatistics::new(&[0, 0]).unwrap();
        assert_eq!(zero.stddev, Some(0.0));
        assert!(zero.cv.is_none());
        let large = TimingStatistics::new(&[u64::MAX, u64::MAX]).unwrap();
        assert_eq!(large.mean, u64::MAX as f64);
        assert_eq!(large.mean_ci95, Some((large.mean, large.mean)));
    }
}
