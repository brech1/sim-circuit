use crate::model::{Component, Executable, Memory};
use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    mem::take,
};
use thiserror::Error;

/// Generic memory for circuit simulation.
#[derive(Debug, PartialEq, Eq)]
pub struct CircuitMemory<T> {
    wires: Vec<Option<T>>,
}

impl<T> CircuitMemory<T> {
    fn new(size: usize) -> Self {
        let mut wires = Vec::with_capacity(size);

        for _ in 0..size {
            wires.push(None);
        }

        Self { wires }
    }
}

impl<T> Memory<T> for CircuitMemory<T>
where
    T: Copy,
{
    type Error = CircuitMemoryError;

    /// Attempts to read a value from the specified memory index.
    /// Returns an error if the index is out of bounds or if no value has been written there yet.
    fn read(&self, index: usize) -> Result<T, Self::Error> {
        match self.wires.get(index) {
            Some(Some(value)) => Ok(*value),
            Some(None) => Err(CircuitMemoryError::UninitializedSlot(index)),
            None => Err(CircuitMemoryError::ReadError(index)),
        }
    }

    /// Writes a value to the specified memory index.
    /// Returns an error if the index is out of bounds or if the slot is already occupied.
    fn write(&mut self, index: usize, value: T) -> Result<(), Self::Error> {
        match self.wires.get_mut(index) {
            Some(slot) => {
                if slot.is_some() {
                    Err(CircuitMemoryError::RewriteAttempt(index))
                } else {
                    *slot = Some(value);
                    Ok(())
                }
            }
            None => Err(CircuitMemoryError::WriteError(index)),
        }
    }
}

/// Circuit builder.
#[derive(Debug, PartialEq, Eq)]
pub struct CircuitBuilder<T, U> {
    components: Vec<T>,
    circuit_inputs: Vec<usize>,
    component_inputs: HashSet<usize>,
    component_outputs: HashSet<usize>,
    index_map: HashMap<usize, usize>,
    next_index: usize,
    _phantom: PhantomData<U>,
}

impl<T, U> CircuitBuilder<T, U>
where
    T: Component + Executable<U, CircuitMemory<U>>,
    U: Copy,
{
    /// Creates a new circuit builder.
    pub fn new() -> Self {
        Self {
            components: Vec::new(),
            circuit_inputs: Vec::new(),
            index_map: HashMap::new(),
            component_inputs: HashSet::new(),
            component_outputs: HashSet::new(),
            next_index: 0,
            _phantom: PhantomData,
        }
    }

    /// Adds circuit inputs to the builder.
    pub fn add_inputs(&mut self, inputs: &[usize]) -> &mut Self {
        self.circuit_inputs.extend(inputs.iter().copied());
        self
    }

    /// Adds a component to the builder.
    pub fn add_component(&mut self, component: T) -> Result<&mut Self, CircuitBuilderError> {
        if component.inputs().is_empty() || component.outputs().is_empty() {
            return Err(CircuitBuilderError::DisconnectedComponent);
        }

        for &input in component.inputs() {
            // Verify that the input is connected to an existing component output or is a circuit input.
            if !self.component_outputs.contains(&input) && !self.circuit_inputs.contains(&input) {
                return Err(CircuitBuilderError::TopologicalOrderError(input));
            }

            self.component_inputs.insert(input);
            self.index_map.entry(input).or_insert_with(|| {
                let index = self.next_index;
                self.next_index += 1;
                index
            });
        }

        for &output in component.outputs() {
            // Verify that the output is not a circuit input.
            if self.circuit_inputs.contains(&output) {
                return Err(CircuitBuilderError::OutputIsACircuitInput(output));
            }

            self.component_outputs.insert(output);
            self.index_map.entry(output).or_insert_with(|| {
                let index = self.next_index;
                self.next_index += 1;
                index
            });
        }

        self.components.push(component);
        Ok(self)
    }

    /// Builds the circuit.
    pub fn build(&mut self) -> Result<GenericCircuit<T, U>, CircuitBuilderError> {
        if self.components.is_empty() {
            return Err(CircuitBuilderError::EmptyBuilder);
        }

        // Calculate circuit inputs and outputs
        let circuit_inputs = self
            .component_inputs
            .difference(&self.component_outputs)
            .copied()
            .collect::<Vec<usize>>();
        let circuit_outputs = self
            .component_outputs
            .difference(&self.component_inputs)
            .copied()
            .collect::<Vec<usize>>();

        Ok(GenericCircuit::new(
            take(&mut self.components),
            take(&mut self.index_map),
            circuit_inputs,
            circuit_outputs,
        ))
    }
}

