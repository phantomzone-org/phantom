use std::time::Instant;

use poulpy_bin_fhe::{
    bdd_arithmetic::{FheUint, FheUintPrepared},
    blind_rotation::CGGI,
};
use poulpy_core::layouts::GLWELayout;
use poulpy_cpu_ref::FFT64Ref;
use poulpy_hal::{layouts::ScratchOwned, source::Source};
use rand_core::Rng;

use crate::{backend::prelude::*, memory::Memory, tests::TestContext};

#[test]
fn test_fhe_ram() {
    println!("Starting!");

    let threads = 1;

    let mut ctx: TestContext<CGGI, FFT64Ref> = TestContext::new();
    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 24);

    let fhe_uint_infos: GLWELayout = ctx.params.fhe_uint_infos();

    // Some deterministic randomness
    let mut source: Source = Source::new([5u8; 32]);

    // Word-size
    let word_size: usize = 32;
    let size: usize = 1024;

    let mask: u32 = ((1u64 << word_size) - 1) as u32;

    // Instantiates the FHE-RAM
    let mut ram: Memory = Memory::alloc(
        ctx.params.module(),
        &ctx.params.ram_infos(),
        word_size,
        size,
    );

    // Allocates some dummy data
    let mut data: Vec<u32> = vec![0u32; ram.size()];
    for (i, x) in data.iter_mut().enumerate() {
        *x = i as u32 //source.next_u32();
    }

    // Populates the FHE-RAM
    ram.encrypt_sk(
        ctx.params.module(),
        &data,
        &ctx.sk_glwe_prepared,
        &mut ctx.source_xa,
        &mut ctx.source_xe,
        &mut scratch.borrow(),
    );

    // Random index
    let idx: u32 = 512;

    // Encrypts the index
    let addr: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        ctx.encrypt_fhe_uint_prepared(idx, &mut scratch.borrow());

    // Reads from the FHE-RAM
    let start: Instant = Instant::now();
    let mut res: FheUint<Vec<u8>, u32, i64> =
        FheUint::alloc_from_infos(ctx.params.module(), &fhe_uint_infos);
    ram.read_stateless(
        threads,
        ctx.params.module(),
        &mut res,
        &addr,
        0,
        &ctx.keys_prepared,
        &mut scratch.borrow(),
    );
    let duration: std::time::Duration = start.elapsed();
    println!("READ Elapsed time: {} ms", duration.as_millis());

    // Check noise & correctness
    assert_eq!(
        data[idx as usize],
        res.decrypt(
            ctx.params.module(),
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow()
        )
    );
    let noise = res
        .noise(
            ctx.params.module(),
            data[idx as usize],
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow(),
        )
        .std()
        .log2();
    assert!(noise < -16.0, "{noise} > -16");

    let start: Instant = Instant::now();
    ram.read_statefull(
        threads,
        ctx.params.module(),
        &mut res,
        &addr,
        0,
        &ctx.keys_prepared,
        &mut scratch.borrow(),
    );

    let duration: std::time::Duration = start.elapsed();
    println!(
        "READ_PREPARE_WRITE Elapsed time: {} ms",
        duration.as_millis()
    );

    // Check noise & correctness
    assert_eq!(
        data[idx as usize],
        res.decrypt(
            ctx.params.module(),
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow()
        )
    );
    let noise = res
        .noise(
            ctx.params.module(),
            data[idx as usize],
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow(),
        )
        .std()
        .log2();
    assert!(noise < -16.0, "{noise} > -16");

    // Value to write on the FHE-RAM
    let value: u32 = source.next_u32() & mask;

    // Encryptes value to write on the FHE-RAM
    let ct_write: FheUint<Vec<u8>, u32, i64> = ctx.encrypt_fhe_uint(value, &mut scratch.borrow());

    // Updates plaintext ram
    data[idx as usize] = value;

    // Writes on the FHE-RAM
    let start: Instant = Instant::now();
    ram.read_statefull_rev(
        threads,
        ctx.params.module(),
        &ct_write,
        &addr,
        0,
        &ctx.keys_prepared,
        &mut scratch.borrow(),
    );
    let duration: std::time::Duration = start.elapsed();
    println!("WRITE Elapsed time: {} ms", duration.as_millis());

    // Reads back at the written index
    ram.read_stateless(
        threads,
        ctx.params.module(),
        &mut res,
        &addr,
        0,
        &ctx.keys_prepared,
        &mut scratch.borrow(),
    );

    // Checks correctness & noise
    assert_eq!(
        data[idx as usize],
        res.decrypt(
            ctx.params.module(),
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow()
        )
    );
    let noise = res
        .noise(
            ctx.params.module(),
            data[idx as usize],
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow(),
        )
        .std()
        .log2();

    assert!(noise < -16.0, "{noise} > -16");

    let mut ram_decrypted: Vec<u32> = vec![0u32; ram.size()];
    ram.decrypt(
        ctx.params.module(),
        ram_decrypted.as_mut_slice(),
        &ctx.sk_glwe_prepared,
        &mut scratch.borrow(),
    );

    assert_eq!(
        ram_decrypted[idx as usize],
        res.decrypt(
            ctx.params.module(),
            &ctx.sk_glwe_prepared,
            &mut scratch.borrow()
        )
    );
}
