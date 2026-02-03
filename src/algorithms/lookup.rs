use std::{collections::HashMap, hash::Hash, io::{BufReader, Read, Write}, marker::PhantomData, ops::Sub};
use rayon::{iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator}, slice::ParallelSlice, *};
use bytemuck::{Pod, Zeroable};
use crate::compressor::*;

pub struct Lookup<T: Pod + Eq + Hash + Send + Sync> {
    _phantom: PhantomData<T>,
    //buffer_size: usize,
}

impl<T: Pod + Eq + Hash + Send + Sync> Compressor for Lookup<T> {
    type Input = T;
    fn compress(&self, uncompressed: &[T], compressed: &mut Vec<u8>) {
        // check occurences of window of elements with varying slice sizes
        // pick the window size / phase combination that yields the highest occuring members
        let best_of_the_best_across_window_sizes_and_phases = (1..64).into_par_iter().map(|window_size| {
            // check every phase and check for highest occurences in TOTAL
            // the "best phase" will be the one with the highest number of occurences
            (0..window_size).into_par_iter().map(|phase| {
                let mut occurences = HashMap::<&[T], u32>::new();

                // go through all chunks
                // make sure to start at phase offset!
                for (index, window) in uncompressed[phase..].chunks(window_size).enumerate() {
                    *occurences.entry(window).or_default() += 1;
                }

                let total_occurences_of_everything = *occurences.values().into_iter().max().unwrap();
                (window_size, phase, total_occurences_of_everything, occurences)
            }).max_by_key(|(window_size, phase, x, p)| *x).unwrap()
        }).max_by_key(|(window_size, phase, x, p)| *x).unwrap();

        // we have 
        // 1. the best window size to use
        // 2. the best phase for the window size
        // 3. the used lookup hashmap
        // we just need to encode the uncompressed data using these parameters
        let (window_size, phase, max_occurences, hash_map) = best_of_the_best_across_window_sizes_and_phases;
        dbg!(window_size);
        dbg!(phase);
        dbg!(max_occurences);
        
        compressed.extend_from_slice(&usize::to_ne_bytes(window_size));
        compressed.extend_from_slice(&usize::to_ne_bytes(phase));

        // write the window slices at the very start...       
        compressed.extend_from_slice(&usize::to_ne_bytes(hash_map.len()));

        // maps window slices to their indices
        let mut new_hash_map = HashMap::<&[T], usize>::new();
        for (entry, value) in hash_map {
            compressed.extend_from_slice(bytemuck::cast_slice(entry));    
            new_hash_map.insert(entry, new_hash_map.len()); 
        }

        // write mode (we can use this to infer how many bytes we will write for each reference to the hashmap)
        let mode = crate::algorithms::common::get_mode(new_hash_map.len() as u64);
        compressed.extend_from_slice(&usize::to_ne_bytes(mode));

        // if phase is not zero, then we need to add the elements that we skipped over at the start... (the one skipped by phase)
        if phase != 0 {
            let pre_phase_slice = bytemuck::cast_slice(&uncompressed[0..phase]);
            compressed.extend_from_slice(pre_phase_slice);
        }

        // go through uncompressed data in chunks, and check from hashmap. we WILL reach a unique value. fosho
        let should_be_aligned = &uncompressed[phase..];
        for slice in should_be_aligned.chunks(window_size) {
            let index = new_hash_map[slice];
            crate::algorithms::common::write_count_bytes_with_mode(index as u64, mode, compressed);
        }
    }

    fn decompress(&self, compressed: &[u8], uncompressed: &mut Vec<T>) {
        
    }

    fn new() -> Self {
        Self {
            _phantom: Default::default(),
            //buffer_size: 256,
        }
    }
}

