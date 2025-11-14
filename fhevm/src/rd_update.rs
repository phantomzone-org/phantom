use poulpy_core::{
    layouts::{GGLWEInfos, GGLWEPreparedToRef, GLWEAutomorphismKeyHelper, GetGaloisElement},
    GLWEAdd, GLWECopy, GLWERotate, GLWESub, GLWETrace, ScratchTakeCore,
};
use poulpy_hal::{
    api::ModuleLogN,
    layouts::{Backend, DataMut, DataRef, Scratch},
};
use poulpy_schemes::{
    define_bdd_2w_to_1w_trait, impl_bdd_2w_to_1w_trait,
    tfhe::bdd_arithmetic::{
        Add, And, ExecuteBDDCircuit2WTo1W, FheUint, FheUintPrepared, Or, Sll, Slt, Sltu, Sra, Srl,
        Sub, UnsignedInteger, Xor,
    },
};

use crate::RD_UPDATE;

define_bdd_2w_to_1w_trait!(pub Auipc, auipc);
impl_bdd_2w_to_1w_trait!(
    Auipc,
    auipc,
    u32,
    crate::codegen::codegen_auipc::AnyBitCircuit,
    crate::codegen::codegen_auipc::OUTPUT_CIRCUITS
);

define_bdd_2w_to_1w_trait!(pub Jalr, jalr);
impl_bdd_2w_to_1w_trait!(
    Jalr,
    jalr,
    u32,
    crate::codegen::codegen_jalr::AnyBitCircuit,
    crate::codegen::codegen_jalr::OUTPUT_CIRCUITS
);

define_bdd_2w_to_1w_trait!(pub Lui, lui);
impl_bdd_2w_to_1w_trait!(
    Lui,
    lui,
    u32,
    crate::codegen::codegen_lui::AnyBitCircuit,
    crate::codegen::codegen_lui::OUTPUT_CIRCUITS
);

pub trait Evaluate<T: UnsignedInteger, BE: Backend> {
    fn eval_enc<R, R1, R2, IM, PC, RA, H, K, M>(
        &self,
        module: &M,
        res: &mut FheUint<R, u32>,
        rs1: &FheUintPrepared<R1, u32, BE>,
        rs2: &FheUintPrepared<R2, u32, BE>,
        imm: &FheUintPrepared<IM, u32, BE>,
        pc: &FheUintPrepared<PC, u32, BE>,
        ram: &FheUint<RA, u32>,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        R: DataMut,
        R1: DataRef,
        R2: DataRef,
        IM: DataRef,
        PC: DataRef,
        RA: DataRef,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        M: ExecuteBDDCircuit2WTo1W<u32, BE>
            + ModuleLogN
            + GLWERotate<BE>
            + GLWETrace<BE>
            + GLWEAdd
            + GLWESub
            + GLWECopy,
        Scratch<BE>: ScratchTakeCore<BE>;
}

impl<BE: Backend> Evaluate<u32, BE> for RD_UPDATE {
    fn eval_enc<R, R1, R2, IM, PC, RA, H, K, M>(
        &self,
        module: &M,
        res: &mut FheUint<R, u32>,
        rs1: &FheUintPrepared<R1, u32, BE>,
        rs2: &FheUintPrepared<R2, u32, BE>,
        imm: &FheUintPrepared<IM, u32, BE>,
        pc: &FheUintPrepared<PC, u32, BE>,
        ram: &FheUint<RA, u32>,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        R: DataMut,
        R1: DataRef,
        R2: DataRef,
        IM: DataRef,
        PC: DataRef,
        RA: DataRef,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        M: ExecuteBDDCircuit2WTo1W<u32, BE>
            + ModuleLogN
            + GLWERotate<BE>
            + GLWETrace<BE>
            + GLWEAdd
            + GLWESub
            + GLWECopy,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        match self {
            Self::NONE => {}
            Self::AUIPC => res.auipc(module, pc, imm, keys, scratch),
            Self::JAL => res.jalr(module, pc, pc, keys, scratch), // ok? second input is not used
            Self::JALR => res.jalr(module, pc, pc, keys, scratch), // ok? second input is not used
            Self::LUI => res.lui(module, imm, imm, keys, scratch),
            Self::ADD => res.add(module, rs1, rs2, keys, scratch),
            Self::SUB => res.sub(module, rs1, rs2, keys, scratch),
            Self::SLL => res.sll(module, rs1, rs2, keys, scratch),
            Self::SLT => res.slt(module, rs1, rs2, keys, scratch),
            Self::SLTU => res.sltu(module, rs1, rs2, keys, scratch),
            Self::XOR => res.xor(module, rs1, rs2, keys, scratch),
            Self::SRL => res.srl(module, rs1, rs2, keys, scratch),
            Self::SRA => res.sra(module, rs1, rs2, keys, scratch),
            Self::OR => res.or(module, rs1, rs2, keys, scratch),
            Self::ADDI => res.add(module, rs1, imm, keys, scratch),
            Self::AND => res.and(module, rs1, rs2, keys, scratch),
            Self::SLTIU => res.sltu(module, rs1, imm, keys, scratch),
            Self::XORI => res.xor(module, rs1, imm, keys, scratch),
            Self::ORI => res.or(module, rs1, imm, keys, scratch),
            Self::ANDI => res.and(module, rs1, imm, keys, scratch),
            Self::SLLI => res.sll(module, rs1, imm, keys, scratch),
            Self::SRLI => res.srl(module, rs1, imm, keys, scratch),
            Self::SRAI => res.sra(module, rs1, imm, keys, scratch),
            Self::SLTI => res.slt(module, rs1, imm, keys, scratch),
            Self::LB => {
                module.glwe_copy(res, ram);
                res.zero_byte(module, 1, keys, scratch);
                res.zero_byte(module, 2, keys, scratch);
                res.zero_byte(module, 3, keys, scratch);
                //res.sext(module, 0, keys, scratch);
            }
            Self::LBU => {
                module.glwe_copy(res, ram);
                res.zero_byte(module, 1, keys, scratch);
                res.zero_byte(module, 2, keys, scratch);
                res.zero_byte(module, 3, keys, scratch);
            }
            Self::LH => {
                module.glwe_copy(res, ram);
                res.zero_byte(module, 2, keys, scratch);
                res.zero_byte(module, 3, keys, scratch);
                res.sext(module, 1, keys, scratch);
            }
            Self::LHU => {
                module.glwe_copy(res, ram);
                res.zero_byte(module, 2, keys, scratch);
                res.zero_byte(module, 3, keys, scratch);
            }
            Self::LW => {
                module.glwe_copy(res, ram);
            }
            _ => {
                unimplemented!()
            }
        }
    }
}
