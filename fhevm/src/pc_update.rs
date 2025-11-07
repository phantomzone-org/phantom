use std::marker::PhantomData;

use poulpy_core::{layouts::GGSWPrepared, GLWECopy, GLWEPacking, ScratchTakeCore};
use poulpy_hal::{
    api::ModuleLogN,
    layouts::{Backend, DataMut, DataRef, Scratch},
};
use poulpy_schemes::tfhe::bdd_arithmetic::{
    BitSize, ExecuteBDDCircuit, FheUint, FheUintPrepared, GetGGSWBit, UnsignedInteger,
};

use crate::keys::RAMKeysHelper;

pub(crate) fn update_pc<R, OPID, PC, RS1, RS2, IMM, H, K, M, BE: Backend>(
    module: &M,
    res: &mut FheUint<R, u32>,
    rs1: &FheUintPrepared<RS1, u32, BE>,
    rs2: &FheUintPrepared<RS2, u32, BE>,
    pc: &FheUintPrepared<PC, u32, BE>,
    imm: &FheUintPrepared<IMM, u32, BE>,
    op_id: &FheUintPrepared<OPID, u32, BE>,
    keys: &H,
    scratch: &mut Scratch<BE>,
) where
    R: DataMut,
    K: DataRef,
    OPID: DataRef,
    PC: DataRef,
    RS1: DataRef,
    RS2: DataRef,
    IMM: DataRef,
    H: RAMKeysHelper<K, BE>,
    M: ModuleLogN + GLWEPacking<BE> + GLWECopy + ExecuteBDDCircuit<u32, BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
{
    let inputs: Vec<&dyn GetGGSWBit<BE>> = [
        op_id as &dyn GetGGSWBit<BE>,
        rs1 as &dyn GetGGSWBit<BE>,
        rs2 as &dyn GetGGSWBit<BE>,
        pc as &dyn GetGGSWBit<BE>,
        imm as &dyn GetGGSWBit<BE>,
    ]
    .to_vec();
    let helper: FheUintHelper<'_, u32, BE> = FheUintHelper {
        data: inputs,
        _phantom: PhantomData,
    };

    let (mut out_bits, scratch_1) = scratch.take_glwe_slice(u32::BITS as usize, res);

    // Evaluates out[i] = circuit[i](a, b)
    module.execute_bdd_circuit(
        &mut out_bits,
        &helper,
        &crate::codegen::codegen_pc_update::OUTPUT_CIRCUITS,
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
        120
    }
}

impl<'a, T: UnsignedInteger, BE: Backend> GetGGSWBit<BE> for FheUintHelper<'a, T, BE> {
    fn get_bit(&self, bit: usize) -> GGSWPrepared<&[u8], BE> {
        const OFFSETS: [usize; 5] = [
            0,   // opid start
            4,   // rs1 start
            36,  // rs2 start
            68,  // pc start
            100, // imm start
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
