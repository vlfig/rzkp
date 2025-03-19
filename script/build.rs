use sp1_build::build_program_with_args;

fn main() {
    build_program_with_args("../aggregator", Default::default());
    build_program_with_args("../fibonacci", Default::default());
}
