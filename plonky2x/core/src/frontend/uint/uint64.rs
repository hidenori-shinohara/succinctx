use array_macro::array;
use ethers::types::U64;
use plonky2::field::types::Field;
use plonky2::hash::hash_types::RichField;

use super::uint256::U256Variable;
use super::Uint;
use crate::frontend::num::biguint::{BigUintTarget, CircuitBuilderBiguint};
use crate::frontend::num::u32::gadgets::arithmetic_u32::U32Target;
use crate::frontend::vars::{EvmVariable, SSZVariable, U32Variable};
use crate::prelude::{
    Add, BoolVariable, ByteVariable, Bytes32Variable, BytesVariable, CircuitBuilder,
    CircuitVariable, Div, LessThanOrEqual, Mul, One, PlonkParameters, Rem, Sub, Variable, Zero,
};
use crate::{make_uint32_n, make_uint32_n_tests};

impl Uint<2> for u64 {
    fn to_little_endian(&self, bytes: &mut [u8]) {
        U64::from(*self).to_little_endian(bytes);
    }

    fn from_little_endian(slice: &[u8]) -> Self {
        U64::from_little_endian(slice).as_u64()
    }

    fn to_big_endian(&self, bytes: &mut [u8]) {
        U64::from(*self).to_big_endian(bytes);
    }

    fn from_big_endian(slice: &[u8]) -> Self {
        U64::from_big_endian(slice).as_u64()
    }

    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let result = U64::from(self).overflowing_add(U64::from(rhs));
        (result.0.as_u64(), result.1)
    }

    fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        let result = U64::from(self).overflowing_sub(U64::from(rhs));
        (result.0.as_u64(), result.1)
    }

    fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
        let result = U64::from(self).overflowing_mul(U64::from(rhs));
        (result.0.as_u64(), result.1)
    }
}

make_uint32_n!(U64Variable, u64, 2);
make_uint32_n_tests!(U64Variable, u64, 2);

impl U64Variable {
    /// Converts a U64Variable to Variable with overflow.
    ///
    /// Note: This function assumes that the U64 is in the range [0, 2^64-2^32+1). Otherwise, it
    /// will overflow.
    pub fn to_variable_with_overflow<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> Variable {
        let digit = builder.constant::<Variable>(L::Field::from_canonical_u64(1 << 32));
        let mut result = builder.mul(self.limbs[1].variable, digit);
        result = builder.add(result, self.limbs[0].variable);
        result
    }

    pub fn to_u256<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> U256Variable {
        let zero = builder.zero::<U32Variable>();
        let result = builder.init::<U256Variable>();
        for i in 0..result.limbs.len() {
            if i < self.limbs.len() {
                builder.connect(self.limbs[i], result.limbs[i]);
            } else {
                builder.connect(zero, result.limbs[i]);
            }
        }
        result
    }
}
