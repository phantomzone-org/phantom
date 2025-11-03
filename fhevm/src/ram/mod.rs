mod address;
mod base;
mod coordinate;
mod coordinate_prepared;
mod ram;

pub use address::*;
pub use base::*;
pub(crate) use coordinate::*;
pub(crate) use coordinate_prepared::*;
pub use ram::*;

#[inline(always)]
pub fn reverse_bits_msb(x: usize, n: u32) -> usize {
    x.reverse_bits() >> (usize::BITS - n)
}
