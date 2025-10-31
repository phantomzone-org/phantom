mod address;
mod arithmetic;
mod base;
pub(crate) mod codegen;
mod conversion;
mod coordinate;
mod coordinate_prepared;
mod keys;
mod parameters;
mod ram;
mod store;

pub use address::*;
pub use arithmetic::*;
pub(crate) use base::*;
pub use conversion::*;
pub(crate) use coordinate::*;
pub(crate) use coordinate_prepared::*;
pub use keys::*;
pub use parameters::*;
pub use ram::*;
pub use store::*;

#[inline(always)]
pub fn reverse_bits_msb(x: usize, n: u32) -> usize {
    x.reverse_bits() >> (usize::BITS - n)
}
