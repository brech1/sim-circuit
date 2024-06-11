use circom_2_arithc::arithmetic_circuit::ArithmeticCircuit;
use serde_json::from_str;
use sim_circuit::{simulate, NumberU32};

#[test]
fn test_matrix_multiplication() {
    let circuit = ArithmeticCircuit::from_info_and_bristol_string(
        from_str(
            r#"
                {
                    "input_name_to_wire_index": {
                        "a11": 0,
                        "a12": 1,
                        "a21": 2,
                        "a22": 3,
                        "b11": 4,
                        "b12": 5,
                        "b21": 6,
                        "b22": 7
                    },
                    "constants": {},
                    "output_name_to_wire_index": {
                        "c11": 10,
                        "c12": 13,
                        "c21": 16,
                        "c22": 19
                    }
                }
            "#,
        )
        .unwrap(),
        "
            12 20
            8 1 1 1 1 1 1 1 1
            4 1 1 1 1

            2 1 0 4 8 AMul
            2 1 1 6 9 AMul
            2 1 8 9 10 AAdd
            2 1 0 5 11 AMul
            2 1 1 7 12 AMul
            2 1 11 12 13 AAdd
            2 1 2 4 14 AMul
            2 1 3 6 15 AMul
            2 1 14 15 16 AAdd
            2 1 2 5 17 AMul
            2 1 3 7 18 AMul
            2 1 17 18 19 AAdd
        ",
    )
    .unwrap();

    // [1 2]   [1 1]   [3 3]
    // [3 4] x [1 1] = [7 7]

    let outputs = simulate(
        &circuit,
        &[
            ("a11".to_string(), NumberU32(1)),
            ("a12".to_string(), NumberU32(2)),
            ("a21".to_string(), NumberU32(3)),
            ("a22".to_string(), NumberU32(4)),
            ("b11".to_string(), NumberU32(1)),
            ("b12".to_string(), NumberU32(1)),
            ("b21".to_string(), NumberU32(1)),
            ("b22".to_string(), NumberU32(1)),
        ]
        .into_iter()
        .collect(),
    )
    .unwrap();

    assert_eq!(
        outputs,
        [
            ("c11".to_string(), NumberU32(3)),
            ("c12".to_string(), NumberU32(3)),
            ("c21".to_string(), NumberU32(7)),
            ("c22".to_string(), NumberU32(7)),
        ]
        .into_iter()
        .collect()
    );
}
