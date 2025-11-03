use poulpy_core::{
    layouts::{GGLWEInfos, GGLWEPreparedToRef, GLWEAutomorphismKeyHelper, GetGaloisElement},
    GLWEAdd, GLWECopy, GLWERotate, GLWETrace, ScratchTakeCore,
};
use poulpy_hal::layouts::{Backend, DataMut, DataRef, Module, Scratch};
use poulpy_schemes::{
    define_bdd_2w_to_1w_trait, impl_bdd_2w_to_1w_trait,
    tfhe::bdd_arithmetic::{
        Add, And, ExecuteBDDCircuit2WTo1W, FheUint, FheUintPrepared, GLWEBlindRotation, Or, Sll,
        Slt, Sltu, Sra, Srl, Sub, UnsignedInteger, Xor,
    },
};

use strum_macros::EnumIter;

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

#[derive(Debug, EnumIter)]
pub enum RVI32ArithmeticOps {
    None,
    Lui,
    Jalr,
    Auipc,
    Addi,
    Slti,
    Sltiu,
    Xori,
    Ori,
    Andi,
    Slli,
    Srli,
    Srai,
    Add,
    Sub,
    Sll,
    Slt,
    Sltu,
    Xor,
    Srl,
    Sra,
    Or,
    And,
}

pub trait Evaluate<T: UnsignedInteger, BE: Backend> {
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

impl<BE: Backend> Evaluate<u32, BE> for RVI32ArithmeticOps {
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
            Self::Jalr => res.auipc(module, pc, pc, key, scratch), // ok? second input is not used
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
        }
    }
}

pub trait VMArithmetic<T: UnsignedInteger, BE: Backend> {
    fn eval_ops<RD, R1, R2, IM, PC, OPS, K, H, EVAL>(
        &self,
        rd: &mut FheUint<RD, T>,
        rs1: &FheUintPrepared<R1, u32, BE>,
        rs2: &FheUintPrepared<R2, u32, BE>,
        imm: &FheUintPrepared<IM, u32, BE>,
        pc: &FheUintPrepared<PC, u32, BE>,
        ops: OPS,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        RD: DataMut,
        R1: DataRef,
        R2: DataRef,
        IM: DataRef,
        PC: DataRef,
        EVAL: Evaluate<T, BE>,
        OPS: IntoIterator<Item = EVAL>,
        OPS::IntoIter: ExactSizeIterator,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        Scratch<BE>: ScratchTakeCore<BE>;

    fn select_rd<RD, O, K, H>(
        &self,
        rd: &mut FheUint<RD, T>,
        op_id: &FheUintPrepared<O, T, BE>,
        ops_len: usize,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        RD: DataMut,
        O: DataRef,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        Scratch<BE>: ScratchTakeCore<BE>;
}

impl<T: UnsignedInteger, BE: Backend> VMArithmetic<T, BE> for Module<BE>
where
    Self: GLWECopy
        + GLWEAdd
        + GLWERotate<BE>
        + GLWEBlindRotation<T, BE>
        + GLWETrace<BE>
        + ExecuteBDDCircuit2WTo1W<T, BE>,
{
    fn eval_ops<RD, R1, R2, IM, PC, OPS, K, H, EVAL>(
        &self,
        rd: &mut FheUint<RD, T>,
        rs1: &FheUintPrepared<R1, u32, BE>,
        rs2: &FheUintPrepared<R2, u32, BE>,
        imm: &FheUintPrepared<IM, u32, BE>,
        pc: &FheUintPrepared<PC, u32, BE>,
        ops: OPS,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        RD: DataMut,
        R1: DataRef,
        R2: DataRef,
        IM: DataRef,
        PC: DataRef,
        EVAL: Evaluate<T, BE>,
        OPS: IntoIterator<Item = EVAL>,
        OPS::IntoIter: ExactSizeIterator,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let ops_vec: Vec<_> = ops.into_iter().collect();
        let n: usize = ops_vec.len();

        let (mut ops_res, scratch_1) = scratch.take_glwe_slice(ops_vec.len(), rd);

        for (i, op) in ops_vec.iter().enumerate() {
            op.eval(
                self,
                &mut FheUint::from_glwe_to_mut(&mut ops_res[i]),
                rs1,
                rs2,
                imm,
                pc,
                keys,
                scratch_1,
            );
        }

        // Packs all results in a single ciphertext
        for i in 0..n {
            if i == 0 {
                self.glwe_copy(rd, &mut ops_res[i]);
            } else {
                self.glwe_add_inplace(rd, &mut ops_res[i]);
            }

            if i < n {
                self.glwe_rotate_inplace(-1, rd, scratch_1);
            }
        }
        self.glwe_rotate_inplace(n as i64, rd, scratch);
    }

    fn select_rd<RD, O, K, H>(
        &self,
        rd: &mut FheUint<RD, T>,
        op_id: &FheUintPrepared<O, T, BE>,
        ops_len: usize,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        RD: DataMut,
        O: DataRef,
        K: GGLWEPreparedToRef<BE> + GetGaloisElement + GGLWEInfos,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let log_size = (usize::BITS - (ops_len - 1).leading_zeros()) as usize;

        self.glwe_blind_rotation_inplace(rd, op_id, false, 0, log_size, 0, scratch);
        // Clean other values
        self.glwe_trace_inplace(rd, T::LOG_BITS as usize, self.log_n(), keys, scratch);
    }
}
