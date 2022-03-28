use blake2::{Blake2b512, Digest};
use masp_phase2::{verify_contribution, MPCParameters};
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

    let masp_spend =
        MPCParameters::read(&mut params, true).expect("couldn't deserialize MASP Spend params");

    let masp_output =
        MPCParameters::read(&mut params, true).expect("couldn't deserialize MASP Output params");
    let masp_convert =
        MPCParameters::read(&mut params, true).expect("couldn't deserialize MASP Convert params");

    let new_masp_spend = MPCParameters::read(&mut new_params, true)
        .expect("couldn't deserialize MASP Spend new_params");

    let new_masp_output = MPCParameters::read(&mut new_params, true)
        .expect("couldn't deserialize MASP Output new_params");

    let new_masp_convert = MPCParameters::read(&mut new_params, true)
        .expect("couldn't deserialize MASP Convert new_params");

    let spend_hash = match verify_contribution(&masp_spend, &new_masp_spend) {
        Ok(hash) => hash,
        Err(_) => panic!("invalid MASP Spend transformation!"),
    };

    let output_hash = match verify_contribution(&masp_output, &new_masp_output) {
        Ok(hash) => hash,
        Err(_) => panic!("invalid MASP Output transformation!"),
    };

    let convert_hash = match verify_contribution(&masp_convert, &new_masp_convert) {
        Ok(hash) => hash,
        Err(_) => panic!("invalid MASP Convert transformation!"),
    };

    let mut h = Blake2b512::new();
    h.update(&spend_hash);
    h.update(&output_hash);
    h.update(&convert_hash);
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
