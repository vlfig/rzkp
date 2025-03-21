//! A simple proof aggregation to detect cycles

use anyhow::Result;
use clap::Parser;
use sp1_sdk::{
    include_elf, EnvProver, HashableKey, ProverClient, SP1Proof, SP1ProofWithPublicValues,
    SP1ProvingKey, SP1PublicValues, SP1Stdin, SP1VerifyingKey,
};

const CYCLER_ELF: &[u8] = include_elf!("cycler");

struct ReadyClient {
    client: EnvProver,
    sp1proving_key: SP1ProvingKey,
    sp1verifying_key: SP1VerifyingKey,
    vk_hash: [u32; 8],
}

impl ReadyClient {
    fn new(program: &[u8]) -> ReadyClient {
        let client = ProverClient::from_env();
        let (sp1proving_key, sp1verifying_key) = client.setup(program);
        let vk_hash = sp1verifying_key.hash_u32();

        ReadyClient {
            client,
            sp1proving_key,
            sp1verifying_key,
            vk_hash,
        }
    }

    fn prove<F>(&self, write_inputs: F) -> Result<SP1ProofWithPublicValues>
    where
        F: FnOnce(&mut SP1Stdin) -> (),
    {
        tracing::info_span!("proving program").in_scope(|| {
            let mut stdin = SP1Stdin::new();
            write_inputs(&mut stdin);
            self.client
                .prove(&self.sp1proving_key, &stdin)
                .compressed()
                .run()
        })
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,
}

fn main() {
    sp1_sdk::utils::setup_logger();

    // Parse the command line arguments.
    let args = Args::parse();

    if args.execute {
        // this execute-only branch doesn't really work well on recursive-proving programs
        // to provide it something non-trivial, we have to prove the previous ones at least
        let mut stdin = SP1Stdin::new();
        stdin.write(&1);
        stdin.write(&[0_u32; 8]);
        stdin.write(&SP1PublicValues::new());
        let (output, report) = ProverClient::from_env()
            .execute(CYCLER_ELF, &stdin)
            .run()
            .expect("execution failed");
        println!("Program executed successfully.");
        println!("output: {:?}", output.as_slice());
        println!("report: {:?}", report);
    } else {
        // simulate a cycle where p1 -> p2  -> p3  -> p1

        let c = ReadyClient::new(CYCLER_ELF);

        let p1 = c
            .prove(|stdin| {
                stdin.write(&1);
                stdin.write(&c.vk_hash);
                stdin.write(&SP1PublicValues::new());
            })
            .expect("proving of 1 failed");
        println!("{:?}", p1); // p1's commitment is the single path {1}

        let p2 = c
            .prove(|stdin| {
                stdin.write(&2);
                stdin.write(&c.vk_hash);
                stdin.write(&p1.clone().public_values.to_vec());
                let SP1Proof::Compressed(proof) = p1.proof else {
                    panic!()
                };
                stdin.write_proof(*proof, c.sp1verifying_key.vk.clone());
            })
            .expect("proving of 2 failed");
        println!("{:?}", p2); // p2's commitment is the pathset {1->2,2->1} (assume no order)

        let p3 = c
            .prove(|stdin| {
                stdin.write(&3);
                stdin.write(&c.vk_hash);
                stdin.write(&p2.public_values.to_vec());
                let SP1Proof::Compressed(proof) = p2.proof else {
                    panic!()
                };
                stdin.write_proof(*proof, c.sp1verifying_key.vk.clone());
            })
            .expect("proving of 3 failed");
        println!("{:?}", p3); // p3's commitment is the pathset {1->2->3,2->1-3,…}

        // So at this point there is proof that p3 has verified that it has p1 and p2 reaching it.
        // This means that a path exists that is one of the permutations of {1,2,3} (no order should be assumed,
        // even though we're just appending for now).
        // p3 in particular knows that it is last, however.
        // if the originator of the proof p3 verified is known, p3 also knows which of the others is
        // the second-to-last.
        // Also, because it knows its neighbours and 1 is one of them, it is able to determine
        // locally that a cycle exists, but to prove it to a third party we need (I guess?) either:
        //   a) p1 to produce a commitment which contains its id twice – thus for p3 to send its
        //      proof on to 1; or
        //   b) p3 to sign its inputs and own state – should be enough since the closing link is its
        //      own sovereign data anyway.
        // Option a) feels conceptually neater though.

        let cycle = c
            .prove(|stdin| {
                stdin.write(&1);
                stdin.write(&c.vk_hash);
                stdin.write(&p3.public_values.to_vec());
                let SP1Proof::Compressed(proof) = p3.proof else {
                    panic!()
                };
                stdin.write_proof(*proof, c.sp1verifying_key.vk.clone());
            })
            .expect("proving of closure failed");
        println!("{:?}", cycle); //

        verify_has_cycle(c, cycle);
    }
}

fn verify_has_cycle(c: ReadyClient, proof: SP1ProofWithPublicValues) {
    if c.client.verify(&proof, &c.sp1verifying_key).is_ok() {
        let mut pvs = proof.public_values.to_vec();
        pvs.sort_unstable();
        let mut deduped = pvs.clone();
        deduped.dedup();
        if pvs != deduped {
            println!("There is indeed a cycle.")
        } else {
            println!("Proof is valid but no cycle.")
        }
    } else {
        println!("Invalid proof.")
    }
}