/// Represents a generic circuit with a topological, linear execution order.
/// Utilizes a generic memory to store wire values and execute gates.
#[derive(Debug, PartialEq, Eq)]
pub struct GenericCircuit<T, U> {
    components: Vec<T>,
    inputs: Vec<usize>,
    outputs: Vec<usize>,
    memory_map: HashMap<usize, usize>,
    _phantom: PhantomData<U>,
}

impl<T, U> Component for GenericCircuit<T, U>
where
    T: Component + Executable<U, CircuitMemory<U>>,
    U: Copy,
{
    /// Returns the indices of the input nodes for the entire circuit.
    fn inputs(&self) -> &[usize] {
        &self.inputs
    }

    /// Returns the indices of the output nodes for the entire circuit.
    fn outputs(&self) -> &[usize] {
        &self.outputs
    }
}

impl<T, U> Executable<U, CircuitMemory<U>> for GenericCircuit<T, U>
where
    T: Component + Executable<U, CircuitMemory<U>>,
    U: Copy,
{
    type Error = CircuitExecutionError;

    fn execute(&self, memory: &mut CircuitMemory<U>) -> Result<(), CircuitExecutionError> {
        for component in &self.components {
            component
                .execute(memory)
                .map_err(|_| CircuitExecutionError::ComponentExecutionError)?;
        }

        Ok(())
    }
}

