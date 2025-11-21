use std::time::Instant;

use poulpy_core::layouts::{
    prepared::GLWESecretPrepared, GLWEInfos, GLWELayout, GLWESecret, LWESecret,
};
use poulpy_cpu_ref::FFT64Ref;
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::ScratchOwned,
    source::Source,
};

use crate::{
    keys::{VMKeys, VMKeysPrepared},
    memory::Memory,
    parameters::CryptographicParameters,
};

use poulpy_schemes::bin_fhe::{
    bdd_arithmetic::{FheUint, FheUintPrepared},
    blind_rotation::CGGI,
};
use rand_core::RngCore;

#[test]
fn test_fhe_ram() {
    println!("Starting!");

    let threads = 1;

    let seed_xs: [u8; 32] = [0u8; 32];
    let seed_xa: [u8; 32] = [0u8; 32];
    let seed_xe: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);
    let mut source_xa: Source = Source::new(seed_xa);
    let mut source_xe: Source = Source::new(seed_xe);

    // See parameters.rs for configuration
    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc(params.n_glwe(), params.rank());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

    let fhe_uint_infos: &GLWELayout = &params.fhe_uint_infos();

    let keys: VMKeys<Vec<u8>, CGGI> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 24);

    let mut sk_prep: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc(params.module(), sk_glwe.rank());
    sk_prep.prepare(params.module(), &sk_glwe);

    let mut keys_prepared: VMKeysPrepared<Vec<u8>, CGGI, FFT64Ref> = VMKeysPrepared::alloc(&params);
    keys_prepared.prepare(params.module(), &keys, scratch.borrow());

    // Some deterministic randomness
    let mut source: Source = Source::new([5u8; 32]);

    // Word-size
    let word_size: usize = 32;
    let size: usize = 1024;

    let mask: u32 = ((1u64 << word_size) - 1) as u32;

    // Instantiates the FHE-RAM
    let mut ram: Memory = Memory::alloc(&params.ram_infos(), word_size, size);

    // Allocates some dummy data
    let mut data: Vec<u32> = vec![0u32; ram.size()];
    for (i, x) in data.iter_mut().enumerate() {
        *x = i as u32 //source.next_u32();
    }

    // Populates the FHE-RAM
    ram.encrypt_sk(
        params.module(),
        &data,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    // Allocates an encrypted address.
    let mut addr: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::alloc_from_infos(params.module(), &params.fhe_uint_prepared_infos());

    // Random index
    let idx: u32 = 512;

    // Encrypts random index
    addr.encrypt_sk(
        params.module(),
        idx,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    // Reads from the FHE-RAM
    let start: Instant = Instant::now();
    let mut res: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(fhe_uint_infos);
    ram.read_stateless(
        threads,
        params.module(),
        &mut res,
        &addr,
        0,
        &keys_prepared,
        scratch.borrow(),
    );
    let duration: std::time::Duration = start.elapsed();
    println!("READ Elapsed time: {} ms", duration.as_millis());

    // Check noise & correctness
    assert_eq!(
        data[idx as usize],
        res.decrypt(params.module(), &sk_prep, scratch.borrow())
    );
    let noise = res
        .noise(
            params.module(),
            data[idx as usize],
            &sk_prep,
            scratch.borrow(),
        )
        .std()
        .log2();
    assert!(noise < -16.0, "{noise} > -16");

    let start: Instant = Instant::now();
    ram.read_statefull(
        threads,
        params.module(),
        &mut res,
        &addr,
        0,
        &keys_prepared,
        scratch.borrow(),
    );

    let duration: std::time::Duration = start.elapsed();
    println!(
        "READ_PREPARE_WRITE Elapsed time: {} ms",
        duration.as_millis()
    );

    // Check noise & correctness
    assert_eq!(
        data[idx as usize],
        res.decrypt(params.module(), &sk_prep, scratch.borrow())
    );
    let noise = res
        .noise(
            params.module(),
            data[idx as usize],
            &sk_prep,
            scratch.borrow(),
        )
        .std()
        .log2();
    assert!(noise < -16.0, "{noise} > -16");

    // Value to write on the FHE-RAM
    let value: u32 = source.next_u32() & mask;

    // Encryptes value to write on the FHE-RAM
    let mut ct_write: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(fhe_uint_infos);
    ct_write.encrypt_sk(
        params.module(),
        value,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    // Updates plaintext ram
    data[idx as usize] = value;

    // Writes on the FHE-RAM
    let start: Instant = Instant::now();
    ram.read_statefull_rev(
        threads,
        params.module(),
        &ct_write,
        &addr,
        0,
        &keys_prepared,
        scratch.borrow(),
    );
    let duration: std::time::Duration = start.elapsed();
    println!("WRITE Elapsed time: {} ms", duration.as_millis());

    // Reads back at the written index
    ram.read_stateless(
        threads,
        params.module(),
        &mut res,
        &addr,
        0,
        &keys_prepared,
        scratch.borrow(),
    );

    // Checks correctness & noise
    assert_eq!(
        data[idx as usize],
        res.decrypt(params.module(), &sk_prep, scratch.borrow())
    );
    let noise = res
        .noise(
            params.module(),
            data[idx as usize],
            &sk_prep,
            scratch.borrow(),
        )
        .std()
        .log2();

    assert!(noise < -16.0, "{noise} > -16");

    let mut ram_decrypted: Vec<u32> = vec![0u32; ram.size()];
    ram.decrypt(
        params.module(),
        ram_decrypted.as_mut_slice(),
        &sk_prep,
        scratch.borrow(),
    );

    assert_eq!(
        ram_decrypted[idx as usize],
        res.decrypt(params.module(), &sk_prep, scratch.borrow())
    );
}
