pub(crate) mod address_read;
pub(crate) mod address_write;
pub(crate) mod base;
pub(crate) mod coordinate;
pub(crate) mod coordinate_prepared;
pub(crate) mod ram;

#[inline(always)]
pub fn reverse_bits_msb(x: usize, n: u32) -> usize {
    x.reverse_bits() >> (usize::BITS - n)
}
