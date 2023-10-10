//! An example of a basic circuit function which wraps an existing circuit and makes it compatible
//! with a standard for serializing and deserializing inputs and outputs.
//!
//! To build the binary:
//!
//!     `cargo build --release --bin circuit_function_field`
//!
//! To build the circuit:
//!
//!     `./target/release/circuit_function_field build`
//!
//! To prove the circuit using evm io:
//!
//!    `./target/release/circuit_function_evm prove --input-json src/bin/circuit_function_evm_input.json`
//!
//! Note that this circuit will not work with field-based io.

use plonky2x::backend::circuit::{Circuit, PlonkParameters};
use plonky2x::backend::function::VerifiableFunction;
use plonky2x::frontend::hint::simple::hint::Hint;
use plonky2x::frontend::vars::ByteVariable;
use plonky2x::prelude::{
    ArrayVariable, BoolVariable, CircuitBuilder, Field, ValueStream, Variable, VariableStream,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct MaskHint {}

impl<L: PlonkParameters<D>, const D: usize> Hint<L, D> for MaskHint {
    fn hint(&self, input_stream: &mut ValueStream<L, D>, output_stream: &mut ValueStream<L, D>) {
        let arr = input_stream.read_value::<ArrayVariable<Variable, N>>();
        let mask = input_stream.read_value::<ArrayVariable<BoolVariable, N>>();

        let mut out = vec![L::Field::ZERO; N];

        let mut nxt: usize = 0;

        for i in 0..N {
            if mask[i] {
                out[nxt] = arr[i];
                nxt += 1;
            }
        }
        output_stream.write_value::<ArrayVariable<Variable, N>>(out);
    }
}

#[derive(Debug, Clone)]
struct SimpleAdditionCircuit;

const N: usize = 3;

impl Circuit for SimpleAdditionCircuit {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        // x \in [0, 2**64-2**32+1)
        let arr = builder.read::<ArrayVariable<Variable, N>>();
        let mask = builder.read::<ArrayVariable<BoolVariable, N>>();

        let mut input_stream = VariableStream::new();
        input_stream.write(&arr);
        input_stream.write(&mask);

        let hint = MaskHint {};
        let output_stream = builder.hint(input_stream, hint);
        let out = output_stream.read::<ArrayVariable<Variable, N>>(builder);

        let r = builder.constant::<Variable>(L::Field::from_canonical_u64(7));
        let mut acc_f = builder.zero::<Variable>();
        let mut acc_g = builder.zero::<Variable>();
        let mut pow_x_f = builder.one::<Variable>();
        let mut pow_x_g = builder.one::<Variable>();
        for i in 0..N {
            let tmp1 = builder.mul(out[i], pow_x_g);
            acc_g = builder.add(acc_g, tmp1);
            pow_x_g = builder.mul(pow_x_g, r);

            let zero = builder.zero::<Variable>();
            let tmp2 = builder.select(mask[i], arr[i], zero);
            let tmp3 = builder.mul(tmp2, pow_x_f);
            acc_f = builder.add(acc_f, tmp3);
            let one = builder.one::<Variable>();
            let tmp4 = builder.select(mask[i], r, one);
            pow_x_f = builder.mul(pow_x_f, tmp4);
        }
        builder.assert_is_equal(acc_f, acc_g);
        builder.write(out);
    }
}

fn main() {
    VerifiableFunction::<SimpleAdditionCircuit>::entrypoint();
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use plonky2x::prelude::{DefaultParameters, GoldilocksField, PoseidonGoldilocksConfig};

    use super::*;

    type F = GoldilocksField;
    type L = DefaultParameters;
    const D: usize = 2;

    #[test]
    fn test_circuit_function_evm() {
        let mut builder = CircuitBuilder::<L, D>::new();
        SimpleAdditionCircuit::define(&mut builder);
        let circuit = builder.build();
        let mut input = circuit.input();
        input.write::<ArrayVariable<Variable, N>>(vec![
            F::from_canonical_u64(1),
            F::from_canonical_u64(2),
            F::from_canonical_u64(3),
        ]);
        input.write::<ArrayVariable<BoolVariable, N>>(vec![false, true, true]);

        let (proof, mut output) = circuit.prove(&input);
        circuit.verify(&proof, &input, &output);

        let out = output.read::<ArrayVariable<Variable, N>>();

        assert_eq!(
            out,
            vec![
                F::from_canonical_u64(2),
                F::from_canonical_u64(3),
                F::from_canonical_u64(0)
            ]
        );
    }
}
