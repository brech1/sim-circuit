use circom_2_arithc::arithmetic_circuit::{ArithmeticCircuit, CircuitInfo};
use sim_circuit::{simulate, NumberU32};

#[test]
fn test_x_mul_x() {
    let circuit = ArithmeticCircuit::from_info_and_bristol_string(
        CircuitInfo {
            input_name_to_wire_index: [("x".to_string(), 0)].into_iter().collect(),
            constants: Default::default(),
            output_name_to_wire_index: [("y".to_string(), 1)].into_iter().collect(),
        },
        "
            1 2
            1 1
            1 1

            2 1 0 0 1 AMul
        ",
    )
    .unwrap();

    // 5 * 5 = 25

    let outputs = simulate(
        circuit,
        [("x".to_string(), NumberU32(5))].into_iter().collect(),
    )
    .unwrap();

    assert_eq!(
        outputs,
        [("y".to_string(), NumberU32(25))].into_iter().collect()
    );
}
