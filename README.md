# ZK Cycle detection

This is a simple prototype of distributed zero-knowledge cycle detection via recursive proof aggregation
on [SP1](https://docs.succinct.xyz/getting-started/install.html).

Distributed: no central entity knows the entire graph.
Zero-knowledge: each node learns the nodes that exist on paths to it only (and whether there's a cycle).

Inspired by [@nibnalin](https://x.com/nibnalin)'s post [Recursive zkSNARKs: Exploring New Territory](https://0xparc.org/blog/groth16-recursion).

## Running the Project

To build the program, run the following command:

```sh
cargo build --release
```

To run the program as the first node in the loop without generating a proof:

```sh
cargo run --release -- --execute
```

To run the program for each node in the loop, generating the execution proof for the next to verify:

```sh
cargo run --release
```

If you're running on the CPU and it is recent:

```zsh
RUSTFLAGS='-C target-cpu=native -C target-feature=+avx512f' && RUST_LOG=info && time cargo run --release
```

or

```bash
RUSTFLAGS='-C target-cpu=native -C target-feature=+avx512f' RUST_LOG=info time cargo run --release
```

This will execute the program and display the output, warming up your motherboard in the process. :-)
