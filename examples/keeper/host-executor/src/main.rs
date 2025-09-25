use zkm_sdk::{utils, ProverClient, ZKMStdin};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] = include_bytes!("../../go-ethereum/cmd/keeper/keeper");

use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;

fn prove_keeper(path: &str) {
    // The input stream that the guest will read from using `zkm_zkvm::io::read`. Note that the
    // types of the elements in the input stream must match the types being read in the guest.
    let mut stdin = ZKMStdin::new();
    let mut file = File::open(path).expect("unable to open file");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("unable to read file");
    stdin.write(&data);
    println!("Payload: {} {}", path, data.len());

    // Create a `ProverClient` method.
    let client = ProverClient::new();

    // Execute the guest using the `ProverClient.execute` method, without generating a proof.
    let (_, report) = client.execute(ELF, stdin.clone()).run().unwrap();
    println!("executed program with {} cycles", report.total_instruction_count());
}

fn main() -> io::Result<()> {
    utils::setup_logger();

    // read payload directory path from command line argument
    let dir = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("用法: {} <input>", env::args().next().unwrap());
        std::process::exit(1);
    });

    for entry in fs::read_dir(dir)? {
        // 读取目录
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            // 只处理文件
            println!("Payload: {}", path.display());
            prove_keeper(path.to_str().unwrap());
        }
    }
    Ok(())
}
