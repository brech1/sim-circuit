use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    str::FromStr,
};

use serde::{Deserialize, Serialize};
use strum_macros::{Display as StrumDisplay, EnumString};

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArithmeticCircuit {
    pub wire_count: u32,
    pub info: CircuitInfo,
    pub gates: Vec<ArithmeticGate>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CircuitInfo {
    pub input_name_to_wire_index: HashMap<String, u32>,
    pub constants: HashMap<String, ConstantInfo>,
    pub output_name_to_wire_index: HashMap<String, u32>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConstantInfo {
    pub value: String,
    pub wire_index: u32,
}

impl ArithmeticCircuit {
    pub fn from_info_and_bristol_string(
        info: CircuitInfo,
        input: &str,
    ) -> Result<ArithmeticCircuit, String> {
        ArithmeticCircuit::read_info_and_bristol(info, &mut BufReader::new(input.as_bytes()))
    }

    pub fn read_info_and_bristol<R: BufRead>(
        info: CircuitInfo,
        r: &mut R,
    ) -> Result<ArithmeticCircuit, String> {
        let (gate_count, wire_count) = BristolLine::read(r)?.circuit_sizes()?;

        let input_count = BristolLine::read(r)?.io_count()?;
        if input_count != info.input_name_to_wire_index.len() as u32 {
            return Err("Input count mismatch".into());
        }

        let output_count = BristolLine::read(r)?.io_count()?;
        if output_count != info.output_name_to_wire_index.len() as u32 {
            return Err("Output count mismatch".into());
        }

        let mut gates = Vec::new();
        for _ in 0..gate_count {
            gates.push(BristolLine::read(r)?.gate()?);
        }

        for line in r.lines() {
            if !line.map_err(|e| e.to_string())?.trim().is_empty() {
                return Err("Unexpected non-whitespace line after gates".into());
            }
        }

        Ok(ArithmeticCircuit {
            wire_count,
            info,
            gates,
        })
    }
}

struct BristolLine(Vec<String>);

impl BristolLine {
    pub fn read(r: &mut impl BufRead) -> Result<Self, String> {
        loop {
            let mut line = String::new();
            r.read_line(&mut line).map_err(|e| e.to_string())?;

            let line = line.trim();

            if line.is_empty() {
                continue;
            }

            return Ok(BristolLine(
                line.split_whitespace()
                    .map(|part| part.to_string())
                    .collect(),
            ));
        }
    }

    pub fn circuit_sizes(&self) -> Result<(u32, u32), String> {
        Ok((self.get(0)?, self.get(1)?))
    }

    pub fn io_count(&self) -> Result<u32, String> {
        let count = self.get::<u32>(0)?;

        if self.0.len() != (count + 1) as usize {
            return Err(format!("Expected {} parts", count + 1));
        }

        for i in 1..self.0.len() {
            if self.get_str(i)? != "1" {
                return Err(format!("Expected 1 at index {}", i));
            }
        }

        Ok(count)
    }

    pub fn gate(&self) -> Result<ArithmeticGate, String> {
        if self.0.len() != 6 {
            return Err("Expected 6 parts".into());
        }

        if self.get::<u32>(0)? != 2 || self.get::<u32>(1)? != 1 {
            return Err("Expected 2 inputs and 1 output".into());
        }

        Ok(ArithmeticGate {
            lh_in: self.get(2)?,
            rh_in: self.get(3)?,
            out: self.get(4)?,
            op: self.get(5)?,
        })
    }

    fn get<T: FromStr>(&self, index: usize) -> Result<T, String> {
        self.0
            .get(index)
            .ok_or(format!("Index {} out of bounds", index))?
            .parse::<T>()
            .map_err(|_| format!("Failed to convert at index {}", index))
    }

    fn get_str(&self, index: usize) -> Result<&str, String> {
        self.0
            .get(index)
            .ok_or(format!("Index {} out of bounds", index))
            .map(|s| s.as_str())
    }
}

/// Represents a circuit gate, with a left-hand input, right-hand input, and output node identifiers.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArithmeticGate {
    pub op: AGateType,
    pub lh_in: u32,
    pub rh_in: u32,
    pub out: u32,
}

/// Types of gates that can be used in an arithmetic circuit.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, EnumString, StrumDisplay)]
pub enum AGateType {
    AAdd,
    ADiv,
    AEq,
    AGEq,
    AGt,
    ALEq,
    ALt,
    AMul,
    ANeq,
    ASub,
    AXor,
    APow,
    AIntDiv,
    AMod,
    AShiftL,
    AShiftR,
    ABoolOr,
    ABoolAnd,
    ABitOr,
    ABitAnd,
}
