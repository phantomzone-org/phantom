use std::marker::PhantomData;

use poulpy_core::{
    layouts::{
        GGLWEInfos, GGLWEPreparedToRef, GGSWPrepared, GLWEAutomorphismKeyHelper, GetGaloisElement,
    },
    GLWECopy, GLWEPacking, ScratchTakeCore,
};
use poulpy_hal::{
    api::ModuleLogN,
    layouts::{Backend, DataMut, DataRef, Scratch},
};
use poulpy_schemes::bin_fhe::bdd_arithmetic::{
    BitSize, ExecuteBDDCircuit, FheUint, FheUintPrepared, GetGGSWBit, UnsignedInteger,
};

pub(crate) fn ram_offset<R, RS1, IMM, H, K, M, BE: Backend>(
    threads: usize,
    module: &M,
    res: &mut FheUint<R, u32>,
    rs1: &FheUintPrepared<RS1, u32, BE>,
    imm: &FheUintPrepared<IMM, u32, BE>,
    keys: &H,
    scratch: &mut Scratch<BE>,
) where
    R: DataMut,
    RS1: DataRef,
    IMM: DataRef,
    H: GLWEAutomorphismKeyHelper<K, BE>,
    K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
    M: ModuleLogN + GLWEPacking<BE> + GLWECopy + ExecuteBDDCircuit<BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
{
    let inputs: Vec<&dyn GetGGSWBit<BE>> =
        [rs1 as &dyn GetGGSWBit<BE>, imm as &dyn GetGGSWBit<BE>].to_vec();
    let helper: FheUintHelper<'_, u32, BE> = FheUintHelper {
        data: inputs,
        _phantom: PhantomData,
    };

    let (mut out_bits, scratch_1) = scratch.take_glwe_slice(u32::BITS as usize, res);

    // Evaluates out[i] = circuit[i](a, b)
    module.execute_bdd_circuit_multi_thread(
        threads,
        &mut out_bits,
        &helper,
        &crate::codegen::codegen_ram_offset::OUTPUT_CIRCUITS,
        scratch_1,
    );

    // Repacks the bits
    res.pack(module, out_bits, keys, scratch_1);
}

struct FheUintHelper<'a, T: UnsignedInteger, BE: Backend> {
    data: Vec<&'a dyn GetGGSWBit<BE>>,
    _phantom: PhantomData<T>,
}

impl<'a, T: UnsignedInteger, BE: Backend> BitSize for FheUintHelper<'a, T, BE> {
    fn bit_size(&self) -> usize {
        64
    }
}

impl<'a, T: UnsignedInteger, BE: Backend> GetGGSWBit<BE> for FheUintHelper<'a, T, BE> {
    fn get_bit(&self, bit: usize) -> GGSWPrepared<&[u8], BE> {
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
