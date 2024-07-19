use sim_circuit::{
    circuit::{CircuitBuilder, CircuitBuilderError, CircuitMemory, GenericCircuit},
    model::{Component, Executable, Memory},
};
use std::fmt::Debug;

#[derive(PartialEq, Eq, Clone)]
pub enum BinaryOperation {
    AND,
    XOR,
    OR,
    NOT,
    NAND,
}

impl Debug for BinaryOperation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOperation::AND => write!(f, "AND"),
            BinaryOperation::XOR => write!(f, "XOR"),
            BinaryOperation::OR => write!(f, "OR"),
            BinaryOperation::NOT => write!(f, "NOT"),
            BinaryOperation::NAND => write!(f, "NAND"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BinaryGate {
    op: BinaryOperation,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
}

impl Component for BinaryGate {
    fn inputs(&self) -> &[usize] {
        &self.inputs
    }

    fn outputs(&self) -> &[usize] {
        &self.outputs
    }

    fn set_inputs(&mut self, inputs: Vec<usize>) {
        self.inputs = inputs;
    }

    fn set_outputs(&mut self, outputs: Vec<usize>) {
        self.outputs = outputs;
    }
}

impl Executable<bool, CircuitMemory<bool>> for BinaryGate {
    type Error = ();

    fn execute(&self, memory: &mut CircuitMemory<bool>) -> Result<(), Self::Error> {
        let a = memory.read(self.inputs[0]).unwrap();
        let b = memory.read(self.inputs[1]).unwrap_or(false);

        let result = match self.op {
            BinaryOperation::AND => a && b,
            BinaryOperation::XOR => a ^ b,
            BinaryOperation::OR => a || b,
            BinaryOperation::NAND => !(a && b),
            BinaryOperation::NOT => !a,
        };

        memory.write(self.outputs[0], result).unwrap();

        Ok(())
    }
}

// Full Adder Circuit
pub type FullAdderCircuit = GenericCircuit<BinaryGate, bool>;

#[derive(Debug, PartialEq, Eq)]
pub struct FullAdder {
    circuit: FullAdderCircuit,
}

impl FullAdder {
    pub fn new() -> Result<Self, CircuitBuilderError> {
        let mut builder = CircuitBuilder::<BinaryGate, bool>::new();
        builder.add_inputs(&[0, 1, 2]); // Inputs: A, B, Carry-In

        // Gates for the full adder
        let sum_gate_1 = BinaryGate {
            op: BinaryOperation::XOR,
            inputs: vec![0, 1],
            outputs: vec![3], // Temp sum
        };

        let carry_gate_1 = BinaryGate {
            op: BinaryOperation::AND,
            inputs: vec![0, 1],
            outputs: vec![4], // Temp carry
        };

        let sum_gate_2 = BinaryGate {
            op: BinaryOperation::XOR,
            inputs: vec![3, 2], // Temp sum and Carry-In
            outputs: vec![5],   // Final sum
        };

        let carry_gate_2 = BinaryGate {
            op: BinaryOperation::AND,
            inputs: vec![3, 2],
            outputs: vec![6], // Temp carry from sum
        };

        let carry_gate_3 = BinaryGate {
            op: BinaryOperation::OR,
            inputs: vec![4, 6], // Temp carries
            outputs: vec![7],   // Final carry-out
        };

        builder.add_component(sum_gate_1)?;
        builder.add_component(carry_gate_1)?;
        builder.add_component(sum_gate_2)?;
        builder.add_component(carry_gate_2)?;
        builder.add_component(carry_gate_3)?;

        Ok(Self {
            circuit: builder.build()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_circuit::circuit::GenericCircuitExecutor;
    use std::collections::HashMap;

    #[test]
    fn test_full_adder() {
        let full_adder = FullAdder::new().unwrap();
        let mut executor = GenericCircuitExecutor::new(full_adder.circuit);

        let input_values = HashMap::from([(0, true), (1, true), (2, false)]);

        let output = executor.run(&input_values).unwrap();
        assert_eq!(output.get(&5), Some(&false)); // Sum = 0
        assert_eq!(output.get(&7), Some(&true)); // Carry = 1
    }
}
