use blake2::{Blake2b, Digest};
use masp_mpc::bridge::BridgeCircuit;
use phase2::parameters::MPCParameters;
use std::fs::File;
use std::io::BufReader;
use std::marker::PhantomData;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: \n<params.params> <path/to/phase1radix>");
        std::process::exit(exitcode::USAGE);
    }
    let params_filename = &args[1];
    let radix_directory = &args[2];

    let should_filter_points_at_infinity = false;

    let current_params = File::open(params_filename).expect("couldn't open params");
    let mut current_params = BufReader::with_capacity(1024 * 1024, current_params);

    let masp_spend =
        MPCParameters::read(&mut current_params, should_filter_points_at_infinity, true)
            .expect("couldn't deserialize MASP Spend params");

    let masp_output =
        MPCParameters::read(&mut current_params, should_filter_points_at_infinity, true)
            .expect("couldn't deserialize MASP Output params");

    let masp_spend_contributions = masp_spend
        .verify(
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
        .expect("Spend parameters are invalid");

    let masp_output_contributions = masp_output
        .verify(
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
        .expect("Output parameters are invalid");

    for (a, b) in masp_spend_contributions
        .into_iter()
        .zip(masp_output_contributions.into_iter())
    {
        let mut h = Blake2b::new();
        h.update(&a);
        h.update(&b);
        let h = h.finalize();

        println!("{}", into_hex(h.as_ref()));
    }
}

fn into_hex(h: &[u8]) -> String {
    let mut f = String::new();

    for byte in &h[..] {
        f += &format!("{:02x}", byte);
    }

    f
}
