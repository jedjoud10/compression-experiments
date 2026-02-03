use std::{io::{BufReader, Read}, marker::PhantomData};
use rayon::{iter::{IntoParallelIterator, IntoParallelRefIterator, ParallelExtend, ParallelIterator}, slice::ParallelSlice, *};
use bytemuck::{Pod, Zeroable};


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
