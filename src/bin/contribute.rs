use blake2::{Blake2b512, Digest};
use itertools::Itertools;
use masp_phase2::MPCParameters;
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

    //let disallow_points_at_infinity = false;

    if print_progress {
        println!("starting");
    }
    // Create an RNG based on a mixture of system randomness and user provided randomness
    let mut rng = {
        use rand::{Rng, SeedableRng};
        use rand_chacha::ChaChaRng;
        use std::convert::TryInto;

        let h = {
            let mut system_rng = rand::rngs::OsRng;
            let mut h = Blake2b512::new();

            // Gather 1024 bytes of entropy from the system
            for _ in 0..1024 {
                let r: u8 = system_rng.gen();
                h.update(&[r]);
            }

            // Hash it all up to make a seed
            h.update(&entropy.as_bytes());
            h.finalize()
        };

        ChaChaRng::from_seed(h[0..32].try_into().unwrap())
    };

    let reader = OpenOptions::new()
        .read(true)
        .open(in_params_filename)
        .expect("unable to open.");

    let mut spend_params =
        MPCParameters::read(&reader, false).expect("unable to read MASP Spend params");

    println!("Contributing to MASP Spend {}...", in_params_filename);
    let mut progress_update_interval: u32 = 0;
    if print_progress {
        let parsed = args[5].parse::<u32>();
        if !parsed.is_err() {
            progress_update_interval = parsed.unwrap();
        }
    }
    let spend_hash = spend_params.contribute(&mut rng, &progress_update_interval);

    let mut output_params =
        MPCParameters::read(&reader, false).expect("unable to read MASP Output params");

    println!("Contributing to MASP Output {}...", in_params_filename);
    let mut progress_update_interval: u32 = 0;
    if print_progress {
        let parsed = args[5].parse::<u32>();
        if !parsed.is_err() {
            progress_update_interval = parsed.unwrap();
        }
    }
    let output_hash = output_params.contribute(&mut rng, &progress_update_interval);

    let mut convert_params =
        MPCParameters::read(&reader, false).expect("unable to read MASP Convert params");

    println!("Contributing to MASP Convert {}...", in_params_filename);
    let mut progress_update_interval: u32 = 0;
    if print_progress {
        let parsed = args[5].parse::<u32>();
        if !parsed.is_err() {
            progress_update_interval = parsed.unwrap();
        }
    }
    let convert_hash = convert_params.contribute(&mut rng, &progress_update_interval);

    let mut h = Blake2b512::new();
    h.update(&spend_hash);
    h.update(&output_hash);
    h.update(&convert_hash);
    let h = h.finalize();

    println!("Contribution hash: 0x{:02x}", h.iter().format(""));

    let mut f = File::create(out_params_filename).unwrap();

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
