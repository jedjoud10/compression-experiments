use std::{io::{BufReader, Read, Write}, marker::PhantomData, ops::Sub};
use rayon::{iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator}, slice::ParallelSlice, *};
use bytemuck::{Pod, Zeroable};
use crate::compressor::*;

pub struct Hybrid<T: Pod + PartialEq> {
    _phantom: PhantomData<T>,
    algorithms: Vec<Box<dyn Compressor<Input = T> + Sync + Send>>,
}

impl<T: Pod + PartialEq> Hybrid<T> {
    pub fn add<C: Compressor<Input = T> + 'static + Send + Sync>(mut self) -> Self {
        let boxed = Box::new(C::new());
        let boxed: Box<dyn Compressor<Input = T> + Sync + Send> = boxed; 
        self.algorithms.push(boxed);
        self
    }
}

impl<T: Pod + PartialEq> Compressor for Hybrid<T> {
    type Input = T;

    fn compress(&self, uncompressed: &[T], compressed: &mut Vec<u8>) { 
        assert!(self.algorithms.len() > 0);
        assert!(self.algorithms.len() < 255);
        let mut best_one = Vec::<u8>::new();
        let mut shortest_len = usize::MAX;
        let mut best_one_index = 0u8;

        for (i, algo) in self.algorithms.iter().enumerate() {
            let mut test_compressed = Vec::<u8>::with_capacity(compressed.capacity());
            algo.compress(uncompressed, &mut test_compressed);

            if test_compressed.len() < shortest_len {
                shortest_len = test_compressed.len();
                best_one = test_compressed;         
                best_one_index = i as u8;   
            }
        }

        compressed.push(best_one_index);
        compressed.extend_from_slice(&best_one);
    }

    fn decompress(&self, compressed: &[u8], uncompressed: &mut Vec<T>) {
        assert!(self.algorithms.len() > 0);
        assert!(self.algorithms.len() < 255);

        let best_one_index = compressed[0];
        let slice = &compressed[1..];

        let algo = &self.algorithms[best_one_index as usize];
        algo.decompress(slice, uncompressed);
    }

    fn new() -> Self {
        Self {
            _phantom: Default::default(),
            algorithms: Vec::new(),
        }
    }
}

