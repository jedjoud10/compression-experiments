use crate::*;
use crate::compressor::Compressor;
use bytemuck::{Pod, Zeroable};

#[test]
fn test_rle_compress_large_data() {
    let rle = RLE::<u8>::new();
    let input: Vec<u8> = vec![42u8; 100_000];
    let mut compressed = Vec::new();
    rle.compress(&input, &mut compressed);
    assert!(!compressed.is_empty());
    assert!(compressed.len() < input.len());
}

#[test]
fn test_rle_decompress_large_data() {
    let rle = RLE::<u8>::new();
    let input: Vec<u8> = vec![42u8; 100_000];
    let mut compressed = Vec::new();
    rle.compress(&input, &mut compressed);
    
    let mut decompressed = Vec::new();
    rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_rle_u32() {
    let rle = RLE::<u32>::new();
    let input = [1u32, 1, 1, 2, 2, 3].as_slice();
    let mut compressed = Vec::new();
    rle.compress(bytemuck::cast_slice(input), &mut compressed);
    
    let mut decompressed = Vec::new();
    rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_rle_u64() {
    let rle = RLE::<u64>::new();
    let input = [100u64, 100, 200, 200, 200].as_slice();
    let mut compressed = Vec::new();
    rle.compress(bytemuck::cast_slice(input), &mut compressed);
    
    let mut decompressed = Vec::new();
    rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_vrle_compress_large_data() {
    let rle = VRLE::<u8>::new();
    let input: Vec<u8> = vec![42u8; 100_000];
    let mut compressed = Vec::new();
    rle.compress(&input, &mut compressed);
    assert!(!compressed.is_empty());
    assert!(compressed.len() < input.len());
}

#[test]
fn test_vrle_decompress_large_data() {
    let rle = VRLE::<u8>::new();
    let input: Vec<u8> = vec![42u8; 100_000];
    let mut compressed = Vec::new();
    rle.compress(&input, &mut compressed);
    
    let mut decompressed = Vec::new();
    rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_vrle_u32() {
    let rle = VRLE::<u32>::new();
    let input = [1u32, 1, 1, 2, 2, 3].as_slice();
    let mut compressed = Vec::new();
    rle.compress(bytemuck::cast_slice(input), &mut compressed);
    
    let mut decompressed = Vec::new();
    rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_vrle_u64() {
    let rle = VRLE::<u64>::new();
    let input = [100u64, 100, 200, 200, 200].as_slice();
    let mut compressed = Vec::new();
    rle.compress(bytemuck::cast_slice(input), &mut compressed);
    
    let mut decompressed = Vec::new();
    rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[repr(C)]
#[derive(Copy, Clone, Pod, PartialEq, Debug, Zeroable)]
struct Point {
    x: u32,
    y: u32,
}

#[test]
fn test_rle_custom_struct() {
    let rle = RLE::<Point>::new();
    let input = [
        Point { x: 1, y: 1 },
        Point { x: 1, y: 1 },
        Point { x: 2, y: 2 },
    ];
    let mut compressed = Vec::new();
    rle.compress(bytemuck::cast_slice(&input), &mut compressed);
    
    let mut decompressed = Vec::new();
    rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_rle_large_custom_struct() {
    let rle = RLE::<Point>::new();
    let input: Vec<Point> = vec![Point { x: 42, y: 99 }; 50_000];
    let mut compressed = Vec::new();
    rle.compress(bytemuck::cast_slice(&input), &mut compressed);
    
    let mut decompressed = Vec::new();
    rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input.as_slice());
}

#[test]
fn test_parchunked_rle_compress_large_data() {
    let par_rle = ParChunked { compressor: RLE::<u8>::new(), chunk_size: None };
    let input: Vec<u8> = vec![42u8; 1_000_000];
    let mut compressed = Vec::new();
    par_rle.compress(&input, &mut compressed);
    assert!(!compressed.is_empty());
    assert!(compressed.len() < input.len());
}

#[test]
fn test_parchunked_rle_decompress_large_data() {
    let par_rle = ParChunked { compressor: RLE::<u8>::new(), chunk_size: None };
    let input: Vec<u8> = vec![42u8; 1_000_000];
    let mut compressed = Vec::new();
    par_rle.compress(&input, &mut compressed);
    
    let mut decompressed = Vec::new();
    par_rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_parchunked_rle_short_data() {
    let par_rle = ParChunked { compressor: RLE::<u8>::new(), chunk_size: None };
    let input: Vec<u8> = vec![7u8; 5];
    let mut compressed = Vec::new();
    par_rle.compress(&input, &mut compressed);
    
    let mut decompressed = Vec::new();
    par_rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_parchunked_rle_u32() {
    let par_rle = ParChunked { compressor: RLE::<u32>::new(), chunk_size: None };
    let input = vec![1u32, 1, 1, 2, 2, 3];
    let mut compressed = Vec::new();
    par_rle.compress(&input, &mut compressed);
    
    let mut decompressed = Vec::new();
    par_rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_parchunked_rle_custom_struct() {
    let par_rle = ParChunked { compressor: RLE::<Point>::new(), chunk_size: None };
    let input = vec![
        Point { x: 1, y: 1 },
        Point { x: 1, y: 1 },
        Point { x: 2, y: 2 },
    ];
    let mut compressed = Vec::new();
    par_rle.compress(&input, &mut compressed);
    
    let mut decompressed = Vec::new();
    par_rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_parchunked_rle_large_custom_struct() {
    let par_rle = ParChunked { compressor: RLE::<Point>::new(), chunk_size: None };
    let input: Vec<Point> = vec![Point { x: 42, y: 99 }; 100_000];
    let mut compressed = Vec::new();
    par_rle.compress(&input, &mut compressed);
    
    let mut decompressed = Vec::new();
    par_rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}