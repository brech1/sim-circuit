use sim_circuit::circuit::{Circuit, Gate, Node, Operation};

#[test]
fn test_matrix_multiplication() {
    let mut circuit = Circuit::new();

    // Initialize nodes
    for i in 1..=20 {
        circuit.add_node(i, Node::new()).unwrap();
    }

    // C11 = A11*B11 + A12*B21
    circuit
        .add_gate(Gate::new(Operation::Multiply, 1, 5, 13))
        .unwrap();
    circuit
        .add_gate(Gate::new(Operation::Multiply, 2, 7, 14))
        .unwrap();
    circuit
        .add_gate(Gate::new(Operation::Add, 13, 14, 9))
        .unwrap();

    // C12 = A11*B12 + A12*B22
    circuit
        .add_gate(Gate::new(Operation::Multiply, 1, 6, 15))
        .unwrap();
    circuit
        .add_gate(Gate::new(Operation::Multiply, 2, 8, 16))
        .unwrap();
    circuit
        .add_gate(Gate::new(Operation::Add, 15, 16, 10))
        .unwrap();

    // C21 = A21*B11 + A22*B21
    circuit
        .add_gate(Gate::new(Operation::Multiply, 3, 5, 17))
        .unwrap();
    circuit
        .add_gate(Gate::new(Operation::Multiply, 4, 7, 18))
        .unwrap();
    circuit
        .add_gate(Gate::new(Operation::Add, 17, 18, 11))
        .unwrap();

    // C22 = A21*B12 + A22*B22
    circuit
        .add_gate(Gate::new(Operation::Multiply, 3, 6, 19))
        .unwrap();
    circuit
        .add_gate(Gate::new(Operation::Multiply, 4, 8, 20))
        .unwrap();
    circuit
        .add_gate(Gate::new(Operation::Add, 19, 20, 12))
        .unwrap();

    // [1 2]   [1 1]   [3 3]
    // [3 4] x [1 1] = [7 7]

    let inputs = vec![1, 2, 3, 4, 1, 1, 1, 1];
    let outputs = circuit.execute(&inputs).unwrap();

    assert_eq!(outputs, vec![3, 3, 7, 7]);
}
