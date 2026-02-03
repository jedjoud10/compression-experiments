use bytemuck::{Pod, Zeroable};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use compression_experiments::*;
use criterion::measurement::{Measurement, ValueFormatter};
use std::fmt;

/*
#[derive(Clone, Copy, Debug)]
pub struct CompressedBytes;

impl Measurement for CompressedBytes {
    type Intermediate = ();
    type Value = u64;

    fn start(&self) -> Self::Intermediate {
        ()
    }

    fn end(&self, _i: Self::Intermediate) -> Self::Value {
        // The actual value comes from iter_custom
        unreachable!("value is supplied by iter_custom")
    }

    fn add(&self, a: &Self::Value, b: &Self::Value) -> Self::Value {
        a + b
    }

    fn zero(&self) -> Self::Value {
        0
    }

    fn to_f64(&self, value: &Self::Value) -> f64 {
        *value as f64
    }

    fn formatter(&self) -> &dyn ValueFormatter {
        &BytesFormatter
    }
}

struct BytesFormatter;

impl ValueFormatter for BytesFormatter {
    fn scale_throughputs(
        &self,
        typical: f64,
        throughput: &Throughput,
        values: &mut [f64],
    ) -> &'static str {
        match *throughput {
            Throughput::Bytes(bytes) => {
                // Convert nanoseconds/iteration to bytes/half-second.
                for val in values {
                    *val = (bytes as f64) / (*val * 2f64 * 10f64.powi(-9))
                }

                "b/s/2"
            }
            Throughput::Elements(elems) => {
                for val in values {
                    *val = (elems as f64) / (*val * 2f64 * 10f64.powi(-9))
                }

                "elem/s/2"
            }
            Throughput::BytesDecimal(_) => todo!(),
        }
    }

    fn scale_values(&self, bytes: f64, values: &mut [f64]) -> &'static str {
        let (factor, unit) = if bytes < 1024f64 {
            (1f64, "b")
        } else if bytes < 1024f64.powi(2) {
            (1f64 / 1024f64, "kb")
        } else if bytes < 1024f64.powi(3) {
            (1f64 / 1024f64.powi(2), "mb")
        } else if bytes < 1024f64.powi(4) {
            (1f64 / 1024f64.powi(3), "gb")
        } else {
            (1f64 / 1024f64.powi(4), "tb")
        };

        for val in values {
            *val *= factor;
        }

        unit
    }

    fn scale_for_machines(&self, _values: &mut [f64]) -> &'static str {
        // no scaling is needed
        "what"
    }
}
*/

fn compress_into_void<C: Compressor<Input = T>, T: Pod + Zeroable + Sync + Send>(data: &[T]) -> u64 {
    let compressor = C::new();
    let mut output = Vec::<u8>::with_capacity(10000000);
    compressor.compress(black_box(&data), &mut output);
    assert!(output.len() > 0);
    output.len() as u64
}

fn decompress_into_void<C: Compressor<Input = T>, T: Pod + Zeroable + Sync + Send>(compressed: &[u8]) {
    let compressor = C::new();
    let mut output = Vec::<T>::with_capacity(10000000);
    compressor.decompress(black_box(&compressed), &mut output);
}

/*
fn alternate_measurement() -> Criterion<CompressedBytes> {
    Criterion::default().with_measurement(CompressedBytes)
}

fn criterion_benchmark_sizes(c: &mut Criterion<CompressedBytes>) {

    static JUMP: usize = 10_000;

    let mut cgroup = c.benchmark_group("compress u64 repeated sizes");
    for size in [JUMP, 2 * JUMP, 4 * JUMP, 8 * JUMP, 16 * JUMP, 32 * JUMP, 64 * JUMP].iter() {
        let data = std::iter::repeat_n(67_67_420u64 + 0xdef_baccu64, *size).collect::<Vec<_>>();

        cgroup.bench_with_input(BenchmarkId::new("COMPRESS RLE", size), size, |b, &size| {
            b.iter_custom(|_| compress_into_void::<RLE<u64>, u64>(black_box(&data)));
        });

        cgroup.bench_with_input(BenchmarkId::new("COMPRESS VRLE", size), size, |b, &size| {
            b.iter_custom(|_| compress_into_void::<VRLE<u64>, u64>(black_box(&data)));
        });

        cgroup.bench_with_input(BenchmarkId::new("COMPRESS PARCHUNKED RLE", size), size, |b, &size| {
            b.iter_custom(|_| compress_into_void::<ParChunked<RLE<u64>>, u64>(black_box(&data)));
        });

        cgroup.bench_with_input(BenchmarkId::new("COMPRESS PARCHUNKED VRLE", size), size, |b, &size| {
            b.iter_custom(|_| compress_into_void::<ParChunked<VRLE<u64>>, u64>(black_box(&data)));
        });
    }
    drop(cgroup);
}
*/

