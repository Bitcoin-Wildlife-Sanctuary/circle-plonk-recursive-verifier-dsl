use circle_plonk_dsl_answer::AnswerResults;
use circle_plonk_dsl_circle::CirclePointQM31Var;
use circle_plonk_dsl_composition::CompositionCheck;
use circle_plonk_dsl_constraint_system::dvar::AllocVar;
use circle_plonk_dsl_constraint_system::ConstraintSystemRef;
use circle_plonk_dsl_data_structures::PlonkWithPoseidonProofVar;
use circle_plonk_dsl_fiat_shamir::FiatShamirResults;
use circle_plonk_dsl_fields::QM31Var;
use circle_plonk_dsl_folding::FoldingResults;
use circle_plonk_dsl_hints::{
    AnswerHints, DecommitHints, FiatShamirHints, FirstLayerHints, InnerLayersHints,
};
use num_traits::One;
use std::io::Write;
use std::path::Path;
use stwo_prover::core::fields::qm31::QM31;
use stwo_prover::core::fri::FriConfig;
use stwo_prover::core::pcs::PcsConfig;
use stwo_prover::core::vcs::poseidon31_merkle::{Poseidon31MerkleChannel, Poseidon31MerkleHasher};
use stwo_prover::examples::plonk_with_poseidon::air::{
    prove_plonk_with_poseidon, verify_plonk_with_poseidon, PlonkWithPoseidonProof,
};

pub fn demo_recurse(
    src: &Path,
    src_config: PcsConfig,
    multipliers: usize,
    dest: &Path,
    dest_config: PcsConfig,
) {
    let mut fs = std::fs::File::open(src).unwrap();

    let proof: PlonkWithPoseidonProof<Poseidon31MerkleHasher> =
        bincode::deserialize_from(&mut fs).unwrap();

    let fiat_shamir_hints = FiatShamirHints::new(
        &proof,
        src_config,
        &[
            (1, QM31::one()),
            (2, QM31::from_u32_unchecked(0, 1, 0, 0)),
            (3, QM31::from_u32_unchecked(0, 0, 1, 0)),
        ],
    );
    let answer_hints = AnswerHints::compute(&fiat_shamir_hints, &proof);
    let decommitment_hints = DecommitHints::compute(&fiat_shamir_hints, &proof);
    let first_layer_hints = FirstLayerHints::compute(&fiat_shamir_hints, &answer_hints, &proof);
    let inner_layer_hints = InnerLayersHints::compute(
        &first_layer_hints.folded_evals_by_column,
        &fiat_shamir_hints,
        &proof,
    );

    let cs = ConstraintSystemRef::new_ref();

    for _ in 0..multipliers {
        let mut proof_var = PlonkWithPoseidonProofVar::new_witness(&cs, &proof);

        let fiat_shamir_results = FiatShamirResults::compute(
            &fiat_shamir_hints,
            &mut proof_var,
            src_config,
            &[
                (1, QM31Var::one(&cs)),
                (2, QM31Var::i(&cs)),
                (3, QM31Var::j(&cs)),
            ],
        );
        CompositionCheck::compute(
            &fiat_shamir_hints,
            &fiat_shamir_results.lookup_elements,
            fiat_shamir_results.random_coeff.clone(),
            fiat_shamir_results.oods_point.clone(),
            &proof_var,
        );

        let answer_results = AnswerResults::compute(
            &CirclePointQM31Var::new_witness(&cs, &fiat_shamir_hints.oods_point),
            &fiat_shamir_hints,
            &fiat_shamir_results,
            &answer_hints,
            &decommitment_hints,
            &proof_var,
            src_config,
        );

        FoldingResults::compute(
            &proof_var,
            &fiat_shamir_hints,
            &fiat_shamir_results,
            &answer_results,
            &first_layer_hints,
            &inner_layer_hints,
        );
    }

    cs.pad();
    cs.check_arithmetics();
    cs.populate_logup_arguments();
    cs.check_poseidon_invocations();

    let (plonk, mut poseidon) = cs.generate_circuit();

    if std::fs::exists(dest).unwrap() {
        return;
    }

    let timer = std::time::Instant::now();
    let proof =
        prove_plonk_with_poseidon::<Poseidon31MerkleChannel>(dest_config, &plonk, &mut poseidon);
    println!("proof generation time: {}s", timer.elapsed().as_secs_f64());

    let encoded = bincode::serialize(&proof).unwrap();
    let mut fs = std::fs::File::create(dest).unwrap();
    fs.write(&encoded).unwrap();

    verify_plonk_with_poseidon::<Poseidon31MerkleChannel>(
        proof,
        dest_config,
        &[
            (1, QM31::one()),
            (2, QM31::from_u32_unchecked(0, 1, 0, 0)),
            (3, QM31::from_u32_unchecked(0, 0, 1, 0)),
        ],
    )
    .unwrap();
}

