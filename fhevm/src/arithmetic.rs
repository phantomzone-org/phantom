use poulpy_core::{
    layouts::{GGLWEInfos, GGLWEPreparedToRef, GLWEAutomorphismKeyHelper, GetGaloisElement},
    ScratchTakeCore,
};
use poulpy_hal::layouts::{Backend, DataMut, DataRef, Scratch};
use poulpy_schemes::{
    define_bdd_2w_to_1w_trait, impl_bdd_2w_to_1w_trait,
    tfhe::bdd_arithmetic::{
        Add, And, ExecuteBDDCircuit2WTo1W, FheUint, FheUintPrepared, Or, Sll, Slt, Sltu, Sra, Srl,
        Sub, UnsignedInteger, Xor,
    },
};

use crate::{OpID, RdOps};

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
    fn id(&self) -> usize;

    fn eval<R, A, B, I, P, H, K, M>(
        &self,
        module: &M,
        res: &mut FheUint<R, u32>,
        rs1: &FheUintPrepared<A, u32, BE>,
        rs2: &FheUintPrepared<B, u32, BE>,
        imm: &FheUintPrepared<I, u32, BE>,
        pc: &FheUintPrepared<P, u32, BE>,
        key: &H,
        scratch: &mut Scratch<BE>,
    ) where
        R: DataMut,
        A: DataRef,
        B: DataRef,
        I: DataRef,
        P: DataRef,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        M: ExecuteBDDCircuit2WTo1W<T, BE>,
        Scratch<BE>: ScratchTakeCore<BE>;
}

impl<BE: Backend> Evaluate<u32, BE> for RdOps {
    fn id(&self) -> usize {
        match self {
            Self::None => OpID::NONE.0 as usize,
            Self::Auipc => OpID::AUIPC.0 as usize,
            Self::Jal => OpID::JAL.0 as usize,
            Self::Jalr => OpID::JALR.0 as usize,
            Self::Lui => OpID::LUI.0 as usize,
            Self::Add => OpID::ADD.0 as usize,
            Self::Sub => OpID::SUB.0 as usize,
            Self::Sll => OpID::SLL.0 as usize,
            Self::Slt => OpID::SLT.0 as usize,
            Self::Sltu => OpID::SLTU.0 as usize,
            Self::Xor => OpID::XOR.0 as usize,
            Self::Srl => OpID::SRL.0 as usize,
            Self::Sra => OpID::SRA.0 as usize,
            Self::Or => OpID::OR.0 as usize,
            Self::Addi => OpID::ADDI.0 as usize,
            Self::And => OpID::AND.0 as usize,
            Self::Sltiu => OpID::SLTIU.0 as usize,
            Self::Xori => OpID::XORI.0 as usize,
            Self::Ori => OpID::ORI.0 as usize,
            Self::Andi => OpID::ANDI.0 as usize,
            Self::Slli => OpID::SLLI.0 as usize,
            Self::Srli => OpID::SRLI.0 as usize,
            Self::Srai => OpID::SRAI.0 as usize,
            Self::Slti => OpID::SLTI.0 as usize,
            _ => {
                unimplemented!()
            }
        }
    }

    fn eval<R, A, B, I, P, H, K, M>(
        &self,
        module: &M,
        res: &mut FheUint<R, u32>,
        rs1: &FheUintPrepared<A, u32, BE>,
        rs2: &FheUintPrepared<B, u32, BE>,
        imm: &FheUintPrepared<I, u32, BE>,
        pc: &FheUintPrepared<P, u32, BE>,
        key: &H,
        scratch: &mut Scratch<BE>,
    ) where
        R: DataMut,
        A: DataRef,
        B: DataRef,
        I: DataRef,
        P: DataRef,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        M: ExecuteBDDCircuit2WTo1W<u32, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        match self {
            Self::None => {}
            Self::Auipc => res.auipc(module, pc, imm, key, scratch),
            Self::Jal => res.jalr(module, pc, pc, key, scratch), // ok? second input is not used
            Self::Jalr => res.jalr(module, pc, pc, key, scratch), // ok? second input is not used
            Self::Lui => res.lui(module, imm, imm, key, scratch),
            Self::Add => res.add(module, rs1, rs2, key, scratch),
            Self::Sub => res.sub(module, rs1, rs2, key, scratch),
            Self::Sll => res.sll(module, rs1, rs2, key, scratch),
            Self::Slt => res.slt(module, rs1, rs2, key, scratch),
            Self::Sltu => res.sltu(module, rs1, rs2, key, scratch),
            Self::Xor => res.xor(module, rs1, rs2, key, scratch),
            Self::Srl => res.srl(module, rs1, rs2, key, scratch),
            Self::Sra => res.sra(module, rs1, rs2, key, scratch),
            Self::Or => res.or(module, rs1, rs2, key, scratch),
            Self::Addi => res.add(module, rs1, imm, key, scratch),
            Self::And => res.and(module, rs1, rs2, key, scratch),
            Self::Sltiu => res.sltu(module, rs1, imm, key, scratch),
            Self::Xori => res.xor(module, rs1, imm, key, scratch),
            Self::Ori => res.or(module, rs1, imm, key, scratch),
            Self::Andi => res.and(module, rs1, imm, key, scratch),
            Self::Slli => res.sll(module, rs1, imm, key, scratch),
            Self::Srli => res.srl(module, rs1, imm, key, scratch),
            Self::Srai => res.sra(module, rs1, imm, key, scratch),
            Self::Slti => res.slt(module, rs1, imm, key, scratch),
            _ => {
                unimplemented!()
            }
        }
    }
}