impl<T, U> GenericCircuit<T, U>
where
    T: Component + Executable<U, CircuitMemory<U>>,
    U: Copy,
{
    /// Creates a new generic circuit.
    pub fn new(
        components: Vec<T>,
        memory_map: HashMap<usize, usize>,
        inputs: Vec<usize>,
        outputs: Vec<usize>,
    ) -> Self {
        Self {
            components,
            memory_map,
            inputs,
            outputs,
            _phantom: PhantomData,
        }
    }

    /// Returns the memory size
    pub fn memory_size(&self) -> usize {
        self.memory_map
            .values()
            .max()
            .map_or(0, |&max_index| max_index + 1)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GenericCircuitExecutor<T, U> {
    circuit: GenericCircuit<T, U>,
    memory: CircuitMemory<U>,
}

impl<T, U> GenericCircuitExecutor<T, U>
where
    T: Component + Executable<U, CircuitMemory<U>>,
    U: Copy,
{
    /// Creates a new generic circuit executor.
    pub fn new(circuit: GenericCircuit<T, U>) -> Self {
        let memory_size = circuit.memory_size();

        Self {
            circuit,
            memory: CircuitMemory::new(memory_size),
        }
    }

    /// Runs the circuit using the provided input values and returns a map of the output values.
    pub fn run(
        &mut self,
        inputs: &HashMap<usize, U>,
    ) -> Result<HashMap<usize, U>, CircuitExecutionError> {
        // Check if the input values match the circuit inputs
        if inputs.len() != self.circuit.inputs().len() {
            return Err(CircuitExecutionError::InputLengthMismatch);
        }

        // Set inputs in  memory
        for &input_index in self.circuit.inputs() {
            if let Some(&value) = inputs.get(&input_index) {
                // Translate external input index to internal memory index using the memory_map
                if let Some(&internal_index) = self.circuit.memory_map.get(&input_index) {
                    self.memory
                        .write(internal_index, value)
                        .map_err(CircuitExecutionError::MemoryError)?;
                } else {
                    return Err(CircuitExecutionError::MemoryMappingError(input_index));
                }
            } else {
                return Err(CircuitExecutionError::InputNotFoundError(input_index));
            }
        }

        // Execute the circuit
        self.circuit.execute(&mut self.memory)?;

        // Retrieve and return output values
        let mut output_values = HashMap::new();
        for &output_index in self.circuit.outputs() {
            if let Some(&internal_index) = self.circuit.memory_map.get(&output_index) {
                match self.memory.read(internal_index) {
                    Ok(value) => {
                        output_values.insert(output_index, value);
                    }
                    Err(e) => {
                        return Err(CircuitExecutionError::MemoryError(e));
                    }
                }
            } else {
                return Err(CircuitExecutionError::UndefinedOutput(output_index));
            }
        }

        Ok(output_values)
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CircuitMemoryError {
    #[error("Error reading memory: Index {0} out of bounds")]
    ReadError(usize),
    #[error("Attempt to read uninitialized memory at index {0}")]
    UninitializedSlot(usize),
    #[error("Error writing memory: Index {0} out of bounds")]
    WriteError(usize),
    #[error("Attempt to rewrite an already initialized memory slot at index {0}")]
    RewriteAttempt(usize),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CircuitBuilderError {
    #[error("Builder is empty")]
    EmptyBuilder,
    #[error("Disconnected component")]
    DisconnectedComponent,
    #[error("Component input {0} not connected to any existing component outputs")]
    TopologicalOrderError(usize),
    #[error("Output {0} is defined as a circuit input")]
    OutputIsACircuitInput(usize),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CircuitExecutionError {
    #[error("Component execution error")]
    ComponentExecutionError,
    #[error("Input {0} not defined")]
    InputNotFoundError(usize),
    #[error("Input length mismatch")]
    InputLengthMismatch,
    #[error("Input {0} not found in memory as a circuit input")]
    MemoryMappingError(usize),
    #[error("Circuit memory error: {0}")]
    MemoryError(#[from] CircuitMemoryError),
    #[error("Output at index {0} is undefined after circuit execution")]
    UndefinedOutput(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Debug;

    #[derive(PartialEq, Eq, Clone)]
    pub enum BinaryOperation {
        AND,
        XOR,
    }

    impl Debug for BinaryOperation {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                BinaryOperation::AND => write!(f, "AND"),
                BinaryOperation::XOR => write!(f, "XOR"),
            }
        }
    }

    #[derive(Debug, PartialEq, Eq, Clone)]
    pub struct BinaryGate {
        op: BinaryOperation,
        inputs: Vec<usize>,
        outputs: Vec<usize>,
    }

    impl crate::model::Component for BinaryGate {
        fn inputs(&self) -> &[usize] {
            &self.inputs
        }

        fn outputs(&self) -> &[usize] {
            &self.outputs
        }
    }

    impl Executable<bool, CircuitMemory<bool>> for BinaryGate {
        type Error = ();

        fn execute(
            &self,
            memory: &mut crate::circuit::CircuitMemory<bool>,
        ) -> Result<(), Self::Error> {
            let a = memory.read(self.inputs[0]).unwrap();
            let b = memory.read(self.inputs[1]).unwrap();

            let result = match self.op {
                BinaryOperation::AND => a && b,
                BinaryOperation::XOR => a ^ b,
            };
            memory.write(self.outputs[0], result).unwrap();
            Ok(())
        }
    }

    #[test]
    fn test_memory_operations() {
        let mut memory: CircuitMemory<bool> = CircuitMemory::new(10);

        // Test read error on uninitialized slot
        assert_eq!(
            memory.read(1),
            Err(CircuitMemoryError::UninitializedSlot(1))
        );

        // Test write and read
        assert_eq!(memory.write(2, true), Ok(()));
        assert_eq!(memory.read(2), Ok(true));

        // Test write error on out of bounds
        assert_eq!(
            memory.write(11, true),
            Err(CircuitMemoryError::WriteError(11))
        );

        // Test read error on out of bounds
        assert_eq!(memory.read(11), Err(CircuitMemoryError::ReadError(11)));

        // Test rewrite
        assert_eq!(
            memory.write(2, false),
            Err(CircuitMemoryError::RewriteAttempt(2))
        );
    }

    #[test]
    fn test_component_execution() {
        let mut memory: CircuitMemory<bool> = CircuitMemory::new(5);

        // Test AND gate
        let and_gate = BinaryGate {
            op: BinaryOperation::AND,
            inputs: vec![0, 1],
            outputs: vec![2],
        };

        // Initialize inputs
        memory.write(0, true).unwrap();
        memory.write(1, false).unwrap();

        // Execute AND gate
        assert_eq!(and_gate.execute(&mut memory), Ok(()));
        assert_eq!(memory.read(2), Ok(false));

        // Test XOR gate
        let xor_gate = BinaryGate {
            op: BinaryOperation::XOR,
            inputs: vec![2, 3],
            outputs: vec![4],
        };

        // Initialize inputs
        memory.write(3, true).unwrap();

        // Execute XOR gate
        assert_eq!(xor_gate.execute(&mut memory), Ok(()));
        assert_eq!(memory.read(4), Ok(true));
    }

    #[test]
    fn test_builder_disconnected_component() {
        let mut builder = CircuitBuilder::<BinaryGate, bool>::new();
        let gate = BinaryGate {
            op: BinaryOperation::AND,
            inputs: vec![],
            outputs: vec![],
        };

        assert_eq!(
            builder.add_component(gate),
            Err(CircuitBuilderError::DisconnectedComponent)
        );
    }

    #[test]
    fn test_builder_topological_order() {
        let mut builder = CircuitBuilder::<BinaryGate, bool>::new();
        let gate = BinaryGate {
            op: BinaryOperation::AND,
            inputs: vec![0],
            outputs: vec![1],
        };

        assert_eq!(
            builder.add_component(gate.clone()),
            Err(CircuitBuilderError::TopologicalOrderError(0))
        );

        builder.add_inputs(&[0]);

        assert!(builder.add_component(gate).is_ok());

        let second_gate = BinaryGate {
            op: BinaryOperation::AND,
            inputs: vec![1],
            outputs: vec![2],
        };

        assert!(builder.add_component(second_gate).is_ok());
    }

    #[test]
    fn test_builder_bad_component_output() {
        let mut builder = CircuitBuilder::<BinaryGate, bool>::new();
        builder.add_inputs(&[0, 1]);

        let gate = BinaryGate {
            op: BinaryOperation::XOR,
            inputs: vec![0],
            outputs: vec![1],
        };

        assert_eq!(
            builder.add_component(gate),
            Err(CircuitBuilderError::OutputIsACircuitInput(1))
        );
    }

    #[test]
    fn test_builder_empty() {
        let mut builder = CircuitBuilder::<BinaryGate, bool>::new();
        assert_eq!(builder.build(), Err(CircuitBuilderError::EmptyBuilder));
    }

    #[test]
    fn test_circuit_builder_and_execution() {
        let mut builder = CircuitBuilder::<BinaryGate, bool>::new();

        // Add circuit inputs
        builder.add_inputs(&[0, 1, 3]);

        // Creating and adding gates
        let and_gate = BinaryGate {
            op: BinaryOperation::AND,
            inputs: vec![0, 1],
            outputs: vec![2],
        };

        let xor_gate = BinaryGate {
            op: BinaryOperation::XOR,
            inputs: vec![2, 3],
            outputs: vec![4],
        };

        builder.add_component(and_gate).unwrap();

        assert!(builder.add_component(xor_gate).is_ok());

        let circuit = builder.build().unwrap();
        let mut executor = GenericCircuitExecutor::new(circuit);

        // Setup inputs and run the circuit
        let input_values = HashMap::from([(0, true), (1, false), (3, true)]);

        let output = executor.run(&input_values).unwrap();

        assert_eq!(output.get(&4), Some(&true));
    }
}
