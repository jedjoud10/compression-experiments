use std::{io::{BufReader, Read}, marker::PhantomData};
use rayon::{iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator}, slice::ParallelSlice, *};
use bytemuck::{Pod, Zeroable};

trait Compressor {
    type Input;
    fn new() -> Self where Self: Sized;
    fn compress(&self, uncompressed: &[Self::Input], compressed: &mut Vec<u8>);
    fn decompress(&self, compressed: &[u8], uncompressed: &mut Vec<Self::Input>);
    fn alignment(&self) -> usize;
}

struct RLE<T: Pod + PartialEq> {
    _phantom: PhantomData<T>,
}

impl<T: Pod + PartialEq> Compressor for RLE<T> {
    type Input = T;
    fn compress(&self, uncompressed: &[T], compressed: &mut Vec<u8>) {        
        let mut last = Option::<&T>::None;
        let mut count = 0u64;

        for x in uncompressed {
            if Some(x) == last {
                count += 1;
            } else {
                if let Some(previous) = last.take() {
                    compressed.extend_from_slice(&u64::to_ne_bytes(count));
                    compressed.extend_from_slice(bytemuck::bytes_of(previous));
                    dbg!(count);
                    count = 0;    
                }

                last = Some(x);
                count = 1;
            }
        }

        if let Some(previous) = last.take() {
            compressed.extend_from_slice(&u64::to_ne_bytes(count));
            compressed.extend_from_slice(bytemuck::bytes_of(previous));
            dbg!(count);
        }
    }

    fn decompress(&self, compressed: &[u8], uncompressed: &mut Vec<T>) {
        let mut index = 0;
        
        while index < compressed.len() {
            let mut copy = [0u8; 8];
            copy.copy_from_slice(&compressed[index..(index + 8)]);
            let count = u64::from_ne_bytes(copy);
            dbg!(count);
            index += 8;
            
            let mut tmp = T::zeroed();
            let value_bytes = bytemuck::bytes_of_mut(&mut tmp);
            value_bytes.copy_from_slice(&compressed[index..(index + value_bytes.len())]);
            index += value_bytes.len();

            for _ in 0..count {
                uncompressed.push(tmp);
            }
        }
        
    }

    fn alignment(&self) -> usize {
        size_of::<T>()
    }

    fn new() -> Self {
        Self {
            _phantom: Default::default()
        }
    }
}


struct ParChunked<C: Compressor + Send + Sync> {
    compressor: C,
}

#[derive(Clone, Copy, Pod, Zeroable, Debug)]
#[repr(C)]
struct ChunkData {
    offset: usize,
    count: usize,
}

impl<C: Compressor + Send + Sync> Compressor for ParChunked<C> where C::Input: Send + Sync {
    type Input = C::Input;

    fn compress(&self, uncompressed: &[C::Input], compressed: &mut Vec<u8>) {
        let collected = uncompressed.par_chunks(4096).map(|chunk| {
            let mut local_compressed = Vec::<u8>::new();
            self.compressor.compress(chunk, &mut local_compressed);
            local_compressed
        }).collect::<Vec<_>>();

        let mut header = Vec::<ChunkData>::new();
        let mut offset = 0;

        for count in collected.iter().map(|c| c.len()) {
            header.push(ChunkData { offset, count });
            offset += count;
        }

        let flattened = collected.into_par_iter().flatten().collect::<Vec<u8>>();
        
        compressed.extend_from_slice(&usize::to_ne_bytes(header.len()));
        compressed.extend_from_slice(bytemuck::cast_slice(&header));
        compressed.extend_from_slice(&flattened);
    }

