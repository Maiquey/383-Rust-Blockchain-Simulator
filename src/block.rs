use crate::queue::{Task, WorkQueue};
use digest::consts::U32;
use sha2::digest::generic_array::GenericArray;
use sha2::{Digest, Sha256};
use std::fmt::Write;
use std::sync;

type Hash = GenericArray<u8, U32>;

#[derive(Debug, Clone)]
pub struct Block {
    prev_hash: Hash,
    generation: u64,
    difficulty: u8,
    data: String,
    proof: Option<u64>,
}

impl Block {
    pub fn initial(difficulty: u8) -> Block {
        // create and return a new initial block
        Block {
            prev_hash: Hash::default(),
            generation: 0,
            difficulty: difficulty,
            data: "".to_string(),
            proof: None
        }
    }

    pub fn next(previous: &Block, data: String) -> Block {
        // create and return a block that could follow `previous` in the chain
        Block {
            prev_hash: previous.hash(),
            generation: previous.generation + 1,
            difficulty: previous.difficulty,
            data: data,
            proof: None
        }
    }

    pub fn hash_string_for_proof(&self, proof: u64) -> String {
        // return the hash string this block would have if we set the proof to `proof`.
        let mut prev_hash_string = String::new();
        write!(&mut prev_hash_string, "{:02x}", self.prev_hash).unwrap();
        format!("{}:{}:{}:{}:{}", prev_hash_string, self.generation, self.difficulty, self.data, proof)
    }

    pub fn hash_string(&self) -> String {
        // self.proof.unwrap() panics if block not mined
        let p = self.proof.unwrap();
        self.hash_string_for_proof(p)
    }

    pub fn hash_for_proof(&self, proof: u64) -> Hash {
        // return the block's hash as it would be if we set the proof to `proof`.
        let mut hasher = Sha256::new();
        let hash_string = self.hash_string_for_proof(proof);
        hasher.update(hash_string);
        let result = hasher.finalize();
        return result;
    }

    pub fn hash(&self) -> Hash {
        // self.proof.unwrap() panics if block not mined
        let p = self.proof.unwrap();
        self.hash_for_proof(p)
    }

    pub fn set_proof(self: &mut Block, proof: u64) {
        self.proof = Some(proof);
    }

    pub fn is_valid_for_proof(&self, proof: u64) -> bool {
        // would this block be valid if we set the proof to `proof`?
        let hash: Hash = self.hash_for_proof(proof);
        let n_bytes: usize = (self.difficulty/8).into();
        let n_bits: usize = (self.difficulty%8).into();

        let last_byte_index = hash.len() - 1;
        if n_bytes > 0 {
            for i in 0..n_bytes {
                if hash[last_byte_index - i] != 0u8{
                    return false;
                }
            }
        }

        let next_byte_from_end = hash.len() - 1 - n_bytes;
        if hash[next_byte_from_end] as usize % (1<<n_bits) != 0 {
            return false;
        }
        return true;
    }

    pub fn is_valid(&self) -> bool {
        if self.proof.is_none() {
            return false;
        }
        self.is_valid_for_proof(self.proof.unwrap())
    }

    // Mine in a very simple way: check sequentially until a valid hash is found.
    // This doesn't *need* to be used in any way, but could be used to do some mining
    // before your .mine is complete. Results should be the same as .mine (but slower).
    pub fn mine_serial(self: &mut Block) {
        let mut p = 0u64;
        while !self.is_valid_for_proof(p) {
            p += 1;
        }
        self.proof = Some(p);
    }
    
    //public function for testing correctness of MiningTask
    pub fn mine_serial_using_task(self: &mut Block){
        let shared_block = sync::Arc::new(self.clone());
        let mining_task = MiningTask::new(shared_block.clone(), 0, 8 * (1<<self.difficulty));
        match mining_task.run() {
            Some(p) => {
                self.set_proof(p);
            }
            None => {
                //No proof found
            }
        }
    }

