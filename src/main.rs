use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use serde_json::from_str;
use sim_circuit::{
    arithmetic_circuit::{ArithmeticCircuit, CircuitInfo},
    simulate, Number, NumberU32,
};

pub fn main() {
    let Args {
        circuit_dir,
        circuit,
        circuit_info,
        circuit_input,
        output,
    } = Args::parse();

    let circuit_path = match (&circuit_dir, circuit) {
        (_, Some(circuit)) => circuit,
        (Some(circuit_dir), _) => circuit_dir.join("circuit.txt"),
        _ => err_required_args(),
    };

    let circuit_info_path = match (&circuit_dir, circuit_info) {
        (_, Some(circuit_info)) => circuit_info,
        (Some(circuit_dir), _) => circuit_dir.join("circuit_info.json"),
        _ => err_required_args(),
    };

    let circuit_input_path = match (&circuit_dir, circuit_input) {
        (_, Some(circuit_input)) => circuit_input,
        (Some(circuit_dir), _) => circuit_dir.join("circuit_input.json"),
        _ => err_required_args(),
    };

    let circuit_info =
        from_str::<CircuitInfo>(&std::fs::read_to_string(circuit_info_path).unwrap()).unwrap();

    let circuit = ArithmeticCircuit::read_info_and_bristol(
        circuit_info,
        &mut BufReader::new(File::open(circuit_path).unwrap()),
    )
    .unwrap();

    let circuit_input =
        from_str::<HashMap<String, String>>(&std::fs::read_to_string(circuit_input_path).unwrap())
            .unwrap();

    let mut input_u32 = HashMap::<String, NumberU32>::new();

    for (name, value) in circuit_input {
        input_u32.insert(name, NumberU32(value.parse().unwrap()));
    }

    let outputs = simulate(&circuit, &input_u32).unwrap();

    let mut output_strings = HashMap::<String, String>::new();

    for (name, value) in outputs {
        output_strings.insert(name, value.to_string());
    }

    if let Some(output) = output {
        std::fs::write(
            output,
            serde_json::to_string_pretty(&output_strings).unwrap(),
        )
        .unwrap();
    } else {
        println!("{}", serde_json::to_string_pretty(&output_strings).unwrap());
    }
}

fn err_required_args() -> ! {
    eprintln!("Required: either --circuit-dir or --circuit");
    std::process::exit(1);
}

#[derive(Parser, Debug)]
#[clap(name = "Circuit Simulator")]
struct Args {
    /// Circuit directory containing circuit.txt, circuit_info.json, circuit_input.json
    #[arg(long, help = "Path to circuit directory")]
    circuit_dir: Option<PathBuf>,

    #[arg(long, help = "Path to circuit.txt")]
    circuit: Option<PathBuf>,

    #[arg(long, help = "Path to circuit_info.json")]
    circuit_info: Option<PathBuf>,

    #[arg(long, help = "Path to circuit_input.json")]
    circuit_input: Option<PathBuf>,

    /// Output file to write the result
    #[arg(short, long, help = "Path to output file (default: stdout)")]
    output: Option<PathBuf>,
}
