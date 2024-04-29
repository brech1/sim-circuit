//! # Circuit Module
//!
//! Circuit module.

use std::collections::{HashMap, HashSet};
use thiserror::Error;

/// Circuit node struct.
pub struct Node {
    /// Node value.
    value: Option<u32>,
}

impl Node {
    /// Create a new node with no initial value.
    pub fn new() -> Node {
        Node { value: None }
    }

    /// Set the node value.
    pub fn set_value(&mut self, value: u32) {
        self.value = Some(value);
    }

    /// Get the node value.
    pub fn get_value(&self) -> Option<u32> {
        self.value
    }
}

/// Gate operations.
pub enum Operation {
    // Arithmetic Operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Exponentiate,
    Modulus,

    // Comparison Operations
    Equals,
    NotEquals,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,

    // Logical Operations
    And,
    Or,

    // Bitwise Operations
    AndBitwise,
    OrBitwise,
    XorBitwise,
    ShiftLeft,
    ShiftRight,
}

/// Circuit gate struct.
pub struct Gate {
    /// Gate operation.
    operation: Operation,
    /// Left input node.
    left_input: u32,
    /// Right input node.
    right_input: u32,
    /// Output node.
    output: u32,
}

impl Gate {
    /// Creates a new gate.
    pub fn new(operation: Operation, left_input: u32, right_input: u32, output: u32) -> Gate {
        Gate {
            operation,
            left_input,
            right_input,
            output,
        }
    }

    /// Executes this gate's operation with the given input values and returns the result.
    pub fn execute(&self, left_input: u32, right_input: u32) -> u32 {
        match self.operation {
            Operation::Add => left_input + right_input,
            Operation::Subtract => left_input - right_input,
            Operation::Multiply => left_input * right_input,
            Operation::Divide => left_input / right_input, // Will panic if right_input is zero
            Operation::Exponentiate => left_input.pow(right_input as u32),
            Operation::Modulus => left_input % right_input, // Will panic if right_input is zero
            Operation::Equals => (left_input == right_input) as u32,
            Operation::NotEquals => (left_input != right_input) as u32,
            Operation::LessThan => (left_input < right_input) as u32,
            Operation::LessOrEqual => (left_input <= right_input) as u32,
            Operation::GreaterThan => (left_input > right_input) as u32,
            Operation::GreaterOrEqual => (left_input >= right_input) as u32,
            Operation::And => (left_input != 0 && right_input != 0) as u32,
            Operation::Or => (left_input != 0 || right_input != 0) as u32,
            Operation::AndBitwise => left_input & right_input,
            Operation::OrBitwise => left_input | right_input,
            Operation::XorBitwise => left_input ^ right_input,
            Operation::ShiftLeft => left_input << right_input,
            Operation::ShiftRight => left_input >> right_input,
        }
    }
}

/// Generic circuit struct.
pub struct Circuit {
    /// Circuit nodes.
    nodes: HashMap<u32, Node>,
    /// Circuit gates.
    gates: Vec<Gate>,
}

impl Circuit {
    /// Create a new circuit.
    pub fn new() -> Circuit {
        Circuit {
            nodes: HashMap::new(),
            gates: Vec::new(),
        }
    }

    /// Add a node to the circuit.
    pub fn add_node(&mut self, id: u32, node: Node) {
        self.nodes.insert(id, node);
    }

    /// Add a gate to the circuit.
    pub fn add_gate(&mut self, gate: Gate) {
        self.gates.push(gate);
    }

    /// Get the input and output nodes from the circuit.
    pub fn get_circuit_io(&self) -> (Vec<u32>, Vec<u32>) {
        let mut input_candidates = HashSet::new();
        let mut output_candidates = HashSet::new();

        for gate in &self.gates {
            input_candidates.insert(gate.left_input);
            input_candidates.insert(gate.right_input);
            output_candidates.insert(gate.output);
        }

        // Filter out incorrect input and output candidates
        let mut inputs = input_candidates
            .difference(&output_candidates)
            .copied()
            .collect::<Vec<_>>();
        let mut outputs = output_candidates
            .difference(&input_candidates)
            .copied()
            .collect::<Vec<_>>();

        inputs.sort_unstable();
        outputs.sort_unstable();

        (inputs, outputs)
    }

