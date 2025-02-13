use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::iop::target::BoolTarget;
use plonky2::plonk::circuit_builder::CircuitBuilder;

use crate::frontend::num::biguint::BigUintTarget;
use crate::frontend::num::u32::gadgets::arithmetic_u32::U32Target;

pub fn _right_rotate<const S: usize>(n: [BoolTarget; S], bits: usize) -> [BoolTarget; S] {
    let mut res = [None; S];
    for i in 0..S {
        res[i] = Some(n[((S - bits) + i) % S])
    }
    res.map(|x| x.unwrap())
}

pub fn _shr<F: RichField + Extendable<D>, const D: usize, const S: usize>(
    n: [BoolTarget; S],
    bits: i64,
    builder: &mut CircuitBuilder<F, D>,
) -> [BoolTarget; S] {
    let mut res = [None; S];
    for i in 0..S {
        if (i as i64) < bits {
            res[i] = Some(BoolTarget::new_unsafe(builder.constant(F::ZERO)));
        } else {
            res[i] = Some(n[(i as i64 - bits) as usize]);
        }
    }
    res.map(|x| x.unwrap())
}

pub fn u64_to_bits<F: RichField + Extendable<D>, const D: usize>(
    value: u64,
    builder: &mut CircuitBuilder<F, D>,
) -> [BoolTarget; 64] {
    let mut bits = [None; 64];
    for (i, bit) in bits.iter_mut().enumerate() {
        if value & (1 << (63 - i)) != 0 {
            *bit = Some(BoolTarget::new_unsafe(builder.constant(F::ONE)));
        } else {
            *bit = Some(BoolTarget::new_unsafe(builder.constant(F::ZERO)));
        }
    }
    bits.map(|x| x.unwrap())
}

pub fn uint32_to_bits<F: RichField + Extendable<D>, const D: usize>(
    value: u32,
    builder: &mut CircuitBuilder<F, D>,
) -> [BoolTarget; 32] {
    let mut bits = [None; 32];
    for (i, bit) in bits.iter_mut().enumerate() {
        if value & (1 << (31 - i)) != 0 {
            *bit = Some(BoolTarget::new_unsafe(builder.constant(F::ONE)));
        } else {
            *bit = Some(BoolTarget::new_unsafe(builder.constant(F::ZERO)));
        }
    }
    bits.map(|x| x.unwrap())
}

pub fn biguint_to_bits_target<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    a: &BigUintTarget,
) -> Vec<BoolTarget> {
    let mut res = Vec::new();
    for i in (0..a.num_limbs()).rev() {
        let bit_targets = builder.split_le_base::<2>(a.get_limb(i).target, 32);
        for j in (0..32).rev() {
            res.push(BoolTarget::new_unsafe(bit_targets[j]));
        }
    }
    res
}

// The bits_target needs to be in big-endian format.
pub fn bits_to_biguint_target<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    bits_target: Vec<BoolTarget>,
) -> BigUintTarget {
    let bit_len = bits_target.len();
    assert_eq!(bit_len % 32, 0);

    let mut u32_targets = Vec::new();
    for i in 0..bit_len / 32 {
        // Per the assert above, we are guaranteed that the chunks will be 32 bool targets,
        // so will fit within a U32Target.
        u32_targets.push(U32Target::from_target_unsafe(
            builder.le_sum(bits_target[i * 32..(i + 1) * 32].iter().rev()),
        ));
    }
    u32_targets.reverse();
    BigUintTarget { limbs: u32_targets }
}
