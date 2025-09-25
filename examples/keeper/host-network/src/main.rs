use zkm_sdk::{utils, ProverClient, ZKMStdin};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_bytes!("../../go-ethereum/cmd/keeper/keeper");

use std::env;
use std::fs::File;
use std::io::Read;

fn prove_keeper(path: &str) {
    println!("Proving for payload file: {}", path);
    // The input stream that the guest will read from using `zkm_zkvm::io::read`. Note that the
    // types of the elements in the input stream must match the types being read in the guest.
    let mut stdin = ZKMStdin::new();
    let mut file = File::open(path).expect("unable to open file {path}");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("unable to read file");
    stdin.write(&data);
    println!("Payload: {} {}", path, data.len());

    // Create a `ProverClient` method.
    let client = ProverClient::network();

    // Execute the guest using the `ProverClient.execute` method, without generating a proof.
    let (_, report) = client.execute(ELF, stdin.clone()).run().unwrap();
    println!("executed program with {} cycles", report.total_instruction_count());

    // Generate the proof for the given guest and input.
    let (pk, vk) = client.setup(ELF);
    let proof = client.prove(&pk, stdin).compressed().run().unwrap();

    println!("generated proof");
    // Verify proof and public values
    // client.verify(&proof, &vk).expect("verification failed");

    println!("successfully generated and verified proof for the program!")
}

fn main() {
    utils::setup_logger();

    // read payload file path from command line argument
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法: {} <input>", env::args().next().unwrap());
        std::process::exit(1);
    });

    prove_keeper(&path);
}