    /// Execute the circuit with the given input values and return the output values.
    pub fn execute(&mut self, inputs: &[u32]) -> Result<Vec<u32>, CircuitError> {
        let (input_nodes, output_nodes) = self.get_circuit_io();

        // Check if the input size matches the circuit input size
        if inputs.len() != input_nodes.len() {
            return Err(CircuitError::InvalidInputSize);
        }

        // Assign input values to input nodes
        for (node_id, &value) in input_nodes.iter().zip(inputs) {
            self.nodes
                .get_mut(node_id)
                .ok_or(CircuitError::NodeNotFound(*node_id))?
                .set_value(value);
        }

        // Execute gates
        for gate in &self.gates {
            let left_input = self
                .nodes
                .get(&gate.left_input)
                .and_then(|node| node.get_value())
                .ok_or_else(|| {
                    if self.nodes.contains_key(&gate.left_input) {
                        CircuitError::NodeValueNotAssigned(gate.left_input)
                    } else {
                        CircuitError::NodeNotFound(gate.left_input)
                    }
                })?;

            let right_input = self
                .nodes
                .get(&gate.right_input)
                .and_then(|node| node.get_value())
                .ok_or_else(|| {
                    if self.nodes.contains_key(&gate.right_input) {
                        CircuitError::NodeValueNotAssigned(gate.right_input)
                    } else {
                        CircuitError::NodeNotFound(gate.right_input)
                    }
                })?;

            let output = gate.execute(left_input, right_input);

            let output_node = self
                .nodes
                .get_mut(&gate.output)
                .ok_or(CircuitError::NodeNotFound(gate.output))?;

            if output_node.get_value().is_some() {
                return Err(CircuitError::OutputNodePreAssigned(gate.output));
            }

            output_node.set_value(output);
        }

        // Collect output values
        output_nodes
            .iter()
            .map(|node_id| {
                self.nodes
                    .get(node_id)
                    .and_then(|node| node.get_value())
                    .ok_or(CircuitError::NodeValueNotFound(*node_id))
            })
            .collect()
    }
}

#[derive(Error, Debug)]
pub enum CircuitError {
    #[error("Invalid input size")]
    InvalidInputSize,
    #[error("Node {0} not found")]
    NodeNotFound(u32),
    #[error("Node value for node {0} not found")]
    NodeValueNotFound(u32),
    #[error("Node {0} has no value assigned")]
    NodeValueNotAssigned(u32),
    #[error("Output node {0} already has a value assigned")]
    OutputNodePreAssigned(u32),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_node() {
        let node = Node::new();
        assert!(node.get_value().is_none());
    }

    #[test]
    fn test_set_node_value() {
        let mut node = Node::new();
        node.set_value(10);
        assert_eq!(node.get_value(), Some(10));
    }

    #[test]
    fn test_create_gate() {
        let gate = Gate::new(Operation::Add, 1, 2, 3);
        assert!(matches!(gate.operation, Operation::Add));
        assert_eq!(gate.left_input, 1);
        assert_eq!(gate.right_input, 2);
        assert_eq!(gate.output, 3);
    }

    #[test]
    fn test_create_circuit() {
        let circuit = Circuit::new();
        assert!(circuit.nodes.is_empty());
        assert!(circuit.gates.is_empty());
    }

    #[test]
    fn test_add_node_to_circuit() {
        let mut circuit = Circuit::new();
        let node = Node::new();
        circuit.add_node(1, node);
        assert!(circuit.nodes.contains_key(&1));
    }

    #[test]
    fn test_add_gate_to_circuit() {
        let mut circuit = Circuit::new();
        let gate = Gate::new(Operation::Multiply, 1, 2, 3);
        circuit.add_gate(gate);
        assert_eq!(circuit.gates.len(), 1);
    }

    #[test]
    fn test_circuit_execute_simple_addition() {
        let mut circuit = Circuit::new();
        circuit.add_node(1, Node::new());
        circuit.add_node(2, Node::new());
        circuit.add_node(3, Node::new());

        circuit.add_gate(Gate::new(Operation::Add, 1, 2, 3));

        let input_values = vec![5, 10];
        let output = circuit.execute(&input_values).unwrap();
        assert_eq!(output, vec![15]);
    }

    #[test]
    fn test_circuit_error_handling() {
        let mut circuit = Circuit::new();
        circuit.add_node(1, Node::new());
        circuit.add_node(2, Node::new());

        let result = circuit.execute(&[1]);
        assert!(matches!(result, Err(CircuitError::InvalidInputSize)));

        circuit.add_node(3, Node::new());
        circuit.add_gate(Gate::new(Operation::Add, 1, 4, 3));

        let result = circuit.execute(&[1, 2]);
        assert!(matches!(result, Err(CircuitError::NodeNotFound(4))));
    }
}
