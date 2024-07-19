//! # Model Module
//!
//! Contains the library model traits.

/// Memory trait.
pub trait Memory<T> {
    type Error;

    /// Reads a value from the memory at the specified index.
    fn read(&self, index: usize) -> Result<T, Self::Error>;

    /// Writes a value to the memory at the specified index.
    fn write(&mut self, index: usize, value: T) -> Result<(), Self::Error>;
}

/// Circuit component trait.
pub trait Component {
    /// Returns the indices of the input nodes.
    fn inputs(&self) -> &[usize];

    /// Returns the indices of the output nodes.
    fn outputs(&self) -> &[usize];

    /// Sets the input nodes.
    fn set_inputs(&mut self, inputs: Vec<usize>);

    /// Sets the output nodes.
    fn set_outputs(&mut self, outputs: Vec<usize>);
}

/// Executable component trait.
pub trait Executable<T, U: Memory<T>>: Component {
    type Error;

    /// Executes the component using the provided memory.
    fn execute(&self, memory: &mut U) -> Result<(), Self::Error>;
}
