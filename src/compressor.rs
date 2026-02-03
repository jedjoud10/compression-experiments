use std::{io::{BufReader, Read}, marker::PhantomData};
use rayon::{iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator}, slice::ParallelSlice, *};
use bytemuck::{Pod, Zeroable};


pub trait Compressor {
    type Input;
    fn new() -> Self where Self: Sized;
    fn compress(&self, uncompressed: &[Self::Input], compressed: &mut Vec<u8>);
    fn decompress(&self, compressed: &[u8], uncompressed: &mut Vec<Self::Input>);
}
