use std::{io::{BufReader, Read}, marker::PhantomData};
use rayon::{iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator}, slice::ParallelSlice, *};
use bytemuck::{Pod, Zeroable};
use crate::compressor::Compressor;

pub struct ParChunked<C: Compressor + Send + Sync> {
    pub compressor: C,
    pub chunk_size: Option<usize>,
}

impl<C: Compressor + Send + Sync> ParChunked<C> {
    pub fn new_with(compressor: C, chunk_size: Option<usize>) -> Self {
        Self {
            compressor, chunk_size
        }
    }
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
        let chunk_size = match self.chunk_size {
            Some(x) => x,
            None => {                
                let n_threads = current_num_threads();
                let chunk_size = (uncompressed.len() + n_threads - 1) / (n_threads / 2);
                chunk_size
            },
        };
        
        let collected = uncompressed.par_chunks(chunk_size).map(|chunk| {
            let mut local_compressed = Vec::<u8>::with_capacity(size_of::<Self::Input>() * chunk.len());
            self.compressor.compress(chunk, &mut local_compressed);
            local_compressed
        }).collect::<Vec<_>>();

        let mut header = Vec::<ChunkData>::new();
        let mut offset = 0;

        for count in collected.iter().map(|c| c.len()) {
            header.push(ChunkData { offset, count });
            offset += count;
        }

        compressed.extend_from_slice(&usize::to_ne_bytes(header.len()));
        compressed.extend_from_slice(bytemuck::cast_slice(&header));
        compressed.par_extend(collected.into_par_iter().flatten());
    }

    fn decompress(&self, compressed: &[u8], uncompressed: &mut Vec<C::Input>) {
        let mut chunk_count_bytes = [0u8; 8];
        let mut index = 0;
        chunk_count_bytes.copy_from_slice(&compressed[index..(index + 8)]);
        index += 8;

        let chunk_count = usize::from_ne_bytes(chunk_count_bytes);
        let bytes_to_read = chunk_count * size_of::<ChunkData>();
        let prefix_sum = bytemuck::cast_slice::<u8, ChunkData>(&compressed[index..(index + bytes_to_read)]);
        index += bytes_to_read;

        let actual_data_bruh = &compressed[index..];

        let collected = prefix_sum.into_par_iter().map(|chunk_data| {
            const COMPRESSION_FACTOR_HINT: usize = 10;

            let compressed_chunk = &actual_data_bruh[chunk_data.offset..(chunk_data.count + chunk_data.offset)];
            let mut local_uncompressed = Vec::<C::Input>::with_capacity(size_of::<Self::Input>() * compressed_chunk.len() * COMPRESSION_FACTOR_HINT);
            self.compressor.decompress(&compressed_chunk, &mut local_uncompressed);
            local_uncompressed
        }).flatten();

        
        uncompressed.par_extend(collected);
    }
    
    fn new() -> Self {
        Self {
            compressor: C::new(),
            chunk_size: None,
        }
    }
}


