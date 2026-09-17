use poulpy_bin_fhe::{
    bdd_arithmetic::{FheUint, FheUintPrepared},
    blind_rotation::{BlindRotationAlgo, CGGI},
};
use poulpy_cpu_ref::FFT64Ref;
use poulpy_hal::layouts::{Module, ScratchOwned};

use crate::{
    backend::{prelude::*, PhantomBackend, PhantomBootstrapModule},
    ram_offset::ram_offset,
    tests::TestContext,
};

#[test]
fn test_ram_offset_fft64_ref() {
    test_ram_offset::<CGGI, FFT64Ref>()
}

fn test_ram_offset<BRA: BlindRotationAlgo, BE: PhantomBackend>()
where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
{
    let threads = 4;

    let mut ctx: TestContext<BRA, BE> = TestContext::new();
    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

    let rs1: u32 = 0x00040fe0;
    let imm: u32 = 0x0000001c;
    let offset: u32 = 1 << 18;

    let rs1_prep: FheUintPrepared<Vec<u8>, u32, BE> =
        ctx.encrypt_fhe_uint_prepared(rs1, &mut scratch.borrow());
    let imm_prep: FheUintPrepared<Vec<u8>, u32, BE> =
        ctx.encrypt_fhe_uint_prepared(imm, &mut scratch.borrow());
    let mut ram: FheUint<Vec<u8>, u32, i64> =
        FheUint::alloc_from_infos(ctx.params.module(), &ctx.params.fhe_uint_infos());

    ram_offset(
        threads,
        ctx.params.module(),
        &mut ram,
        &rs1_prep,
        &imm_prep,
        &ctx.keys_prepared,
        &mut scratch.borrow(),
    );

    assert_eq!(
        rs1.wrapping_add(imm).wrapping_sub(offset),
        ram.decrypt(
            ctx.params.module(),
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow()
        )
    );

    println!(
        "rs1.wrapping_add(imm).wrapping_sub(offset): {}",
        rs1.wrapping_add(imm).wrapping_sub(offset)
    );
}
