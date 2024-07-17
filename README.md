# Sim Circuit

[![MIT licensed][mit-badge]][mit-url]
[![Build Status][actions-badge]][actions-url]
[![codecov][codecov-badge]][codecov-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/brech1/sim-circuit/blob/master/LICENSE
[actions-badge]: https://github.com/brech1/sim-circuit/actions/workflows/build.yml/badge.svg
[actions-url]: https://github.com/brech1/sim-circuit/actions?query=branch%3Amaster
[codecov-badge]: https://codecov.io/github/brech1/sim-circuit/graph/badge.svg
[codecov-url]: https://app.codecov.io/github/brech1/sim-circuit/

Sim Circuit is a Rust crate that provides a generic circuit model along with predefined structures to build circuits with custom execution over custom values.

## Model

The model views a circuit as a set of components, each with a custom execution, that operate over a linear set of slots defined as a memory.

### Memory Trait

Defines a memory structure with read and write operations.

```rust
pub trait Memory<T> {
    type Error;

    /// Reads a value from the memory at the specified index.
    fn read(&self, index: usize) -> Result<T, Self::Error>;

    /// Writes a value to the memory at the specified index.
    fn write(&mut self, index: usize, value: T) -> Result<(), Self::Error>;
}
```

### Component Trait

Defines a component structure with input and output nodes.

```rust
pub trait Component {
    /// Returns the indices of the input nodes.
    fn inputs(&self) -> &[usize];

    /// Returns the indices of the output nodes.
    fn outputs(&self) -> &[usize];
}
```

### Executable Trait

Defines the execution of a component over a custom memory.

```rust
pub trait Executable<T, U: Memory<T>>: Component {
    type Error;

    /// Executes the component using the provided memory.
    fn execute(&self, memory: &mut U) -> Result<(), Self::Error>;
}
```

## Predefined Structures

These are the provided structures to help build circuits with the model.

In the `CircuitMemory`, the component type `T` is the value type. In the rest of the components, `T` is the component type `T` and `U` the memory value type.

### Circuit Memory

A storage structure to keep track of the values of all nodes in the circuit during execution.

```rust
pub struct CircuitMemory<T> {
  wires: Vec<Option<T>>,
}
```

### Circuit Builder

Used to create and configure a circuit by adding components and specifying input nodes.

```rust
pub struct CircuitBuilder<T, U> {
  components: Vec<T>,
  circuit_inputs: Vec<usize>,
  component_inputs: HashSet<usize>,
  component_outputs: HashSet<usize>,
  index_map: HashMap<usize, usize>,
  next_index: usize,
  _phantom: PhantomData<U>,
}
```

### Generic Circuit

Represents the entire circuit with a defined execution order. It also implements the `Component` trait, allowing it to be part of another circuit.

```rust
pub struct GenericCircuit<T, U> {
  components: Vec<T>,
  inputs: Vec<usize>,
  outputs: Vec<usize>,
  memory_map: HashMap<usize, usize>,
  _phantom: PhantomData<U>,
}
```

### Circuit Executor

Used to execute the circuit over a memory with specific input values.

```rust
pub struct GenericCircuitExecutor<T, U> {
    circuit: GenericCircuit<T, U>,
    memory: CircuitMemory<U>,
}
```

## Implementation

Let's see how to use the provided structures.

### Define Custom Components

First, you need to define your custom components by implementing the `Component` and `Executable` traits.

```rust
pub enum BinaryOperation {
    AND,
    XOR,
}

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
}

impl Executable<bool, CircuitMemory<bool>> for BinaryGate {
  type Error = ();

  fn execute(
    &self,
    memory: &mut CircuitMemory<bool>,
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
```

### Create a Circuit Builder

Use `CircuitBuilder` to create and configure your circuit by adding components and specifying input nodes.

```rust
let mut builder = CircuitBuilder::<BinaryGate, bool>::new();
builder.add_inputs(&[0, 1]);
```

### Add Components

Add your custom components to the circuit builder.

```rust
let and_gate = BinaryGate {
  op: BinaryOperation::AND,
  inputs: vec![0, 1],
  outputs: vec![2],
};

builder.add_component(and_gate).unwrap();
```

### Build The Circuit

Build the circuit using the circuit builder.

```rust
let circuit = builder.build().unwrap();
```

### Create a Circuit Executor

Use `GenericCircuitExecutor` to execute the circuit over a memory with specific input values.

```rust
let mut executor = GenericCircuitExecutor::new(circuit);
```

### Execute The Circuit

Execute the circuit with specific input values.

```rust
let input_values = HashMap::from([(0, true), (1, false)]);
let output = executor.run(&input_values).unwrap();
```
