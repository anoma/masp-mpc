use blake2::{Blake2b512, Digest};
use masp_phase2::MPCParameters;
use std::fs::File;
use std::io::{BufReader, Read};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: \ncontribution_to_check final_contribution");
        std::process::exit(exitcode::USAGE);
    }
    let contribution_to_check = &args[1];
    let final_contribution = &args[2];

    let current_params = File::open(contribution_to_check).expect("couldn't open params");
    let mut current_params = BufReader::with_capacity(1024 * 1024, current_params);
    current_params.seek_relative(64).unwrap();
    let contribution_hash = hash_reader(64, current_params);

    let ctc = extract_internal_hashes(contribution_to_check, false);
    let target_internal_hash = ctc
        .last()
        .expect("At least one contribution needed in contribution_to_check");
    let final_internal_hashes = extract_internal_hashes(final_contribution, false);

    for (i, internal_hash) in final_internal_hashes.iter().enumerate() {
        if internal_hash == target_internal_hash {
            println!(
                "Contribution with contribution hash {} found at round {}",
                contribution_hash,
                i + 1
            );
            std::process::exit(0);
        }
    }
    println!(
        "Contribution with contribution hash {} not found",
        contribution_hash
    );
}

fn into_hex(h: &[u8]) -> String {
    let mut f = String::new();

    for byte in &h[..] {
        f += &format!("{:02x}", byte);
    }

    f
}

fn extract_internal_hashes(params_filename: &str, checked: bool) -> Vec<[u8; 64]> {
    let current_params = File::open(params_filename).expect("couldn't open params");
    let mut current_params = BufReader::with_capacity(1024 * 1024, current_params);
    current_params.seek_relative(64).unwrap();

    let masp_spend = MPCParameters::read(&mut current_params, checked)
        .expect("couldn't deserialize MASP Spend params");

    let masp_output = MPCParameters::read(&mut current_params, checked)
        .expect("couldn't deserialize MASP Output params");

    let masp_convert = MPCParameters::read(&mut current_params, checked)
        .expect("couldn't deserialize MASP Convert params");

    let verify_params = checked;
    let masp_spend_contributions = if !verify_params {
        extract_contributions(&masp_spend)
    } else {
        masp_spend
            .verify(masp_proofs::circuit::sapling::Spend {
                value_commitment: None,
                proof_generation_key: None,
                payment_address: None,
                commitment_randomness: None,
                ar: None,
                auth_path: vec![None; 32], // Tree depth is 32 for sapling
                anchor: None,
            })
            .expect("MASP Spend parameters are invalid")
    };

    let masp_output_contributions = if !verify_params {
        extract_contributions(&masp_output)
    } else {
        masp_output
            .verify(masp_proofs::circuit::sapling::Output {
                value_commitment: None,
                payment_address: None,
                commitment_randomness: None,
                esk: None,
                asset_identifier: vec![None; 256],
            })
            .expect("MASP Output parameters are invalid")
    };

    let masp_convert_contributions = if !verify_params {
        extract_contributions(&masp_convert)
    } else {
        masp_convert
            .verify(masp_proofs::circuit::convert::Convert {
                value_commitment: None,
                auth_path: vec![None; 32], // Tree depth is 32 for sapling
                anchor: None,
            })
            .expect("MASP Convert parameters are invalid")
    };
    let mut internal_hashes = vec![];
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
        internal_hashes.push(h.into());
    }
    internal_hashes
}

fn extract_contributions(params: &MPCParameters) -> Vec<[u8; 64]> {
    params
        .contributions
        .iter()
        .map(|pubkey| {
            let sink = std::io::sink();
            let mut sink = masp_phase2::HashWriter::new(sink);
            pubkey.write(&mut sink).unwrap();
            let h = sink.into_hash();
            let mut response = [0u8; 64];
            response.copy_from_slice(h.as_ref());
            response
        })
        .collect()
}

/// Below code from b2sum crate, MIT License Copyright (c) 2017 John Downey
fn hash_reader<R>(length: usize, mut reader: R) -> String
where
    R: std::io::BufRead,
{
    let mut digest = blake2b_simd::Params::new().hash_length(length).to_state();

    loop {
        let count = {
            let data = reader.fill_buf().unwrap();
            if data.is_empty() {
                break;
            }

            digest.update(data);
            data.len()
        };

        reader.consume(count);
    }

    let output = digest.finalize();
    let result = output.to_hex().to_ascii_lowercase();

    result
}
