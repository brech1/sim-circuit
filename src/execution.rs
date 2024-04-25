//! # Execution Module
//!
//! Execution module.

use crate::circuit::Operation;

/// Process an operation given two input values.
pub fn process_operation(operation: Operation, left_input: u32, right_input: u32) -> u32 {
    match operation {
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
