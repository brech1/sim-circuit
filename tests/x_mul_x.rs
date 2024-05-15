use sim_circuit::circuit::{Circuit, Gate, Node, Operation};

#[test]
fn test_x_mul_x() {
    let mut circuit = Circuit::new();

    // Initialize nodes
    circuit.add_node(1, Node::new()).unwrap();
    circuit.add_node(2, Node::new()).unwrap();

    // out = x * x
    circuit
        .add_gate(Gate::new(Operation::Multiply, 1, 1, 2))
        .unwrap();

    let inputs = vec![5];
    let outputs = circuit.execute(&inputs).unwrap();

    assert_eq!(outputs, vec![25]);
}