fn main() {
    let standard_config = PcsConfig {
        pow_bits: 20,
        fri_config: FriConfig::new(0, 5, 16),
    };
    let fast_prover_config = PcsConfig {
        pow_bits: 20,
        fri_config: FriConfig::new(0, 1, 80),
    };
    let fast_prover2_config = PcsConfig {
        pow_bits: 20,
        fri_config: FriConfig::new(0, 3, 27),
    };
    let fast_verifier_config = PcsConfig {
        pow_bits: 20,
        fri_config: FriConfig::new(0, 8, 10),
    };
    let fast_verifier2_config = PcsConfig {
        pow_bits: 20,
        fri_config: FriConfig::new(0, 9, 9),
    };
    let fast_verifier3_config = PcsConfig {
        pow_bits: 20,
        fri_config: FriConfig::new(0, 10, 8),
    };

    demo_recurse(
        Path::new("../../components/test_data/recursive_proof_17_18.bin"),
        standard_config,
        5,
        Path::new("data/level1-5.bin"),
        fast_prover_config,
    );
    demo_recurse(
        Path::new("data/level1-5.bin"),
        fast_prover_config,
        1,
        Path::new("data/level2-1.bin"),
        fast_prover2_config,
    );
    demo_recurse(
        Path::new("data/level2-1.bin"),
        fast_prover2_config,
        1,
        Path::new("data/level3-1.bin"),
        standard_config,
    );
    demo_recurse(
        Path::new("data/level3-1.bin"),
        standard_config,
        5,
        Path::new("data/level4-5.bin"),
        fast_prover_config,
    );
    demo_recurse(
        Path::new("data/level4-5.bin"),
        fast_prover_config,
        1,
        Path::new("data/level5-1.bin"),
        fast_prover2_config,
    );
    demo_recurse(
        Path::new("data/level5-1.bin"),
        fast_prover2_config,
        1,
        Path::new("data/level6-1.bin"),
        standard_config,
    );
    demo_recurse(
        Path::new("data/level6-1.bin"),
        standard_config,
        1,
        Path::new("data/level7-1.bin"),
        standard_config,
    );
    demo_recurse(
        Path::new("data/level7-1.bin"),
        standard_config,
        1,
        Path::new("data/level8-1.bin"),
        fast_verifier_config,
    );
    demo_recurse(
        Path::new("data/level8-1.bin"),
        fast_verifier_config,
        1,
        Path::new("data/level9-1.bin"),
        fast_verifier_config,
    );
    demo_recurse(
        Path::new("data/level9-1.bin"),
        fast_verifier_config,
        1,
        Path::new("data/level10-1.bin"),
        fast_verifier2_config,
    );
    demo_recurse(
        Path::new("data/level10-1.bin"),
        fast_verifier2_config,
        1,
        Path::new("data/level11-1.bin"),
        fast_verifier2_config,
    );
    demo_recurse(
        Path::new("data/level11-1.bin"),
        fast_verifier2_config,
        1,
        Path::new("data/level12-1.bin"),
        fast_verifier3_config,
    );
    demo_recurse(
        Path::new("data/level12-1.bin"),
        fast_verifier3_config,
        1,
        Path::new("data/level13-1.bin"),
        fast_verifier3_config,
    );
    demo_recurse(
        Path::new("data/level13-1.bin"),
        fast_verifier3_config,
        1,
        Path::new("data/level14-1.bin"),
        fast_prover_config,
    );
}
