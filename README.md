# Sim Circuit

A Rust library that provides utilities for simulating logic gate circuits.

## CLI

```sh
cargo run -- --circuit-dir example_circuits/a_plus_b
```

```json
{
  "c": "8"
}
```

When using a directory, it should contain these files:
- `circuit_info.json`: Information about inputs, outputs, and constants
- `circuit_input.json`: Input values to provide to the circuit
- `circuit.txt`: The circuit layout in [Bristol format](https://nigelsmart.github.io/MPC-Circuits/)

Alternatively, explicit locations for all or some of the files can be provided using their specific parameters (see `cargo run -- --help`).

## API

This crate primarily exposes the `simulate` function, which takes an `ArithmeticCircuit` and inputs to that circuit.

```rs
pub fn simulate<N: Number>(
    circuit: &ArithmeticCircuit,
    inputs: &HashMap<String, N>,
) -> Result<HashMap<String, N>, NumberError>;
```

Circuits can be evaluated over any kind of number that implements the `Number` trait. For reference, `NumberU32` is provided which uses unsigned 32-bit arithmetic.
