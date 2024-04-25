//! # Circuit Module
//!
//! Circuit module.

use std::collections::HashMap;

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

    /// Check if the node has a value.
    pub fn has_value(&self) -> bool {
        self.value.is_some()
    }
}

/// Gate operations enum.
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
    operation: Operation,
    left_input: u32,
    right_input: u32,
    output: u32,
}

impl Gate {
    pub fn new(operation: Operation, left_input: u32, right_input: u32, output: u32) -> Gate {
        Gate {
            operation,
            left_input,
            right_input,
            output,
        }
    }
}

/// Generic circuit struct.
pub struct Circuit {
    /// Circuit nodes.
    nodes: HashMap<u32, Node>,
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

    /// Get a node from the circuit.
    pub fn get_node(&self, id: u32) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Add a gate to the circuit.
    pub fn add_gate(&mut self, gate: Gate) {
        self.gates.push(gate);
    }

    /// Get the gates from the circuit.
    pub fn get_gates(&self) -> &Vec<Gate> {
        &self.gates
    }
}
