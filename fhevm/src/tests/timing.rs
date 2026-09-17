//! Timing probes for the multi-threaded paths, run explicitly with
//! `cargo test --release -p fhevm -- --ignored timing --nocapture`.
use std::time::Instant;

use poulpy_bin_fhe::{
    bdd_arithmetic::{Add, FheUint, FheUintPrepared},
    blind_rotation::{BlindRotationAlgo, CGGI},
};
use poulpy_cpu_avx::FFT64Avx;
use poulpy_hal::layouts::{Module, ScratchOwned};

use crate::{
    backend::{prelude::*, PhantomBackend, PhantomBootstrapModule},
    prepare::PrepareMultiple,
    tests::TestContext,
};

#[test]
#[ignore]
fn timing_fft64_avx() {
    timing::<CGGI, FFT64Avx>("FFT64Avx");
}

fn timing<BRA: BlindRotationAlgo, BE: PhantomBackend>(label: &str)
where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<BRA, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
{
    let mut ctx: TestContext<BRA, BE> = TestContext::new();
    let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 28);
    let a: FheUint<Vec<u8>, u32, i64> = ctx.encrypt_fhe_uint(12345, &mut scratch.borrow());
    let b: FheUint<Vec<u8>, u32, i64> = ctx.encrypt_fhe_uint(67890, &mut scratch.borrow());
    let module = ctx.params.module();
    let mut a_prep: FheUintPrepared<Vec<u8>, u32, BE> =
        FheUintPrepared::alloc_from_infos(module, &ctx.params.fhe_uint_prepared_infos());
    let mut b_prep: FheUintPrepared<Vec<u8>, u32, BE> =
        FheUintPrepared::alloc_from_infos(module, &ctx.params.fhe_uint_prepared_infos());
    let mut res: FheUint<Vec<u8>, u32, i64> =
        FheUint::alloc_from_infos(module, &ctx.params.fhe_uint_infos());

    for &threads in &[1usize, 16] {
        let t = Instant::now();
        a_prep.prepare_custom_multi_thread(
            threads,
            module,
            &a,
            0,
            32,
            &ctx.keys_prepared,
            &mut scratch.borrow(),
        );
        println!(
            "{label} threads={threads} library prepare (32 bits): {:?}",
            t.elapsed()
        );

        let t = Instant::now();
        module.prepare_multiple_fheuint(
            threads,
            &mut vec![&mut a_prep, &mut b_prep],
            vec![&a, &b],
            vec![32, 32],
            &ctx.keys_prepared,
            &mut scratch.borrow(),
        );
        println!(
            "{label} threads={threads} phantom prepare (64 bits): {:?}",
            t.elapsed()
        );

        let t = Instant::now();
        res.add_multi_thread(
            threads,
            module,
            &a_prep,
            &b_prep,
            &ctx.keys_prepared,
            &mut scratch.borrow(),
        );
        println!(
            "{label} threads={threads} library add (32 bits): {:?}",
            t.elapsed()
        );
    }
    assert_eq!(
        res.decrypt(module, &ctx.sk_glwe_prepared, &mut scratch.borrow()),
        12345u32.wrapping_add(67890)
    );
}
