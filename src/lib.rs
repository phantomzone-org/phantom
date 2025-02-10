pub mod address;
pub mod circuit_bootstrapping;
pub mod decompose;
pub mod instructions;
pub mod interpreter;
pub mod memory;
pub mod packing;
pub mod parameters;
pub mod test_vector;
pub mod trace;

#[inline(always)]
pub fn reverse_bits_msb(x: usize, n: u32) -> usize {
    x.reverse_bits() >> (usize::BITS - n)
}
