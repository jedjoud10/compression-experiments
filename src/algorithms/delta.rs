use std::{io::{BufReader, Read, Write}, marker::PhantomData, ops::{Add, Sub}};
use rayon::{iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator}, slice::ParallelSlice, *};
use bytemuck::{Pod, Zeroable};
use crate::compressor::*;

pub struct Delta<T: Pod + PartialEq + Sub<T, Output = T> + Add<T, Output = T>> {
    _phantom: PhantomData<T>,
}

impl<T: Pod + PartialEq + Sub<T, Output = T> + Add<T, Output = T>> Compressor for Delta<T> {
    type Input = T;
    fn compress(&self, uncompressed: &[T], compressed: &mut Vec<u8>) { 
        let mut previous = uncompressed[0];
        compressed.extend_from_slice(bytemuck::bytes_of(&[previous]));
    
        for x in uncompressed[1..].iter() {
            let delta = *x - previous;
            previous = *x;
            let slice = bytemuck::bytes_of(&delta);
            compressed.extend_from_slice(&slice);
        }
    }

    fn decompress(&self, compressed: &[u8], uncompressed: &mut Vec<T>) {
        let first = bytemuck::from_bytes::<T>(&compressed[0..(size_of::<T>())]);
        let mut index = size_of::<T>();

        while index <= compressed.len() {
            let delta = bytemuck::from_bytes::<T>(&compressed[0..(size_of::<T>())]);
            let new = *first + *delta;

            

            index += size_of::<T>();
        }
    }

    fn new() -> Self {
        Self {
            _phantom: Default::default()
        }
    }
}

