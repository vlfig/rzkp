#![no_main]
sp1_zkvm::entrypoint!(main);

use sha2::{Digest, Sha256};
use sp1_zkvm::io::{commit, read};
use sp1_zkvm::lib::verify::verify_sp1_proof;

pub fn main() {
    // read our own id
    let id = read::<u8>();

    // Read the verification key hash
    let vkey = read::<[u32; 8]>();

    // Read the public values of the given proof (the ids that reach it)
    let mut public_values: Vec<u8> = read::<Vec<u8>>();
    // the memory layout of Vec seems wasteful, compacting it may reduce circuit count?

    // verify the proof
    println!("cycle-tracker-report-start: verification");
    if !public_values.is_empty() {
        verify_sp1_proof(&vkey, &Sha256::digest(&public_values).into());
    }
    println!("cycle-tracker-report-end: verification");

    // verification successful: mutate public values, becomes commitment
    println!("cycle-tracker-report-start: push own id");
    public_values.push(id);
    println!("cycle-tracker-report-end: push own id");

    // commit to previous commitment plus our own id
    commit(&public_values);
}
