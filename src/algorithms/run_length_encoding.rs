use std::{io::{BufReader, Read}, marker::PhantomData};
use rayon::{iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator}, slice::ParallelSlice, *};
use bytemuck::{Pod, Zeroable};
use crate::compressor::*;

pub struct RLE<T: Pod + PartialEq> {
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
                    count = 0;    
                }

                last = Some(x);
                count = 1;
            }
        }

        if let Some(previous) = last.take() {
            compressed.extend_from_slice(&u64::to_ne_bytes(count));
            compressed.extend_from_slice(bytemuck::bytes_of(previous));
        }
    }

    fn decompress(&self, compressed: &[u8], uncompressed: &mut Vec<T>) {
        let mut index = 0;
        
        while index < compressed.len() {
            let mut copy = [0u8; 8];
            copy.copy_from_slice(&compressed[index..(index + 8)]);
            let count = u64::from_ne_bytes(copy);
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

    fn new() -> Self {
        Self {
            _phantom: Default::default()
        }
    }
}

