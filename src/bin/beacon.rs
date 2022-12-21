use blake2::Blake2b512;
use itertools::Itertools;
use masp_phase2::MPCParameters;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::Write;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 7 {
        println!("Usage: \n<in_params.params> <in_btc_hash> <in_eth_hash> <in_zec_hash> <in_num_iterations_exp> <out_params.params>");
        std::process::exit(exitcode::USAGE);
    }
    let in_params_filename = &args[1];
    let btc_hash = &args[2];
    let eth_hash = &args[3];
    let zec_hash = &args[4];
    let num_iterations_exp = &args[5].parse::<usize>().unwrap();
    let out_params_filename = &args[6];
    let print_progress = true;

    if *num_iterations_exp < 10 || *num_iterations_exp > 63 {
        println!("in_num_iterations_exp should be in [10, 63] range");
        std::process::exit(exitcode::DATAERR);
    }

    let disallow_points_at_infinity = false;

    // Create an RNG based on the outcome of the random beacon
    let mut rng = {
        use byteorder::{BigEndian, ReadBytesExt};
        use rand::SeedableRng;
        use rand_chacha::ChaChaRng;
        use std::convert::TryInto;

        let mut cur_hash = {
            let mut h = Sha256::new();

            for beacon_hash in [btc_hash, eth_hash, zec_hash] {
                // The hash used for the beacon
                let hash_result = hex::decode(beacon_hash);
                if hash_result.is_err() {
                    println!("Beacon hash should be in hexadecimal format");
                    std::process::exit(exitcode::DATAERR);
                }
                let hash_result = hash_result.unwrap();
                if hash_result.len() != 32 {
                    println!("Beacon hash should be 32 bytes long");
                    std::process::exit(exitcode::DATAERR);
                }
                h.update(&hash_result);
            }
            h.finalize().to_vec()
        };

        if cur_hash.len() != 32 {
            println!("Beacon hash should be 32 bytes long");
            std::process::exit(exitcode::DATAERR);
        }
        // Performs 2^n hash iterations over it
        let n: usize = *num_iterations_exp;

        for i in 0..(1u64 << n) {
            // Print 1024 of the interstitial states
            // so that verification can be
            // parallelized

            if i % (1u64 << (n - 10)) == 0 {
                print!("{}: ", i);
                for b in cur_hash.iter() {
                    print!("{:02x}", b);
                }
                println!("");
            }

            let mut h = Sha256::new();
            h.update(&cur_hash);
            cur_hash = h.finalize().to_vec();
        }

        print!("Final result of beacon: ");
        for b in cur_hash.iter() {
            print!("{:02x}", b);
        }
        println!();

        ChaChaRng::from_seed(cur_hash[0..32].try_into().unwrap())
    };

    println!("Done creating a beacon RNG");

    let reader = OpenOptions::new()
        .read(true)
        .open(in_params_filename)
        .expect("unable to open.");
    let mut reader = BufReader::with_capacity(1024 * 1024, reader);

    reader.seek_relative(64).unwrap();
    let mut spend_params = MPCParameters::read(&mut reader, false).expect("unable to read params");

    println!("Contributing to MASP Spend {}...", in_params_filename);

    let spend_hash = spend_params.contribute(&mut rng, &0);

    let mut output_params = MPCParameters::read(&mut reader, false).expect("unable to read params");

    println!("Contributing to MASP Output {}...", in_params_filename);

    let output_hash = output_params.contribute(&mut rng, &0);

    let mut convert_params =
        MPCParameters::read(&mut reader, false).expect("unable to read MASP Convert params");

    println!("Contributing to MASP Convert {}...", in_params_filename);
    let mut progress_update_interval: u32 = 0;
    /*if print_progress {
        let parsed = args[5].parse::<u32>();
        if !parsed.is_err() {
            progress_update_interval = parsed.unwrap();
        }
    }
    */
    let convert_hash = convert_params.contribute(&mut rng, &progress_update_interval);

    let mut h = Blake2b512::new();
    h.update(&spend_hash);
    h.update(&output_hash);
    h.update(&convert_hash);
    let h = h.finalize();

    println!("Contribution hash: 0x{:02x}", h.iter().format(""));

    let mut f = File::create(out_params_filename).unwrap();
    for _ in 0..64 {
        f.write_all(&[0u8]).unwrap();
    }
    println!("Writing MASP Spend parameters to {}.", out_params_filename);
    spend_params
        .write(&mut f)
        .expect("failed to write updated MASP Spend parameters");
    if print_progress {
        println!("wrote MASP Spend");
    }

    println!("Writing MASP Output parameters to {}.", out_params_filename);
    output_params
        .write(&mut f)
        .expect("failed to write updated MASP Output parameters");
    if print_progress {
        println!("wrote MASP Output");
    }

    println!(
        "Writing MASP Convert parameters to {}.",
        out_params_filename
    );
    convert_params
        .write(&mut f)
        .expect("failed to write updated MASP Convert parameters");
    if print_progress {
        println!("wrote MASP Convert");
    }
}

// Place beacon value here (2^42 SHA256 hash of Bitcoin block hash #534861)
//        let beacon_value: [u8; 32] =
//          hex!("2bf41a959668e5b9b688e58d613b3dcc99ee159a880cf764ec67e6488d8b8af3");
