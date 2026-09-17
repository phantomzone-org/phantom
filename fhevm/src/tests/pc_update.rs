use poulpy_bin_fhe::{
    bdd_arithmetic::{FheUint, FheUintPrepared},
    blind_rotation::{BlindRotationAlgo, CGGI},
};
use poulpy_cpu_ref::FFT64Ref;
use poulpy_hal::layouts::{Module, ScratchOwned};
use rand_core::Rng;

use crate::{
    backend::{prelude::*, PhantomBackend, PhantomBootstrapModule},
    tests::TestContext,
    update_pc, PCU,
};

#[test]
fn test_pc_update_fft64_ref() {
    test_pc_update::<CGGI, FFT64Ref>()
}

fn test_pc_update<BRA: BlindRotationAlgo, BE: PhantomBackend>()
where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
{
    let threads = 2;

    let mut ctx: TestContext<BRA, BE> = TestContext::new();
    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    let mut pc: FheUint<Vec<u8>, u32, i64> =
        FheUint::alloc_from_infos(ctx.params.module(), &ctx.params.fhe_uint_infos());

    let mut rng: poulpy_hal::source::Source = poulpy_hal::source::Source::new([7u8; 32]);

    [
        PCU::BEQ
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs2_equal_rs1(),
        PCU::BEQ
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs1_lt_rs2(),
        PCU::BNE
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs1_lt_rs2(),
        PCU::BNE
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs2_equal_rs1(),
        PCU::BLTU
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs1_lt_rs2(),
        PCU::BLTU
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs1_gte_rs2(),
        PCU::BGEU
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs1_gte_rs2(),
        PCU::BGEU
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs1_lt_rs2(),
        PCU::BLT
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs1_lt_rs2_signed(),
        PCU::BLT
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs1_gte_rs2_signed(),
        PCU::BGE
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs1_gte_rs2_signed(),
        PCU::BGE
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32())
            .set_rs1_lt_rs2_signed(),
        PCU::JAL
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32()),
        PCU::JALR
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32()),
        PCU::NONE
            .u_pc(rng.next_u32())
            .u_imm(rng.next_u32())
            .u_rs1(rng.next_u32())
            .u_rs2(rng.next_u32()),
    ]
    .iter()
    .for_each(|pcu| {
        let rs1_prep: FheUintPrepared<Vec<u8>, u32, BE> =
            ctx.encrypt_fhe_uint_prepared(pcu.rs1, &mut scratch.borrow());
        let rs2_prep: FheUintPrepared<Vec<u8>, u32, BE> =
            ctx.encrypt_fhe_uint_prepared(pcu.rs2, &mut scratch.borrow());
        let imm_prep: FheUintPrepared<Vec<u8>, u32, BE> =
            ctx.encrypt_fhe_uint_prepared(pcu.imm, &mut scratch.borrow());
        let pc_prep: FheUintPrepared<Vec<u8>, u32, BE> =
            ctx.encrypt_fhe_uint_prepared(pcu.pc, &mut scratch.borrow());
        let pc_id: FheUintPrepared<Vec<u8>, u32, BE> =
            ctx.encrypt_fhe_uint_prepared(pcu.op_type as u32, &mut scratch.borrow());
        update_pc(
            threads,
            ctx.params.module(),
            &mut pc,
            &rs1_prep,
            &rs2_prep,
            &pc_prep,
            &imm_prep,
            &pc_id,
            &ctx.keys_prepared,
            &mut scratch.borrow(),
        );
        let have: u32 = pc.decrypt(
            ctx.params.module(),
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow(),
        );
        println!("{} {}", pcu.expected_update(), have);
        assert_eq!(pcu.expected_update(), have);
    });
}
