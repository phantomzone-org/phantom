use std::marker::PhantomData;

use poulpy_bin_fhe::bdd_arithmetic::{
    BitSize, FheUint, FheUintPrepared, GetGGSWBit, UnsignedInteger,
};
use poulpy_core::{
    layouts::{prepared::GGSWPrepared, GetAutomorphismKey},
    ScratchArenaTakeCore,
};
use poulpy_hal::layouts::{Module, ScratchArena};

#[allow(unused_imports)]
use crate::backend::prelude::*;
use crate::backend::{execute_bdd_circuit_parallel, PhantomBackend, PhantomModule};

pub(crate) fn ram_offset<H, BE: PhantomBackend>(
    threads: usize,
    module: &Module<BE>,
    res: &mut FheUint<Vec<u8>, u32, i64>,
    rs1: &FheUintPrepared<Vec<u8>, u32, BE>,
    imm: &FheUintPrepared<Vec<u8>, u32, BE>,
    keys: &H,
    scratch: &mut ScratchArena<'_, BE>,
) where
    H: GetAutomorphismKey<BE>,
    Module<BE>: PhantomModule<BE>,
{
    let inputs: Vec<&dyn GetGGSWBit<BE>> =
        [rs1 as &dyn GetGGSWBit<BE>, imm as &dyn GetGGSWBit<BE>].to_vec();
    let helper: FheUintHelper<'_, u32, BE> = FheUintHelper {
        data: inputs,
        _phantom: PhantomData,
    };

    let (mut out_bits, mut scratch_1) = scratch
        .borrow()
        .take_glwe_slice_scratch(u32::BITS as usize, res);

    // Evaluates out[i] = circuit[i](a, b)
    execute_bdd_circuit_parallel(
        threads,
        module,
        &mut out_bits,
        &helper,
        &crate::codegen::codegen_ram_offset::OUTPUT_CIRCUITS,
        &mut scratch_1,
    );

    // Repacks the bits
    res.pack(module, out_bits, keys, &mut scratch_1);
}

struct FheUintHelper<'a, T: UnsignedInteger, BE: PhantomBackend> {
    data: Vec<&'a dyn GetGGSWBit<BE>>,
    _phantom: PhantomData<T>,
}

impl<'a, T: UnsignedInteger, BE: PhantomBackend> BitSize for FheUintHelper<'a, T, BE> {
    fn bit_size(&self) -> usize {
        64
    }
}

impl<'a, T: UnsignedInteger, BE: PhantomBackend> GetGGSWBit<BE> for FheUintHelper<'a, T, BE> {
    fn get_bit(&self, bit: usize) -> &GGSWPrepared<Vec<u8>, BE> {
        const OFFSETS: [usize; 2] = [
            0,  // rs1
            32, // imm
        ];

        // Find which segment the bit belongs to
        let (index, base) = OFFSETS
            .iter()
            .enumerate()
            .rev() // reverse so we find the largest offset <= bit
            .find(|(_, &start)| bit >= start)
            .unwrap(); // safe if bit is within expected range

        self.data[index].get_bit(bit - base)
    }
}
