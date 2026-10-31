//! Export pointwise observations from the actual Rust implementations for plotting.
use micro_optimize_algo::math::log2::{
    code::{
        atanh_series::AtanhSeries,
        library_fastapprox::{FastapproxFast, FastapproxFaster},
        library_libm::Libm,
        library_micromath::Micromath,
        linear::Linear,
        quadratic::Quadratic,
        std::Std,
    },
    log2, Log2Variant,
};
use std::io::{self, Write};

fn sample(writer: &mut impl Write, domain: &str, input: f32) -> io::Result<()> {
    let bit_approx = log2(input);
    let reference = Std::compute(input);
    let error = f64::from(bit_approx) - f64::from(reference);
    let linear = Linear::compute(input);
    let quadratic = Quadratic::compute(input);
    let series = AtanhSeries::compute(input);
    write!(
        writer,
        "{domain},{input},{bit_approx},{reference},{error},{},{linear},{quadratic},{series},{},{},{}",
        error.abs(),
        (f64::from(linear) - f64::from(reference)).abs(),
        (f64::from(quadratic) - f64::from(reference)).abs(),
        (f64::from(series) - f64::from(reference)).abs(),
    )?;
    for result in [
        Libm::compute(input),
        Micromath::compute(input),
        FastapproxFast::compute(input),
        FastapproxFaster::compute(input),
    ] {
        write!(
            writer,
            ",{result},{}",
            (f64::from(result) - f64::from(reference)).abs()
        )?;
    }
    writeln!(writer)
}

fn main() -> io::Result<()> {
    let mut writer = io::BufWriter::new(io::stdout().lock());
    writeln!(
        writer,
        "domain,input,bit_approx,std,signed_error,absolute_error,linear,quadratic,atanh_series,linear_absolute_error,quadratic_absolute_error,atanh_series_absolute_error,libm,libm_absolute_error,micromath,micromath_absolute_error,fastapprox_fast,fastapprox_fast_absolute_error,fastapprox_faster,fastapprox_faster_absolute_error"
    )?;
    // Include exact powers of two and their immediate predecessors to reveal jumps.
    for (domain, exponents, steps) in [
        ("local", 125u32..130, 2048u32),
        ("positive-normal", 1u32..255, 256u32),
    ] {
        for exponent in exponents {
            for index in 0..steps {
                let bits = (exponent << 23) | (index * ((1 << 23) / steps));
                sample(&mut writer, domain, f32::from_bits(bits))?;
            }
            sample(
                &mut writer,
                domain,
                f32::from_bits(((exponent + 1) << 23) - 1),
            )?;
        }
        if domain == "local" {
            sample(&mut writer, domain, 8.0)?;
        }
    }
    writer.flush()
}
