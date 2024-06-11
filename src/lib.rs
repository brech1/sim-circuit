//! # Sim Circuit
//!
//! Utilities for simulating logic gate circuits.

mod number;
mod number_u32;
mod simulate;

pub use number::{Number, NumberError};
pub use number_u32::NumberU32;
pub use simulate::simulate;
