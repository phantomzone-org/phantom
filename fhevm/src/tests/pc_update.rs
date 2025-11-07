use poulpy_backend::{FFT64Avx, FFT64Ref};
use poulpy_core::{
    layouts::{
        GGLWEToGGSWKeyPreparedFactory, GGSWPreparedFactory, GLWEAutomorphismKeyPreparedFactory,
        GLWEInfos, GLWESecret, GLWESecretPrepared, GLWESecretPreparedFactory, LWESecret,
    },
    GGLWEToGGSWKeyEncryptSk, GGSWAutomorphism, GLWEAutomorphismKeyEncryptSk, GLWEDecrypt,
    GLWEEncryptSk, GLWEExternalProduct, GLWEPackerOps, GLWEPacking, GLWETrace, ScratchTakeCore,
};
use poulpy_hal::{
    api::{ModuleN, ModuleNew, ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{Backend, Module, Scratch, ScratchOwned},
    source::Source,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{
        BDDKeyEncryptSk, BDDKeyPreparedFactory, FheUint, FheUintPrepare, FheUintPrepared,
        FheUintPreparedEncryptSk, FheUintPreparedFactory, GGSWBlindRotation,
    },
    blind_rotation::{BlindRotationAlgo, BlindRotationKey, BlindRotationKeyFactory, CGGI},
};
use rand_core::RngCore;

use crate::{
    keys::{VMKeys, VMKeysPrepared},
    parameters::CryptographicParameters,
    update_pc, OpIDPCUpdate,
};

#[test]
fn test_pc_update_fft64_ref() {
    test_pc_update::<CGGI, FFT64Avx>()
}

fn test_pc_update<BRA: BlindRotationAlgo, BE: Backend>()
where
    Module<BE>: ModuleNew<BE>
        + GLWESecretPreparedFactory<BE>
        + FheUintPreparedFactory<u32, BE>
        + ModuleN
        + GLWEEncryptSk<BE>
        + FheUintPreparedEncryptSk<u32, BE>
        + GLWEAutomorphismKeyEncryptSk<BE>
        + GGLWEToGGSWKeyEncryptSk<BE>
        + GLWETrace<BE>
        + BDDKeyEncryptSk<BRA, BE>
        + GGSWPreparedFactory<BE>
        + GLWEExternalProduct<BE>
        + GLWEPackerOps<BE>
        + GLWEPacking<BE>
        + FheUintPrepare<BRA, u32, BE>
        + GGSWBlindRotation<u32, BE>
        + GGSWPreparedFactory<BE>
        + GLWEDecrypt<BE>
        + GLWEAutomorphismKeyPreparedFactory<BE>
        + GGLWEToGGSWKeyPreparedFactory<BE>
        + BDDKeyPreparedFactory<BRA, BE>
        + GGSWAutomorphism<BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
    Scratch<BE>: ScratchTakeCore<BE>,
    BlindRotationKey<Vec<u8>, BRA>: BlindRotationKeyFactory<BRA>,
{
    let params: CryptographicParameters<BE> = CryptographicParameters::<BE>::new();
    let module: &Module<BE> = params.module();

    let mut source_xs: Source = Source::new([0u8; 32]);
    let mut source_xa: Source = Source::new([0u8; 32]);
    let mut source_xe: Source = Source::new([0u8; 32]);

    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BE> =
        GLWESecretPrepared::alloc(module, sk_glwe.rank());
    sk_glwe_prepared.prepare(module, &sk_glwe);

    let ggsw_infos: &poulpy_core::layouts::GGSWLayout = &params.ggsw_infos();
    let glwe_infos: &poulpy_core::layouts::GLWELayout = &params.glwe_ct_infos();

    let mut rs1_prep: FheUintPrepared<Vec<u8>, u32, BE> =
        FheUintPrepared::alloc_from_infos(module, ggsw_infos);
    let mut rs2_prep: FheUintPrepared<Vec<u8>, u32, BE> =
        FheUintPrepared::alloc_from_infos(module, ggsw_infos);
    let mut imm_prep: FheUintPrepared<Vec<u8>, u32, BE> =
        FheUintPrepared::alloc_from_infos(module, ggsw_infos);
    let mut pc_prep: FheUintPrepared<Vec<u8>, u32, BE> =
        FheUintPrepared::alloc_from_infos(module, ggsw_infos);
    let mut pc_id: FheUintPrepared<Vec<u8>, u32, BE> =
        FheUintPrepared::alloc_from_infos(module, ggsw_infos);
    let mut pc: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(glwe_infos);

    let keys: VMKeys<Vec<u8>, BRA> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);
    let mut keys_prepared: VMKeysPrepared<Vec<u8>, BRA, BE> = VMKeysPrepared::alloc(&params);
    keys_prepared.prepare(module, &keys, scratch.borrow());

    [
        PCU::BEQ
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs2_equal_rs1(),
        PCU::BEQ
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs1_lt_rs2(),
        PCU::BNE
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs1_lt_rs2(),
        PCU::BNE
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs2_equal_rs1(),
        PCU::BLTU
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs1_lt_rs2(),
        PCU::BLTU
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs1_gte_rs2(),
        PCU::BGEU
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs1_gte_rs2(),
        PCU::BGEU
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs1_lt_rs2(),
        PCU::BLT
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs1_lt_rs2_signed(),
        PCU::BLT
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs1_gte_rs2_signed(),
        PCU::BGE
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs1_gte_rs2_signed(),
        PCU::BGE
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32())
            .set_rs1_lt_rs2_signed(),
        PCU::JAL
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32()),
        PCU::JALR
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32()),
        PCU::NONE
            .u_pc(source_xa.next_u32())
            .u_imm(source_xa.next_u32())
            .u_rs1(source_xa.next_u32())
            .u_rs2(source_xa.next_u32()),
    ]
    .iter_mut()
    .for_each(|pcu| {
        rs1_prep.encrypt_sk(
            module,
            pcu.rs1,
            &sk_glwe_prepared,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );
        rs2_prep.encrypt_sk(
            module,
            pcu.rs2,
            &sk_glwe_prepared,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );
        imm_prep.encrypt_sk(
            module,
            pcu.imm,
            &sk_glwe_prepared,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );
        pc_prep.encrypt_sk(
            module,
            pcu.pc,
            &sk_glwe_prepared,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );
        pc_id.encrypt_sk(
            module,
            pcu.op_type as u32,
            &sk_glwe_prepared,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );

        update_pc(
            module,
            &mut pc,
            &rs1_prep,
            &rs2_prep,
            &pc_prep,
            &imm_prep,
            &pc_id,
            &keys_prepared,
            scratch.borrow(),
        );

        println!(
            "{} {}",
            pcu.expected_update(),
            pc.decrypt(module, &sk_glwe_prepared, scratch.borrow())
        );

        assert_eq!(
            pcu.expected_update(),
            pc.decrypt(module, &sk_glwe_prepared, scratch.borrow())
        );
    });
}

