//! This binary just splits the parameters up into separate files.

use masp_phase2::MPCParameters;
use std::fs::File;
use std::io::{BufReader, BufWriter};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        println!("Usage: \n<params.params>");
        std::process::exit(exitcode::USAGE);
    }
    let current_params = File::open(&args[1]).expect("couldn't open params");
    let mut current_params = BufReader::with_capacity(1024 * 1024, current_params);

    // Used for Namada Trusted Setup contribution files where the first 64 bytes contain the hash of the previous contribution file
    // Offset 64 bytes to access the parameters 
    current_params.seek_relative(64).unwrap();

    let masp_spend = MPCParameters::read(&mut current_params, false)
        .expect("couldn't deserialize MASP Spend params");

    let masp_output = MPCParameters::read(&mut current_params, false)
        .expect("couldn't deserialize MASP Output params");

    let masp_convert = MPCParameters::read(&mut current_params, false)
        .expect("couldn't deserialize MASP Convert params");

    {
        let f = File::create("masp-spend.params").expect("couldn't create `./masp-spend.params`");
        let mut f = BufWriter::with_capacity(1024 * 1024, f);
        masp_spend
            .write(&mut f)
            .expect("couldn't write new MASP Spend params");
    }

    {
        let f = File::create("masp-output.params").expect("couldn't create `./masp-output.params`");
        let mut f = BufWriter::with_capacity(1024 * 1024, f);
        masp_output
            .write(&mut f)
            .expect("couldn't write new MASP Output params");
    }

    {
        let f =
            File::create("masp-convert.params").expect("couldn't create `./masp-convert.params`");
        let mut f = BufWriter::with_capacity(1024 * 1024, f);
        masp_convert
            .write(&mut f)
            .expect("couldn't write new MASP Convert params");
    }
}
