use zkm_sdk::{utils, ProverClient, ZKMProofWithPublicValues, ZKMStdin};
use std::time::{Duration, Instant};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_bytes!("../../reth");
const STDIN: &[u8] = include_bytes!("../../reth-stdin.bin");
fn prove_reth() {
    // The input stream that the guest will read from using `zkm_zkvm::io::read`. Note that the
    // types of the elements in the input stream must match the types being read in the guest.

    let stdin: ZKMStdin = bincode::deserialize(STDIN).unwrap();

    // Create a `ProverClient` method.
    let client = ProverClient::new();

    let start = Instant::now();
    // Execute the guest using the `ProverClient.execute` method, without generating a proof.
    let (_, report) = client.execute(ELF, stdin.clone()).run().unwrap();
    let end = Instant::now();
    let duration = end.duration_since(start);

    println!("executed program with {} cycles, {} seconds", report.total_instruction_count(), duration.as_secs_f64());
/*
    // Generate the proof for the given guest and input.
    let (pk, vk) = client.setup(ELF);
    let proof = client.prove(&pk, stdin).compressed().run().unwrap();

    println!("generated proof");
    // Verify proof and public values
    client.verify(&proof, &vk).expect("verification failed");

    // Test a round trip of proof serialization and deserialization.
    proof.save("proof-with-pis.bin").expect("saving proof failed");
    let deserialized_proof =
        ZKMProofWithPublicValues::load("proof-with-pis.bin").expect("loading proof failed");

    // Verify the deserialized proof.
    client.verify(&deserialized_proof, &vk).expect("verification failed");

    println!("successfully generated and verified proof for the program!")
 */
}

fn main() {
    utils::setup_logger();

    prove_reth();
}
