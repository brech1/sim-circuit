//! # Circuit Module
//!
//! Circuit module.

use std::collections::{HashMap, HashSet, VecDeque};
use thiserror::Error;

#[derive(Debug, Default)]
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

#[derive(Debug)]
/// Gate operations.
pub enum Operation {
    // Arithmetic Operations
    Add,
    Subtract,
    Multiply,
    Divide,
    IntegerDivide,
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

#[derive(Debug)]
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
            Operation::IntegerDivide => left_input / right_input, // Will panic if right_input is zero
            Operation::Exponentiate => left_input.pow(right_input),
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

#[derive(Debug, Default)]
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
    pub fn add_node(&mut self, id: u32, node: Node) -> Result<(), CircuitError> {
        if self.nodes.contains_key(&id) {
            return Err(CircuitError::NodeAlreadyExists(id));
        }

        self.nodes.insert(id, node);
        Ok(())
    }

    /// Add a gate to the circuit.
    pub fn add_gate(&mut self, gate: Gate) -> Result<(), CircuitError> {
        self.ensure_node_exists(gate.left_input)?;
        self.ensure_node_exists(gate.right_input)?;
        self.ensure_node_exists(gate.output)?;

        self.gates.push(gate);
        Ok(())
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

    // Execute the circuit with the given input values and return the output values without modifying the circuit.
    pub fn execute(&self, inputs: &[u32]) -> Result<Vec<u32>, CircuitError> {
        let (input_nodes, output_nodes) = self.get_circuit_io();

        // Validate input size
        if inputs.len() != input_nodes.len() {
            return Err(CircuitError::InvalidInputSize { received: inputs.len() as u32, need: input_nodes.len() as u32 });
        }

        let mut local_node_values = HashMap::new();

        // Tracks the number of dependencies each node has.
        let mut in_degree = HashMap::new();
        // Maps each input node to the set of output nodes that directly depend on its value.
        let mut dependency_graph = HashMap::new();
        // Queue of nodes with no dependencies, ready to be processed.
        let mut zero_in_degree = VecDeque::new();

        // Initialize input nodes with input values
        for (&node_id, &value) in input_nodes.iter().zip(inputs) {
            local_node_values.insert(node_id, value);
        }

        // Build dependency graph and in-degree map
        for gate in &self.gates {
            let output = gate.output;

            for &input in &[gate.left_input, gate.right_input] {
                dependency_graph
                    .entry(input)
                    .or_insert_with(Vec::new)
                    .push(output);
                *in_degree.entry(output).or_insert(0) += 1;
            }
        }

        // Add nodes with zero in-degree
        for node in self.nodes.keys() {
            if in_degree.get(node).is_none() {
                zero_in_degree.push_back(*node);
            }
        }

        // Implement Kahn's Algorithm
        let mut sorted_nodes = Vec::new();
        while let Some(node) = zero_in_degree.pop_front() {
            sorted_nodes.push(node);
            if let Some(dependents) = dependency_graph.remove(&node) {
                for dependent in dependents {
                    let degree = in_degree.get_mut(&dependent).unwrap();
                    *degree -= 1;
                    if *degree == 0 {
                        zero_in_degree.push_back(dependent);
                    }
                }
            }
        }

        // Check for unprocessed nodes
        if sorted_nodes.len() != self.nodes.len() {
            return Err(CircuitError::CyclicDependencyDetected);
        }

        // Execute gates in topological order
        for gate in &self.gates {
            let left_value = *local_node_values
                .get(&gate.left_input)
                .ok_or(CircuitError::NodeValueNotAssigned(gate.left_input))?;
            let right_value = *local_node_values
                .get(&gate.right_input)
                .ok_or(CircuitError::NodeValueNotAssigned(gate.right_input))?;
            let result = gate.execute(left_value, right_value);

            // Check if the output node already has a value assigned
            if local_node_values.contains_key(&gate.output) {
                return Err(CircuitError::OutputNodePreAssigned(gate.output));
            }

            local_node_values.insert(gate.output, result);
        }

        // Collect output values
        output_nodes
            .iter()
            .map(|&node_id| {
                local_node_values
                    .get(&node_id)
                    .copied()
                    .ok_or(CircuitError::NodeValueNotFound(node_id))
            })
            .collect()
    }

    /// Helper to check node existence.
    fn ensure_node_exists(&self, node_id: u32) -> Result<(), CircuitError> {
        if self.nodes.contains_key(&node_id) {
            Ok(())
        } else {
            Err(CircuitError::NodeNotFound(node_id))
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum CircuitError {
    #[error("Cyclic dependency detected")]
    CyclicDependencyDetected,
    #[error("Invalid input size")]
    InvalidInputSize { received: u32, need: u32 },
    #[error("Node {0} already exists")]
    NodeAlreadyExists(u32),
    #[error("Node {0} not found")]
    NodeNotFound(u32),
    #[error("Node {0} has no value assigned")]
    NodeValueNotAssigned(u32),
    #[error("Node value for node {0} not found")]
    NodeValueNotFound(u32),
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
    fn test_add_node() {
        let mut circuit = Circuit::new();
        let node = Node::new();

        circuit.add_node(1, node).unwrap();

        assert!(circuit.nodes.contains_key(&1));
    }

    #[test]
    fn test_add_node_error() {
        let mut circuit = Circuit::new();
        let node = Node::new();

        circuit.add_node(1, node).unwrap();

        let err = circuit.add_node(1, Node::new());
        assert_eq!(err, Err(CircuitError::NodeAlreadyExists(1)));
    }

    #[test]
    fn test_add_gate() {
        let mut circuit = Circuit::new();

        circuit.add_node(1, Node::new()).unwrap();
        circuit.add_node(2, Node::new()).unwrap();
        circuit.add_node(3, Node::new()).unwrap();

        let result = circuit.add_gate(Gate::new(Operation::Add, 1, 2, 3));
        assert!(result.is_ok());
        assert_eq!(circuit.gates.len(), 1);
    }

    #[test]
    fn test_add_gate_error() {
        let mut circuit = Circuit::new();

        circuit.add_node(1, Node::new()).unwrap();
        circuit.add_node(3, Node::new()).unwrap();

        let err = circuit.add_gate(Gate::new(Operation::Add, 1, 2, 3));
        assert_eq!(err, Err(CircuitError::NodeNotFound(2)));
    }

    #[test]
    fn test_execute() {
        let mut circuit = Circuit::new();

        circuit.add_node(1, Node::new()).unwrap();
        circuit.add_node(2, Node::new()).unwrap();
        circuit.add_node(3, Node::new()).unwrap();

        circuit
            .add_gate(Gate::new(Operation::Add, 1, 2, 3))
            .unwrap();

        let input_values = vec![5, 10];
        let output = circuit.execute(&input_values).unwrap();
        assert_eq!(output, vec![15]);
    }

    #[test]
    fn test_execute_cross_dep() {
        let mut circuit = Circuit::new();

        circuit.add_node(1, Node::new()).unwrap();
        circuit.add_node(2, Node::new()).unwrap();
        circuit.add_node(3, Node::new()).unwrap();
        circuit.add_node(4, Node::new()).unwrap();
        circuit.add_node(5, Node::new()).unwrap();

        circuit
            .add_gate(Gate::new(Operation::Add, 1, 2, 3))
            .unwrap();
        circuit
            .add_gate(Gate::new(Operation::Multiply, 3, 2, 4))
            .unwrap();
        circuit
            .add_gate(Gate::new(Operation::Subtract, 4, 1, 5))
            .unwrap();

        // ((3 + 2) * 2 ) - 3 = 7

        let input_values = vec![3, 2];
        let output = circuit.execute(&input_values).unwrap();
        assert_eq!(output, vec![7]);
    }

    #[test]
    fn test_execute_error_invalid_input_size() {
        let mut circuit = Circuit::new();

        circuit.add_node(1, Node::new()).unwrap();
        circuit.add_node(2, Node::new()).unwrap();

        let err = circuit.execute(&[1]);
        assert_eq!(err, Err(CircuitError::InvalidInputSize { received: 1, need: 0 }));
    }

    #[test]
    fn test_execute_error_cyclic_dependency() {
        let mut circuit = Circuit::new();

        circuit.add_node(1, Node::new()).unwrap();
        circuit.add_node(2, Node::new()).unwrap();
        circuit.add_node(3, Node::new()).unwrap();
        circuit.add_node(4, Node::new()).unwrap();
        circuit.add_node(5, Node::new()).unwrap();

        // Valid computation
        circuit
            .add_gate(Gate::new(Operation::Add, 1, 2, 3))
            .unwrap();

        // Cyclic dependency
        circuit
            .add_gate(Gate::new(Operation::Multiply, 3, 4, 5))
            .unwrap();
        circuit
            .add_gate(Gate::new(Operation::Add, 5, 1, 4))
            .unwrap();

        let input_values = vec![1, 2];
        let err = circuit.execute(&input_values);
        assert_eq!(err, Err(CircuitError::CyclicDependencyDetected));
    }
}
