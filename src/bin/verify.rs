use blake2::{Blake2b512, Digest};
use masp_phase2::MPCParameters;
use std::fs::File;
use std::io::BufReader;

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

    // Used for Namada Trusted Setup contribution files where the first 64 bytes contain the hash of the previous contribution file
    // Offset 64 bytes to access the parameters 
    current_params.seek_relative(64).unwrap();

    let masp_spend = MPCParameters::read(&mut current_params, true)
        .expect("couldn't deserialize MASP Spend params");

    let masp_output = MPCParameters::read(&mut current_params, true)
        .expect("couldn't deserialize MASP Output params");

    let masp_convert = MPCParameters::read(&mut current_params, true)
        .expect("couldn't deserialize MASP Convert params");

    let masp_spend_contributions = masp_spend
        .verify(
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
        .expect("MASP Spend parameters are invalid");

    let masp_output_contributions = masp_output
        .verify(
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
        .expect("MASP Output parameters are invalid");

    let masp_convert_contributions = masp_convert
        .verify(
            masp_proofs::circuit::convert::Convert {
                value_commitment: None,
                auth_path: vec![None; 32], // Tree depth is 32 for sapling
                anchor: None,
            },
            //should_filter_points_at_infinity,
            //radix_directory,
        )
        .expect("MASP Convert parameters are invalid");

    for (spend_hash, output_hash, convert_hash) in itertools::multizip((
        masp_spend_contributions.into_iter(),
        masp_output_contributions.into_iter(),
        masp_convert_contributions.into_iter(),
    )) {
        let mut h = Blake2b512::new();
        h.update(&spend_hash);
        h.update(&output_hash);
        h.update(&convert_hash);
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