    pub fn mine_serial_parallel(self: &Block, start: u64, end: u64)-> Option<u64>{
        let mut p = start;
        while p <= end {
            if self.is_valid_for_proof(p){
                return Some(p);
            }
            p += 1;
        }
        return None;
    }

    //deprecated test function using serial mining to ensure mine_range logic is correct before further implementation
    pub fn mine_range_serial(self: &Block, _workers: usize, start: u64, end: u64, chunks: u64) -> u64 {
        // With `workers` threads, check proof values in the given range, breaking up
	    // into `chunks` tasks in a work queue. Return the first valid proof found.
        // HINTS:
        // - Create and use a queue::WorkQueue.
        // - Use sync::Arc to wrap a clone of self for sharing.
        let num_values_to_check = end - start + 1;
        let mut chunk_length = num_values_to_check / chunks;
        if num_values_to_check % chunks != 0 {
            chunk_length += 1;
        }

        for i in 0..chunks {
            //simulate spawning a thread
            let parallel_start = chunk_length*i;
            let parallel_end = parallel_start + chunk_length - 1;
            match self.mine_serial_parallel(parallel_start, parallel_end){
                Some(p) => {
                    return p;
                }
                None => {
                    //Do nothing
                }
            }
        }
        return 0;
    }

    pub fn mine_range(self: &Block, workers: usize, start: u64, end: u64, chunks: u64) -> u64 {
        // With `workers` threads, check proof values in the given range, breaking up
        // into `chunks` tasks in a work queue. Return the first valid proof found.
        // HINTS:
        // - Create and use a queue::WorkQueue.
        // - Use sync::Arc to wrap a clone of self for sharing.
        // use MiningTask here
        // check proof values for this block from start to end (inclusive). 
        // The calculation should be done in parallel by the given number of workers and dividing the work into chunks approximately equal parts.
        // Use the work queue. Should be fairly easy to do the work in parallel, and to stop checking proof values after a valid proof is found.
        let mut q = WorkQueue::<MiningTask>::new(workers);
        let shared_block = sync::Arc::new(self.clone());

        let num_values_to_check = end - start + 1;
        let mut chunk_length = num_values_to_check / chunks;
        //last chunk may be shorter than the rest
        if num_values_to_check % chunks != 0 {
            chunk_length += 1;
        }

        for i in 0..chunks {
            let parallel_start = chunk_length*i;
            let mut parallel_end = parallel_start + chunk_length - 1;
            if parallel_end > end {
                parallel_end = end;
            }
            let _ = q.enqueue(
                MiningTask::new (
                    shared_block.clone(),
                    parallel_start,
                    parallel_end,
                )
            );

        }

        let result = q.recv();

        q.shutdown();

        return result;
    }

    pub fn mine_for_proof(self: &Block, workers: usize) -> u64 {
        let range_start: u64 = 0;
        let range_end: u64 = 8 * (1 << self.difficulty); // 8 * 2^(bits that must be zero)
        let chunks: u64 = 2345;
        self.mine_range(workers, range_start, range_end, chunks)
    }

    pub fn mine(self: &mut Block, workers: usize) {
        self.proof = Some(self.mine_for_proof(workers));
    }
}

struct MiningTask {
    block: sync::Arc<Block>,
    start: u64,
    end: u64,
}

impl MiningTask {
    pub fn new(block: sync::Arc<Block>, start: u64, end: u64) -> MiningTask {
        MiningTask {
            block: block,
            start: start,
            end: end
        }
    }
}

impl Task for MiningTask {
    type Output = u64;

    fn run(&self) -> Option<u64> {
        //must return an Option<Output> value. None means no valid proof found, Some(p) means p is valid proof
        let mut p = self.start;
        while p <= self.end {
            if self.block.is_valid_for_proof(p){
                return Some(p);
            }
            p += 1;
        }
        return None;
    }
}
