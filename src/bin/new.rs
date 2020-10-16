use masp_mpc::bridge::BridgeCircuit;
use phase2::parameters::MPCParameters;
use std::fs::File;
use std::marker::PhantomData;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: \n<out_params.params> <path/to/phase1radix>");
        std::process::exit(exitcode::USAGE);
    }
    let params_filename = &args[1];
    let radix_directory = &args[2];

    let should_filter_points_at_infinity = false;

    println!("Creating initial parameters for Spend...");

    let mut f = File::create(params_filename).unwrap();

    // MASP spend circuit
    let params = MPCParameters::new(
        BridgeCircuit {
            circuit: masp_proofs::circuit::sapling::Spend {
                value_commitment: None,
                proof_generation_key: None,
                payment_address: None,
                commitment_randomness: None,
                ar: None,
                auth_path: vec![None; 32], // Tree depth is 32 for sapling
                anchor: None,
            },
            _scalar: PhantomData::<bls12_381::Scalar>,
        },
        should_filter_points_at_infinity,
        radix_directory,
    )
    .unwrap();
    println!("Writing initial Spend parameters to {}.", params_filename);

    params.write(&mut f).expect("unable to write Spend params");

    println!("Creating initial parameters for Output...");

    // MASP output circuit
    let params = MPCParameters::new(
        BridgeCircuit {
            circuit: masp_proofs::circuit::sapling::Output {
                value_commitment: None,
                payment_address: None,
                commitment_randomness: None,
                esk: None,
                asset_identifier: vec![None; 256],
            },
            _scalar: PhantomData::<bls12_381::Scalar>,
        },
        should_filter_points_at_infinity,
        radix_directory,
    )
    .unwrap();

    println!("Writing initial Output parameters to {}.", params_filename);

    params.write(&mut f).expect("unable to write Output params");
}

#[test]
fn test_hash() {
    use bellman_ce::pairing::bls12_381::Bls12;
    use bellman_ce::Circuit;
    {
        let mut cs = masp_mpc::test::TestConstraintSystem::<Bls12>::new();

        BridgeCircuit {
            circuit: masp_proofs::circuit::sapling::Spend {
                value_commitment: None,
                proof_generation_key: None,
                payment_address: None,
                commitment_randomness: None,
                ar: None,
                auth_path: vec![None; 32], // Tree depth is 32 for sapling
                anchor: None,
            },
            _scalar: PhantomData::<bls12_381::Scalar>,
        }
        .synthesize(&mut cs)
        .unwrap();

        assert_eq!(cs.num_constraints(), 100637);
        assert_eq!(
            cs.hash(),
            "34e4a634c80e4e4c6250e63b7855532e60b36d1371d4d7b1163218b69f09eb3d"
        );
    }
    {
        let mut cs = masp_mpc::test::TestConstraintSystem::<Bls12>::new();

        BridgeCircuit {
            circuit: masp_proofs::circuit::sapling::Output {
                value_commitment: None,
                payment_address: None,
                commitment_randomness: None,
                esk: None,
                asset_identifier: vec![None; 256],
            },
            _scalar: PhantomData::<bls12_381::Scalar>,
        }
        .synthesize(&mut cs)
        .unwrap();

        assert_eq!(cs.num_constraints(), 31205);
        assert_eq!(
            cs.hash(),
            "93e445d7858e98c7138558df341f020aedfe75893535025587d64731e244276a"
        );
    }
}
