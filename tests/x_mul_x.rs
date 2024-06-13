use serde_json::from_str;
use sim_circuit::{arithmetic_circuit::ArithmeticCircuit, simulate, NumberU32};

#[test]
fn test_x_mul_x() {
    let circuit = ArithmeticCircuit::from_info_and_bristol_string(
        from_str(
            r#"
                {
                    "input_name_to_wire_index": {
                        "x": 0
                    },
                    "constants": {},
                    "output_name_to_wire_index": {
                        "y": 1
                    }
                }
            "#,
        )
        .unwrap(),
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
        &circuit,
        &[("x".to_string(), NumberU32(5))].into_iter().collect(),
    )
    .unwrap();

    assert_eq!(
        outputs,
        [("y".to_string(), NumberU32(25))].into_iter().collect()
    );
}
