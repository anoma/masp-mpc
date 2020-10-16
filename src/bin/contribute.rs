use blake2::{Blake2b, Digest};
use itertools::Itertools;
use phase2::parameters::MPCParameters;
use std::fs::File;
use std::fs::OpenOptions;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 4 && args.len() != 6 {
        println!("Usage: \n<in_params.params> <out_params.params> <in_str_entropy>");
        std::process::exit(exitcode::USAGE);
    }
    if args.len() == 6 && args[4] != "-v" {
        println!("Usage: \n<in_params.params> <out_params.params> <in_str_entropy> -v <progress_interval>");
        std::process::exit(exitcode::USAGE);
    }
    let in_params_filename = &args[1];
    let out_params_filename = &args[2];
    let entropy = &args[3];
    let print_progress = args.len() == 6 && args[4] == "-v";

    let disallow_points_at_infinity = false;

    if print_progress {
        println!("starting");
    }
    // Create an RNG based on a mixture of system randomness and user provided randomness
    let mut rng = {
        use byteorder::{BigEndian, ReadBytesExt};
        use rand::chacha::ChaChaRng;
        use rand::{OsRng, Rng, SeedableRng};

        let h = {
            let mut system_rng = OsRng::new().unwrap();
            let mut h = Blake2b::new();

            // Gather 1024 bytes of entropy from the system
            for _ in 0..1024 {
                let r: u8 = system_rng.gen();
                h.update(&[r]);
            }

            // Hash it all up to make a seed
            h.update(&entropy.as_bytes());
            h.finalize()
        };

        let mut digest = &h[..];

        // Interpret the first 32 bytes of the digest as 8 32-bit words
        let mut seed = [0u32; 8];
        for i in 0..8 {
            seed[i] = digest
                .read_u32::<BigEndian>()
                .expect("digest is large enough for this to work");
        }

        ChaChaRng::from_seed(&seed)
    };

    let reader = OpenOptions::new()
        .read(true)
        .open(in_params_filename)
        .expect("unable to open.");

    let mut spend_params = MPCParameters::read(&reader, disallow_points_at_infinity, true)
        .expect("unable to read params");

    println!("Contributing to Spend {}...", in_params_filename);
    let mut progress_update_interval: u32 = 0;
    if print_progress {
        let parsed = args[5].parse::<u32>();
        if !parsed.is_err() {
            progress_update_interval = parsed.unwrap();
        }
    }
    let spend_hash = spend_params.contribute(&mut rng, &progress_update_interval);

    let mut output_params = MPCParameters::read(&reader, disallow_points_at_infinity, true)
        .expect("unable to read params");

    println!("Contributing to Output {}...", in_params_filename);
    let mut progress_update_interval: u32 = 0;
    if print_progress {
        let parsed = args[5].parse::<u32>();
        if !parsed.is_err() {
            progress_update_interval = parsed.unwrap();
        }
    }
    let output_hash = output_params.contribute(&mut rng, &progress_update_interval);

    let mut h = Blake2b::new();
    h.update(&spend_hash);
    h.update(&output_hash);
    let h = h.finalize();

    println!("Contribution hash: 0x{:02x}", h.iter().format(""));

    let mut f = File::create(out_params_filename).unwrap();

    println!("Writing Spend parameters to {}.", out_params_filename);
    spend_params
        .write(&mut f)
        .expect("failed to write updated Spend parameters");
    if print_progress {
        println!("wrote Spend");
    }

    println!("Writing Output parameters to {}.", out_params_filename);
    output_params
        .write(&mut f)
        .expect("failed to write updated Output parameters");
    if print_progress {
        println!("wrote Output");
    }
}