    fn decompress(&self, compressed: &[u8], uncompressed: &mut Vec<C::Input>) {
        let mut chunk_count_bytes = [0u8; 8];
        let mut index = 0;
        chunk_count_bytes.copy_from_slice(&compressed[index..(index + 8)]);
        index += 8;

        let chunk_count = usize::from_ne_bytes(chunk_count_bytes);
        dbg!(chunk_count);
        let bytes_to_read = chunk_count * size_of::<ChunkData>();
        let prefix_sum = bytemuck::cast_slice::<u8, ChunkData>(&compressed[index..(index + bytes_to_read)]);
        index += bytes_to_read;

        let actual_data_bruh = &compressed[index..];

        let collected = prefix_sum.into_par_iter().map(|chunk_data| {
            dbg!(chunk_data);
            let compressed_chunk = &actual_data_bruh[chunk_data.offset..(chunk_data.count + chunk_data.offset)];
            let mut local_uncompressed = Vec::<C::Input>::new();
            self.compressor.decompress(&compressed_chunk, &mut local_uncompressed);
            local_uncompressed
        }).flatten();

        
        uncompressed.par_extend(collected);
        dbg!(uncompressed.len());
    }
        
    fn alignment(&self) -> usize {
        todo!()
    }
    
    fn new() -> Self {
        Self {
            compressor: C::new(),
        }
    }
}

/*
struct Schema {
    test: Vec<Box<dyn Compressor>>,
}

impl Compressor for Schema {
    fn new() -> Self where Self: Sized {
        Schema {
            test: Vec::default(),
        }
    }

    fn compress(&self, uncompressed: &[u8], compressed: &mut Vec<u8>) {
        let mut previous_input = Vec::from(uncompressed);
        let mut previous_output = Vec::<u8>::new();
        
        for compressor in self.test.iter() {
            previous_output.clear();
            compressor.compress(&previous_input, &mut previous_output);
            
            std::mem::swap(&mut previous_input, &mut previous_output);
        }

        compressed.extend_from_slice(&previous_input);
    }

    fn decompress(&self, compressed: &[u8], buffuncompresseder: &mut Vec<u8>) {
        
    }
}

impl Schema {
    fn new() -> Schema {
        Schema { test:  Vec::new() }
    }

    fn add<T: Compressor + 'static>(&mut self) {
        self.test.push(Box::new(T::new()));
    }
    
    fn add_schema(&mut self, schema: Schema) {
        self.test.push(Box::new(schema));
    }

    fn compress(&self, input: &[u8]) -> Vec<u8> {
        let mut output = Vec::<u8>::new();
        let compressor: &dyn Compressor = self;
        compressor.compress(&input, &mut output);
        output
    }
}
*/
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
    let par_rle = ParChunked { compressor: RLE::<u8>::new() };
    let input: Vec<u8> = vec![42u8; 1_000_000];
    let mut compressed = Vec::new();
    par_rle.compress(&input, &mut compressed);
    assert!(!compressed.is_empty());
    assert!(compressed.len() < input.len());
}

#[test]
fn test_parchunked_rle_decompress_large_data() {
    let par_rle = ParChunked { compressor: RLE::<u8>::new() };
    let input: Vec<u8> = vec![42u8; 1_000_000];
    let mut compressed = Vec::new();
    par_rle.compress(&input, &mut compressed);
    
    let mut decompressed = Vec::new();
    par_rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_parchunked_rle_short_data() {
    let par_rle = ParChunked { compressor: RLE::<u8>::new() };
    let input: Vec<u8> = vec![7u8; 5];
    let mut compressed = Vec::new();
    par_rle.compress(&input, &mut compressed);
    
    let mut decompressed = Vec::new();
    par_rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_parchunked_rle_u32() {
    let par_rle = ParChunked { compressor: RLE::<u32>::new() };
    let input = vec![1u32, 1, 1, 2, 2, 3];
    let mut compressed = Vec::new();
    par_rle.compress(&input, &mut compressed);
    
    let mut decompressed = Vec::new();
    par_rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}

#[test]
fn test_parchunked_rle_custom_struct() {
    let par_rle = ParChunked { compressor: RLE::<Point>::new() };
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
    let par_rle = ParChunked { compressor: RLE::<Point>::new() };
    let input: Vec<Point> = vec![Point { x: 42, y: 99 }; 100_000];
    let mut compressed = Vec::new();
    par_rle.compress(&input, &mut compressed);
    
    let mut decompressed = Vec::new();
    par_rle.decompress(&compressed, &mut decompressed);
    assert_eq!(decompressed, input);
}
