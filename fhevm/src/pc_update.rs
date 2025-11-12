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
#[cfg(test)]
use crate::PC_UPDATE;

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
    M: ModuleLogN + GLWEPacking<BE> + GLWECopy + ExecuteBDDCircuit<BE>,
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

#[cfg(test)]
pub(crate) struct PCU {
    pub(crate) op_type: PC_UPDATE,
    // registers
    pub(crate) rs1: u32,
    pub(crate) rs2: u32,
    // program counter
    pub(crate) pc: u32,
    // 20 bit immediate
    pub(crate) imm: u32,
}

#[cfg(test)]
impl PCU {
    pub(crate) const NONE: PCU = PCU::new(PC_UPDATE::NONE);

    pub(crate) const BEQ: PCU = PCU::new(PC_UPDATE::BEQ);
    pub(crate) const BNE: PCU = PCU::new(PC_UPDATE::BNE);

    pub(crate) const BLT: PCU = PCU::new(PC_UPDATE::BLT);
    pub(crate) const BGE: PCU = PCU::new(PC_UPDATE::BGE);

    pub(crate) const BLTU: PCU = PCU::new(PC_UPDATE::BLTU);
    pub(crate) const BGEU: PCU = PCU::new(PC_UPDATE::BGEU);

    pub(crate) const JAL: PCU = PCU::new(PC_UPDATE::JAL);
    pub(crate) const JALR: PCU = PCU::new(PC_UPDATE::JALR);

    pub(crate) const fn new(op_type: PC_UPDATE) -> Self {
        Self {
            op_type,
            rs1: 0,
            rs2: 0,
            pc: 0,
            imm: 0,
        }
    }

    pub(crate) fn u_rs1(mut self, rs1: u32) -> Self {
        self.rs1 = rs1;
        self
    }

    pub(crate) fn u_rs2(mut self, rs2: u32) -> Self {
        self.rs2 = rs2;
        self
    }

    pub(crate) fn u_pc(mut self, pc: u32) -> Self {
        self.pc = pc;
        self
    }

    pub(crate) fn u_imm(mut self, imm: u32) -> Self {
        let imm = imm & ((1 << 20) - 1);
        self.imm = imm;
        self
    }

    pub(crate) fn set_rs2_equal_rs1(mut self) -> Self {
        self.rs2 = self.rs1;
        self
    }

    pub(crate) fn set_rs1_lt_rs2(mut self) -> Self {
        if self.rs1 == self.rs2 {
            self.rs1 += 1;
        }
        let tmp = self.rs2;
        self.rs2 = std::cmp::max(self.rs1, self.rs2);
        self.rs1 = std::cmp::min(tmp, self.rs1);
        self
    }

    pub(crate) fn set_rs1_gte_rs2(mut self) -> Self {
        let tmp = self.rs2;
        self.rs2 = std::cmp::min(self.rs1, self.rs2);
        self.rs1 = std::cmp::max(tmp, self.rs1);
        self
    }

    pub(crate) fn set_rs1_lt_rs2_signed(mut self) -> Self {
        if self.rs1 == self.rs2 {
            self.rs1 += 1;
        }
        let tmp = self.rs2 as i32;
        self.rs2 = std::cmp::max(self.rs1 as i32, self.rs2 as i32) as u32;
        self.rs1 = std::cmp::min(tmp, self.rs1 as i32) as u32;
        self
    }

    pub(crate) fn set_rs1_gte_rs2_signed(mut self) -> Self {
        let tmp = self.rs2 as i32;
        self.rs2 = std::cmp::min(self.rs1 as i32, self.rs2 as i32) as u32;
        self.rs1 = std::cmp::max(tmp, self.rs1 as i32) as u32;
        self
    }

    pub(crate) fn expected_update(&self) -> u32 {
        use crate::sext;
        self.op_type.eval_plain(sext(self.imm, 19), self.rs1, self.rs2, self.pc)
    }
}
