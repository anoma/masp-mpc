use blake2::{Blake2b, Digest};
use phase2::parameters::{verify_contribution, MPCParameters};
use std::fs::File;
use std::io::BufReader;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        println!("Usage: \n<in_params.params> <out_params.params>");
        std::process::exit(exitcode::USAGE);
    }
    let params = File::open(&args[1]).unwrap();
    let mut params = BufReader::with_capacity(1024 * 1024, params);

    let new_params = File::open(&args[2]).unwrap();
    let mut new_params = BufReader::with_capacity(1024 * 1024, new_params);

    let masp_spend = MPCParameters::read(&mut params, false, true)
        .expect("couldn't deserialize Sapling Spend params");

    let masp_output = MPCParameters::read(&mut params, false, true)
        .expect("couldn't deserialize Sapling Output params");

    let new_masp_spend = MPCParameters::read(&mut new_params, false, true)
        .expect("couldn't deserialize Sapling Spend new_params");

    let new_masp_output = MPCParameters::read(&mut new_params, false, true)
        .expect("couldn't deserialize Sapling Output new_params");

    let h1 = match verify_contribution(&masp_spend, &new_masp_spend) {
        Ok(hash) => hash,
        Err(_) => panic!("invalid transformation!"),
    };

    let h2 = match verify_contribution(&masp_output, &new_masp_output) {
        Ok(hash) => hash,
        Err(_) => panic!("invalid transformation!"),
    };

    let mut h = Blake2b::new();
    h.update(&h1);
    h.update(&h2);
    let h = h.finalize();

    println!("{}", into_hex(h.as_ref()));
}

fn into_hex(h: &[u8]) -> String {
    let mut f = String::new();

    for byte in &h[..] {
        f += &format!("{:02x}", byte);
    }

    f
}
