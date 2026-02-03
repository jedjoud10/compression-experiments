use std::{io::{BufReader, Read}, marker::PhantomData};
use rayon::{iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator}, slice::ParallelSlice, *};
use bytemuck::{Pod, Zeroable};


pub trait Compressor {
    type Input;
    fn new() -> Self where Self: Sized;
    fn compress(&self, uncompressed: &[Self::Input], compressed: &mut Vec<u8>);
    fn decompress(&self, compressed: &[u8], uncompressed: &mut Vec<Self::Input>);
}

pub struct NaiveCompressor<T> {
    _phantom: PhantomData<T>,
}
impl<T: Pod + Zeroable> Compressor for NaiveCompressor<T> {
    type Input = T;

    fn new() -> Self where Self: Sized {
        Self { _phantom: Default::default() }
    }

    fn compress(&self, uncompressed: &[Self::Input], compressed: &mut Vec<u8>) {
        compressed.copy_from_slice(bytemuck::cast_slice(uncompressed));
    }

    fn decompress(&self, compressed: &[u8], uncompressed: &mut Vec<Self::Input>) {
        uncompressed.copy_from_slice(bytemuck::cast_slice(compressed));
    }
}