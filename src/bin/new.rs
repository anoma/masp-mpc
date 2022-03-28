use masp_phase2::MPCParameters;
use std::fs::File;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: \n<out_params.params> <path/to/phase1radix>");
        std::process::exit(exitcode::USAGE);
    }
    let params_filename = &args[1];
    //let radix_directory = &args[2];

    //let should_filter_points_at_infinity = false;

    println!("Creating initial parameters for MASP Spend...");

    let mut f = File::create(params_filename).unwrap();

    // MASP spend circuit
    let spend_params = MPCParameters::new(
        masp_proofs::circuit::sapling::Spend {
            value_commitment: None,
            proof_generation_key: None,
            payment_address: None,
            commitment_randomness: None,
            ar: None,
            auth_path: vec![None; 32], // Tree depth is 32 for sapling
            anchor: None,
        },
        //should_filter_points_at_infinity,
        //radix_directory,
    )
    .unwrap();
    println!(
        "Writing initial MASP Spend parameters to {}.",
        params_filename
    );

    spend_params
        .write(&mut f)
        .expect("unable to write MASP Spend params");

    println!("Creating initial parameters for MASP Output...");

    // MASP output circuit
    let output_params = MPCParameters::new(
        masp_proofs::circuit::sapling::Output {
            value_commitment: None,
            payment_address: None,
            commitment_randomness: None,
            esk: None,
            asset_identifier: vec![None; 256],
        },
        //should_filter_points_at_infinity,
        //radix_directory,
    )
    .unwrap();

    println!(
        "Writing initial MASP Output parameters to {}.",
        params_filename
    );

    output_params
        .write(&mut f)
        .expect("unable to write MASP Output params");

    // MASP Convert circuit
    let convert_params = MPCParameters::new(
        masp_proofs::circuit::convert::Convert {
            value_commitment: None,
            auth_path: vec![None; 32], // Tree depth is 32 for sapling
            anchor: None,
        },
        //should_filter_points_at_infinity,
        //radix_directory,
    )
    .unwrap();
    println!(
        "Writing initial MASP Convert parameters to {}.",
        params_filename
    );

    convert_params
        .write(&mut f)
        .expect("unable to write MASP Convert params");
}

#[test]
fn test_hash() {
    use bellman::Circuit;
    use bls12_381::Bls12;
    {
        let mut cs = bellman::gadgets::test::TestConstraintSystem::<Bls12>::new();

        masp_proofs::circuit::sapling::Spend {
            value_commitment: None,
            proof_generation_key: None,
            payment_address: None,
            commitment_randomness: None,
            ar: None,
            auth_path: vec![None; 32], // Tree depth is 32 for sapling
            anchor: None,
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
        let mut cs = bellman::gadgets::test::TestConstraintSystem::<Bls12>::new();

        masp_proofs::circuit::sapling::Output {
            value_commitment: None,
            payment_address: None,
            commitment_randomness: None,
            esk: None,
            asset_identifier: vec![None; 256],
        }
        .synthesize(&mut cs)
        .unwrap();

        assert_eq!(cs.num_constraints(), 31205);
        assert_eq!(
            cs.hash(),
            "93e445d7858e98c7138558df341f020aedfe75893535025587d64731e244276a"
        );
    }
    {
        let mut cs = bellman::gadgets::test::TestConstraintSystem::<Bls12>::new();

        masp_proofs::circuit::convert::Convert {
            value_commitment: None,
            auth_path: vec![None; 32], // Tree depth is 32 for sapling
            anchor: None,
        }
        .synthesize(&mut cs)
        .unwrap();

        assert_eq!(cs.num_constraints(), 47358);
        assert_eq!(
            cs.hash(),
            "f74b47ef6e59081548f81f5806bd15b1f4a65d2e57681e6db2b8db7eef2ff814"
        );
    }
}
