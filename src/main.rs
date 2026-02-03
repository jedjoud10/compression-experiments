mod flexible_compression;
mod compressor;
mod algorithms;
mod tests;

use std::{any::TypeId, hash::Hash};

use bytemuck::{Pod, Zeroable};
pub use algorithms::*;

use crate::compressor::Compressor;

fn compress_into_void<C: Compressor<Input = T>, T: Pod + Zeroable + Sync + Send>(data: &[T]) -> u64 {
    let compressor = C::new();
    let mut output = Vec::<u8>::with_capacity(10000000);
    compressor.compress(&data, &mut output);
    assert!(output.len() > 0);
    output.len() as u64
}

fn compress_into_void_with<C: Compressor<Input = T>, T: Pod + Zeroable + Sync + Send>(compressor: C, data: &[T]) -> u64 {
    let mut output = Vec::<u8>::with_capacity(10000000);
    compressor.compress(&data, &mut output);
    assert!(output.len() > 0);
    output.len() as u64
}

fn pseudo_random(seed: u32) -> u32 {
    let mut value = seed;
    value ^= value << 13;
    value ^= value >> 17;
    value ^= value << 5;
    value = value.wrapping_mul(0x322adf);
    value ^= value >> 11;
    value = value.wrapping_add(0x9e3779b9);
    value
}

fn main() {
    static JUMP: usize = 10_000;

    for size in [JUMP, 2 * JUMP, 4 * JUMP, 8 * JUMP, 16 * JUMP, 32 * JUMP, 64 * JUMP].iter() {
        test_for_data_set("sequential", (0..*size).map(|i| i as u32));
        test_for_data_set("constant", (0..*size).map(|_| 420_6767_420 as u64));
        test_for_data_set("modulo", (0..*size).map(|i| (i % 52) as u64));
        test_for_data_set("pseudo-random", (0..*size).map(|i| pseudo_random(i as u32) as u32));
        test_for_data_set("sine", (0..*size).map(|i| ((i as f32 * std::f32::consts::PI / 2.0).sin() as f32 * 20.0) as i32));
    }
}

fn test_for_data_set<T: Pod + Zeroable + Send + Sync + PartialEq + Eq + Hash>(name: &str, data: impl Iterator<Item = T>) {
    let data = data.collect::<Vec<T>>();
    let rle_size = compress_into_void::<RLE<T>, T>(&data);
    let vrle_size = compress_into_void::<VRLE<T>, T>(&data);
    let par_rle_size = compress_into_void::<ParChunked<RLE<T>>, T>(&data);
    let par_vrle_size = compress_into_void::<ParChunked<VRLE<T>>, T>(&data);
    let lookup_size = compress_into_void::<Lookup<T>, T>(&data);

    let compressor = ParChunked::new_with(
        Hybrid::new()
            .add::<VRLE<T>>()
            .add::<RLE<T>>()
            .add::<Lookup<T>>()
            .add::<RLE<T, Lookup<T>>>()
            .add::<VRLE<T, Lookup<T>>>(),
    None);
    let wtf = compress_into_void_with(compressor, &data);

    let original_size = (data.len() * std::mem::size_of::<T>()) as u64;

    let type_name = std::any::type_name::<T>();
    println!("Testing '{name}' with type '{type_name}'");
    println!("Size: {} bytes", original_size);
    println!("  RLE: {:.2}%", (rle_size as f64 / original_size as f64) * 100.0);
    println!("  VRLE: {:.2}%", (vrle_size as f64 / original_size as f64) * 100.0);
    println!("  ParChunked<RLE>: {:.2}%", (par_rle_size as f64 / original_size as f64) * 100.0);
    println!("  ParChunked<VRLE>: {:.2}%", (par_vrle_size as f64 / original_size as f64) * 100.0);
    println!("  Lookup: {:.2}%", (lookup_size as f64 / original_size as f64) * 100.0);
    println!("  WTF: {:.2}%", (wtf as f64 / original_size as f64) * 100.0);
    println!();
}