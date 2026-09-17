use poulpy_bin_fhe::{
    bdd_arithmetic::{
        Add, And, ExecuteBDDCircuit2WTo1W, FheUint, FheUintPrepared, Or, Sll, Slt, Sltu, Sra, Srl,
        Sub, UnsignedInteger, Xor,
    },
    define_bdd_2w_to_1w_trait, impl_bdd_2w_to_1w_trait,
};
use poulpy_core::{
    layouts::{GGSWInfos, GLWEInfos, GLWEToBackendMut, GetAutomorphismKey},
    GLWEBytesOf,
};
use poulpy_hal::layouts::{Backend, Module, ScratchArena};

#[allow(unused_imports)]
use crate::backend::prelude::*;
use crate::{
    backend::{for_each_with_scratch, worker_bytes, PhantomBackend, PhantomModule},
    RD_UPDATE,
};

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

pub trait Evaluate<T: UnsignedInteger, BE: PhantomBackend> {
    /// Scratch bytes needed by [`Self::eval_enc`] with one thread.
    fn eval_enc_tmp_bytes<R, G, H>(
        &self,
        module: &Module<BE>,
        res_infos: &R,
        ggsw_infos: &G,
        keys: &H,
    ) -> usize
    where
        R: GLWEInfos,
        G: GGSWInfos,
        H: GetAutomorphismKey<BE>,
        Module<BE>: PhantomModule<BE>;

    #[allow(clippy::too_many_arguments)]
    fn eval_enc<H>(
        &self,
        threads: usize,
        module: &Module<BE>,
        res: &mut FheUint<Vec<u8>, u32, i64>,
        rs1: &FheUintPrepared<Vec<u8>, u32, BE>,
        rs2: &FheUintPrepared<Vec<u8>, u32, BE>,
        imm: &FheUintPrepared<Vec<u8>, u32, BE>,
        pc: &FheUintPrepared<Vec<u8>, u32, BE>,
        ram: &FheUint<Vec<u8>, u32, i64>,
        keys: &H,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        H: GetAutomorphismKey<BE>,
        Module<BE>: PhantomModule<BE>;
}

impl<BE: PhantomBackend> Evaluate<u32, BE> for RD_UPDATE {
    fn eval_enc_tmp_bytes<R, G, H>(
        &self,
        module: &Module<BE>,
        res_infos: &R,
        ggsw_infos: &G,
        keys: &H,
    ) -> usize
    where
        R: GLWEInfos,
        G: GGSWInfos,
        H: GetAutomorphismKey<BE>,
        Module<BE>: PhantomModule<BE>,
    {
        // A `FheUint` is only needed to name the op's `*_tmp_bytes` method.
        let probe: FheUint<Vec<u8>, u32, i64> =
            FheUint::alloc(module, res_infos.base2k(), res_infos.k(), res_infos.rank());
        // The load ops isolate bytes with rotations and traces.
        let atk = keys
            .get_automorphism_key(-1, res_infos.k())
            .unwrap_or_else(|e| panic!("{e}"));
        let byte_ops: usize = module
            .glwe_trace_tmp_bytes(res_infos, res_infos, &atk)
            .max(module.glwe_rotate_tmp_bytes());
        match self {
            Self::NONE => 0,
            Self::AUIPC => probe.auipc_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::JAL | Self::JALR => probe.jalr_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::LUI => probe.lui_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::ADD | Self::ADDI => probe.add_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::SUB => probe.sub_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::SLL | Self::SLLI => probe.sll_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::SLT | Self::SLTI => probe.slt_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::SLTU | Self::SLTIU => probe.sltu_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::XOR | Self::XORI => probe.xor_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::SRL | Self::SRLI => probe.srl_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::SRA | Self::SRAI => probe.sra_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::OR | Self::ORI => probe.or_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::AND | Self::ANDI => probe.and_tmp_bytes(module, res_infos, ggsw_infos, keys),
            Self::LB | Self::LBU | Self::LH | Self::LHU | Self::LW => byte_ops,
            _ => 0,
        }
    }

