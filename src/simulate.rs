use std::collections::HashMap;

use circom_2_arithc::arithmetic_circuit::ArithmeticCircuit;

use crate::number::{Number, NumberError};

pub fn simulate<N: Number>(
    circuit: &ArithmeticCircuit,
    inputs: &HashMap<String, N>,
) -> Result<HashMap<String, N>, NumberError> {
    let mut wires = vec![N::zero(); circuit.wire_count as usize];

    for (name, wire_id) in &circuit.info.input_name_to_wire_index {
        wires[*wire_id as usize] = inputs
            .get(name)
            .ok_or(NumberError::MissingInput(name.clone()))?
            .clone();
    }

    for (_, constant) in &circuit.info.constants {
        wires[constant.wire_index as usize] = N::from_str(&constant.value.clone())?;
    }

    for gate in &circuit.gates {
        let lh = &wires[gate.lh_in as usize];
        let rh = &wires[gate.rh_in as usize];

        wires[gate.out as usize] = lh.infix_op(gate.op, rh)?;
    }

    let mut outputs = HashMap::new();

    for (name, wire_id) in &circuit.info.output_name_to_wire_index {
        outputs.insert(name.clone(), wires[*wire_id as usize].clone());
    }

    Ok(outputs)
}