pub(crate) fn sign_extend(value: u32, bitlen: usize) -> u32 {
    assert!((value >> bitlen) == 0);
    let msb = (value >> (bitlen - 1)) & 1;
    let mut out_v = value;
    for i in bitlen..32 {
        out_v += msb << i;
    }
    return out_v;
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(u32)]
enum PCU_T {
    NONE = OpIDPCUpdate::NONE,
    BEQ = OpIDPCUpdate::BEQ,
    BNE = OpIDPCUpdate::BNE,
    BLT = OpIDPCUpdate::BLT,
    BGE = OpIDPCUpdate::BGE,
    BLTU = OpIDPCUpdate::BLTU,
    BGEU = OpIDPCUpdate::BGEU,
    JAL = OpIDPCUpdate::JAL,
    JALR = OpIDPCUpdate::JALR,
}

struct PCU {
    op_type: PCU_T,
    // registers
    rs1: u32,
    rs2: u32,
    // program counter
    pc: u32,
    // 20 bit immediate
    imm: u32,
}

impl PCU {
    const NONE: PCU = PCU::new(PCU_T::NONE);

    const BEQ: PCU = PCU::new(PCU_T::BEQ);
    const BNE: PCU = PCU::new(PCU_T::BNE);

    const BLT: PCU = PCU::new(PCU_T::BLT);
    const BGE: PCU = PCU::new(PCU_T::BGE);

