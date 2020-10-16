use itertools::Itertools;
use phase2::parameters::MPCParameters;
use sha2::{Digest, Sha256};
use std::fs::File;
use std::fs::OpenOptions;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        println!("Usage: \n<in_params.params> <in_beacon_hash> <in_num_iterations_exp> <out_params.params>");
        std::process::exit(exitcode::USAGE);
    }
    let in_params_filename = &args[1];
    let beacon_hash = &args[2];
    let num_iterations_exp = &args[3].parse::<usize>().unwrap();
    let out_params_filename = &args[4];

    if *num_iterations_exp < 10 || *num_iterations_exp > 63 {
        println!("in_num_iterations_exp should be in [10, 63] range");
        std::process::exit(exitcode::DATAERR);
    }

    let disallow_points_at_infinity = false;

    // Create an RNG based on the outcome of the random beacon
    let mut rng = {
        use byteorder::{BigEndian, ReadBytesExt};
        use rand::chacha::ChaChaRng;
        use rand::SeedableRng;

        // The hash used for the beacon
        let hash_result = hex::decode(beacon_hash);
        if hash_result.is_err() {
            println!("Beacon hash should be in hexadecimal format");
            std::process::exit(exitcode::DATAERR);
        }
        let mut cur_hash = hash_result.unwrap();
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

        let mut digest = &cur_hash[..];

        let mut seed = [0u32; 8];
        for i in 0..8 {
            seed[i] = digest
                .read_u32::<BigEndian>()
                .expect("digest is large enough for this to work");
        }

        ChaChaRng::from_seed(&seed)
    };

    println!("Done creating a beacon RNG");

    let reader = OpenOptions::new()
        .read(true)
        .open(in_params_filename)
        .expect("unable to open.");
    let mut spend_params = MPCParameters::read(&reader, disallow_points_at_infinity, true)
        .expect("unable to read params");

    println!("Contributing to Spend {}...", in_params_filename);

    let spend_hash = spend_params.contribute(&mut rng, &0);

    let mut output_params = MPCParameters::read(&reader, disallow_points_at_infinity, true)
        .expect("unable to read params");

    println!("Contributing to Output {}...", in_params_filename);

    let output_hash = output_params.contribute(&mut rng, &0);

    let mut h = blake2::Blake2b::new();
    h.update(&spend_hash);
    h.update(&output_hash);
    let h = h.finalize();
    println!("Contribution hash: 0x{:02x}", h.iter().format(""));

    println!("Writing Spend parameters to {}.", out_params_filename);
    let mut f = File::create(out_params_filename).unwrap();
    spend_params
        .write(&mut f)
        .expect("failed to write updated parameters");

    println!("Writing Output parameters to {}.", out_params_filename);
    output_params
        .write(&mut f)
        .expect("failed to write updated parameters");
}

// Place beacon value here (2^42 SHA256 hash of Bitcoin block hash #534861)
//        let beacon_value: [u8; 32] =
//          hex!("2bf41a959668e5b9b688e58d613b3dcc99ee159a880cf764ec67e6488d8b8af3");
