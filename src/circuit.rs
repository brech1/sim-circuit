use crate::model::{Component, Executable, Memory};
use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    mem::take,
};
use thiserror::Error;

/// Generic memory for circuit simulation.
#[derive(Debug)]
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
#[derive(Debug)]
pub struct CircuitBuilder<T, U>
where
    T: Component + Executable<U, CircuitMemory<U>>,
    U: Copy,
{
    components: Vec<T>,
    index_map: HashMap<usize, usize>,
    component_inputs: HashSet<usize>,
    component_outputs: HashSet<usize>,
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
            index_map: HashMap::new(),
            component_inputs: HashSet::new(),
            component_outputs: HashSet::new(),
            next_index: 0,
            _phantom: PhantomData,
        }
    }

    /// Adds a component to the builder.
    pub fn add_component(&mut self, component: T) -> Result<&mut Self, CircuitBuilderError> {
        if component.inputs().is_empty() || component.outputs().is_empty() {
            return Err(CircuitBuilderError::DisconnectedComponent);
        }

        for &input in component.inputs() {
            // Verify that the input is connected to an existing output.
            if !self.component_outputs.contains(&input) {
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
#[derive(Debug)]
pub struct GenericCircuit<T, U>
where
    T: Component + Executable<U, CircuitMemory<U>>,
    U: Copy,
{
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

pub struct GenericCircuitExecutor<T, U>
where
    T: Component + Executable<U, CircuitMemory<U>>,
    U: Copy,
{
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

    /// Runs the circuit using the provided input values.
    pub fn run(&mut self, inputs: &[U]) -> Result<(), CircuitExecutionError> {
        // Check if the input values matches the circuit inputs
        if inputs.len() != self.circuit.inputs().len() {
            return Err(CircuitExecutionError::InputLengthMismatch);
        }

        // Set inputs in the memory
        for (input_index, &value) in self.circuit.inputs().iter().zip(inputs.iter()) {
            self.memory
                .write(*input_index, value)
                .map_err(CircuitExecutionError::MemoryError)?;
        }

        // Execute circuit
        self.circuit.execute(&mut self.memory)?;

        // Check if all outputs are defined
        for &output_index in self.circuit.outputs() {
            if self.memory.read(output_index).is_err() {
                return Err(CircuitExecutionError::UndefinedOutput(output_index));
            }
        }

        Ok(())
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
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CircuitExecutionError {
    #[error("Component execution Error")]
    ComponentExecutionError,
    #[error("Input length mismatch")]
    InputLengthMismatch,
    #[error("Circuit memory error")]
    MemoryError(#[from] CircuitMemoryError),
    #[error("Output at index {0} is undefined after circuit execution")]
    UndefinedOutput(usize),
}