fn criterion_benchmark_times(c: &mut Criterion) {

    static JUMP: usize = 10_000;

    let mut cgroup = c.benchmark_group("compress u64 repeated");
    for size in [JUMP, 2 * JUMP, 4 * JUMP, 8 * JUMP, 16 * JUMP, 32 * JUMP, 64 * JUMP].iter() {
        let data = std::iter::repeat_n(67_67_420u64 + 0xdef_baccu64, *size).collect::<Vec<_>>();

        cgroup.bench_with_input(BenchmarkId::new("COMPRESS RLE", size), size, |b, &size| {
            b.iter(|| compress_into_void::<RLE<u64>, u64>(black_box(&data)));
        });

        cgroup.bench_with_input(BenchmarkId::new("COMPRESS VRLE", size), size, |b, &size| {
            b.iter(|| compress_into_void::<VRLE<u64>, u64>(black_box(&data)));
        });

        cgroup.bench_with_input(BenchmarkId::new("COMPRESS PARCHUNKED RLE", size), size, |b, &size| {
            b.iter(|| compress_into_void::<ParChunked<RLE<u64>>, u64>(black_box(&data)));
        });

        cgroup.bench_with_input(BenchmarkId::new("COMPRESS PARCHUNKED VRLE", size), size, |b, &size| {
            b.iter(|| compress_into_void::<ParChunked<VRLE<u64>>, u64>(black_box(&data)));
        });
    }
    drop(cgroup);

    let mut dgroup = c.benchmark_group("decompress u64 repeated");
    for size in [JUMP, 2 * JUMP, 4 * JUMP, 8 * JUMP, 16 * JUMP, 32 * JUMP, 64 * JUMP].iter() {
        let data = std::iter::repeat_n(67_67_420u64 + 0xdef_baccu64, *size).collect::<Vec<_>>();

        dgroup.bench_with_input(BenchmarkId::new("DECOMPRESS RLE", size), size, |b, &size| {
            let compressor = RLE::<u64>::new();
            let mut compressed = Vec::<u8>::with_capacity(10000000);
            compressor.compress(&data, &mut compressed);


            b.iter(|| decompress_into_void::<RLE<u64>, u64>(black_box(&compressed)));
        });

        dgroup.bench_with_input(BenchmarkId::new("DECOMPRESS VRLE", size), size, |b, &size| {
            let compressor = VRLE::<u64>::new();
            let mut compressed = Vec::<u8>::with_capacity(10000000);
            compressor.compress(&data, &mut compressed);

            b.iter(|| decompress_into_void::<VRLE<u64>, u64>(black_box(&compressed)));
        });

        dgroup.bench_with_input(BenchmarkId::new("DECOMPRESS PARCHUNKED RLE", size), size, |b, &size| {
            let compressor = ParChunked::<RLE<u64>>::new();
            let mut compressed = Vec::<u8>::with_capacity(10000000);
            compressor.compress(&data, &mut compressed);

            b.iter(|| decompress_into_void::<ParChunked<RLE<u64>>, u64>(black_box(&compressed)));
        });

        dgroup.bench_with_input(BenchmarkId::new("DECOMPRESS PARCHUNKED VRLE", size), size, |b, &size| {
            let compressor = ParChunked::<VRLE<u64>>::new();
            let mut compressed = Vec::<u8>::with_capacity(10000000);
            compressor.compress(&data, &mut compressed);

            b.iter(|| decompress_into_void::<ParChunked<VRLE<u64>>, u64>(black_box(&compressed)));
        });
    }
}

/*
criterion_group! {
    name = size_benches;
    config = alternate_measurement();
    targets = criterion_benchmark_sizes
}
*/
criterion_group!(time_benches, criterion_benchmark_times);

criterion_main!(time_benches);