    fn eval_enc<H>(
        &self,
        threads: usize,
        module: &Module<BE>,
        res: &mut FheUint<Vec<u8>, u32, i64>,
        rs1: &FheUintPrepared<Vec<u8>, u32, BE>,
        rs2: &FheUintPrepared<Vec<u8>, u32, BE>,
        imm: &FheUintPrepared<Vec<u8>, u32, BE>,
        pc: &FheUintPrepared<Vec<u8>, u32, BE>,
        ram: &FheUint<Vec<u8>, u32, i64>,
        keys: &H,
        scratch: &mut ScratchArena<'_, BE>,
    ) where
        H: GetAutomorphismKey<BE>,
        Module<BE>: PhantomModule<BE>,
    {
        match self {
            Self::NONE => {}
            Self::AUIPC => res.auipc_multi_thread(threads, module, pc, imm, keys, scratch),
            Self::JAL => res.jalr_multi_thread(threads, module, pc, pc, keys, scratch), // ok? second input is not used
            Self::JALR => res.jalr_multi_thread(threads, module, pc, pc, keys, scratch), // ok? second input is not used
            Self::LUI => res.lui_multi_thread(threads, module, imm, imm, keys, scratch),
            Self::ADD => res.add_multi_thread(threads, module, rs1, rs2, keys, scratch),
            Self::SUB => res.sub_multi_thread(threads, module, rs1, rs2, keys, scratch),
            Self::SLL => res.sll_multi_thread(threads, module, rs1, rs2, keys, scratch),
            Self::SLT => res.slt_multi_thread(threads, module, rs1, rs2, keys, scratch),
            Self::SLTU => res.sltu_multi_thread(threads, module, rs1, rs2, keys, scratch),
            Self::XOR => res.xor_multi_thread(threads, module, rs1, rs2, keys, scratch),
            Self::SRL => res.srl_multi_thread(threads, module, rs1, rs2, keys, scratch),
            Self::SRA => res.sra_multi_thread(threads, module, rs1, rs2, keys, scratch),
            Self::OR => res.or_multi_thread(threads, module, rs1, rs2, keys, scratch),
            Self::ADDI => res.add_multi_thread(threads, module, rs1, imm, keys, scratch),
            Self::AND => res.and_multi_thread(threads, module, rs1, rs2, keys, scratch),
            Self::SLTIU => res.sltu_multi_thread(threads, module, rs1, imm, keys, scratch),
            Self::XORI => res.xor_multi_thread(threads, module, rs1, imm, keys, scratch),
            Self::ORI => res.or_multi_thread(threads, module, rs1, imm, keys, scratch),
            Self::ANDI => res.and_multi_thread(threads, module, rs1, imm, keys, scratch),
            Self::SLLI => res.sll_multi_thread(threads, module, rs1, imm, keys, scratch),
            Self::SRLI => res.srl_multi_thread(threads, module, rs1, imm, keys, scratch),
            Self::SRAI => res.sra_multi_thread(threads, module, rs1, imm, keys, scratch),
            Self::SLTI => res.slt_multi_thread(threads, module, rs1, imm, keys, scratch),
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

/// Evaluates every op of `ops` on the same operands, one op per task over at
/// most `threads` workers, writing `res[i] = ops[i](...)`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn eval_ops_parallel<H, BE: PhantomBackend>(
    threads: usize,
    module: &Module<BE>,
    ops: &[RD_UPDATE],
    res: &mut [FheUint<Vec<u8>, u32, i64>],
    rs1: &FheUintPrepared<Vec<u8>, u32, BE>,
    rs2: &FheUintPrepared<Vec<u8>, u32, BE>,
    imm: &FheUintPrepared<Vec<u8>, u32, BE>,
    pc: &FheUintPrepared<Vec<u8>, u32, BE>,
    ram: &FheUint<Vec<u8>, u32, i64>,
    keys: &H,
    scratch: &mut ScratchArena<'_, BE>,
) where
    H: Sync + GetAutomorphismKey<BE>,
    Module<BE>: PhantomModule<BE>,
{
    assert_eq!(ops.len(), res.len());
    if res.is_empty() {
        return;
    }
    let per_worker: usize = worker_bytes::<BE>(
        ops.iter()
            .map(|op| op.eval_enc_tmp_bytes(module, &res[0], rs1, keys))
            .max()
            .unwrap_or(0),
    );
    for_each_with_scratch(threads, res, 0, per_worker, scratch, &|i, out, scratch| {
        ops[i].eval_enc(
            1,
            module,
            out,
            rs1,
            rs2,
            imm,
            pc,
            ram,
            keys,
            &mut scratch.borrow(),
        );
    });
}