    const BLTU: PCU = PCU::new(PCU_T::BLTU);
    const BGEU: PCU = PCU::new(PCU_T::BGEU);

    const JAL: PCU = PCU::new(PCU_T::JAL);
    const JALR: PCU = PCU::new(PCU_T::JALR);

    const fn new(op_type: PCU_T) -> Self {
        Self {
            op_type,
            rs1: 0,
            rs2: 0,
            pc: 0,
            imm: 0,
        }
    }

    fn u_rs1(mut self, rs1: u32) -> Self {
        self.rs1 = rs1;
        self
    }

    fn u_rs2(mut self, rs2: u32) -> Self {
        self.rs2 = rs2;
        self
    }

    fn u_pc(mut self, pc: u32) -> Self {
        self.pc = pc;
        self
    }

    fn u_imm(mut self, imm: u32) -> Self {
        let imm = imm & ((1 << 20) - 1);
        self.imm = imm;
        self
    }

    fn set_rs2_equal_rs1(mut self) -> Self {
        self.rs2 = self.rs1;
        self
    }

    fn set_rs1_lt_rs2(mut self) -> Self {
        if self.rs1 == self.rs2 {
            self.rs1 += 1;
        }
        let tmp = self.rs2;
        self.rs2 = std::cmp::max(self.rs1, self.rs2);
        self.rs1 = std::cmp::min(tmp, self.rs1);
        self
    }

    fn set_rs1_gte_rs2(mut self) -> Self {
        let tmp = self.rs2;
        self.rs2 = std::cmp::min(self.rs1, self.rs2);
        self.rs1 = std::cmp::max(tmp, self.rs1);
        self
    }

    fn set_rs1_lt_rs2_signed(mut self) -> Self {
        if self.rs1 == self.rs2 {
            self.rs1 += 1;
        }
        let tmp = self.rs2 as i32;
        self.rs2 = std::cmp::max(self.rs1 as i32, self.rs2 as i32) as u32;
        self.rs1 = std::cmp::min(tmp, self.rs1 as i32) as u32;
        self
    }

    fn set_rs1_gte_rs2_signed(mut self) -> Self {
        let tmp = self.rs2 as i32;
        self.rs2 = std::cmp::min(self.rs1 as i32, self.rs2 as i32) as u32;
        self.rs1 = std::cmp::max(tmp, self.rs1 as i32) as u32;
        self
    }

    fn expected_update(&self) -> u32 {
        let se_imm: u32 = sign_extend(self.imm, 20);
        let default_case: u32 = self.pc + 4;
        match self.op_type {
            PCU_T::NONE => default_case,
            PCU_T::BEQ => {
                if self.rs1 == self.rs2 {
                    self.pc.wrapping_add(se_imm)
                } else {
                    default_case
                }
            }
            PCU_T::BNE => {
                if self.rs1 != self.rs2 {
                    self.pc.wrapping_add(se_imm)
                } else {
                    default_case
                }
            }
            PCU_T::BLT => {
                if (self.rs1 as i32) < self.rs2 as i32 {
                    self.pc.wrapping_add(se_imm)
                } else {
                    default_case
                }
            }
            PCU_T::BLTU => {
                if self.rs1 < self.rs2 {
                    self.pc.wrapping_add(se_imm)
                } else {
                    default_case
                }
            }
            PCU_T::BGE => {
                if (self.rs1 as i32) >= (self.rs2 as i32) {
                    self.pc.wrapping_add(se_imm)
                } else {
                    default_case
                }
            }
            PCU_T::BGEU => {
                if self.rs1 >= self.rs2 {
                    self.pc.wrapping_add(se_imm)
                } else {
                    default_case
                }
            }
            PCU_T::JAL => self.pc.wrapping_add(se_imm),
            PCU_T::JALR => (self.pc.wrapping_add(se_imm).wrapping_shr(1)).wrapping_shl(1),
        }
    }
}
