use std::time::Instant;

use itertools::Itertools;
use poulpy_backend::FFT64Ref;
use poulpy_core::{
    layouts::{
        prepared::GLWESecretPrepared, GLWEInfos, GLWELayout, GLWEPlaintext, GLWESecret, LWEInfos,
        GLWE,
    },
    GLWEDecrypt, GLWEEncryptSk, ScratchTakeCore,
};
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow, ScratchTakeBasic},
    layouts::{Backend, Module, Scratch, ScratchOwned},
    source::Source,
};

use fhevm::{
    keys::{RAMKeys, RAMKeysPrepared},
    parameters::{CryptographicParameters, DECOMP_N},
    Address, Ram,
};

use poulpy_schemes::tfhe::bdd_arithmetic::{FheUint, ToBits};
use rand_core::RngCore;

#[test]
fn test_fhe_ram() {
    println!("Starting!");

    let seed_xs: [u8; 32] = [0u8; 32];
    let seed_xa: [u8; 32] = [0u8; 32];
    let seed_xe: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);
    let mut source_xa: Source = Source::new(seed_xa);
    let mut source_xe: Source = Source::new(seed_xe);

    // See parameters.rs for configuration
    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk.fill_ternary_prob(0.5, &mut source_xs);

    let keys: RAMKeys<Vec<u8>> = RAMKeys::encrypt_sk(&params, &sk, &mut source_xa, &mut source_xe);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 24);

    let mut sk_prep: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc(params.module(), sk.rank());
    sk_prep.prepare(params.module(), &sk);

    let mut keys_prepared: RAMKeysPrepared<Vec<u8>, FFT64Ref> = RAMKeysPrepared::alloc(&params);
    keys_prepared.prepare(params.module(), &keys, scratch.borrow());

    // Some deterministic randomness
    let mut source: Source = Source::new([5u8; 32]);

    // Word-size
    let word_size: usize = 32;
    let max_addr: usize = 1 << 6;
    let decomp_n: Vec<u8> = DECOMP_N.into();

    let mask: u32 = ((1u64 << word_size) - 1) as u32;

    // Instantiates the FHE-RAM
    let mut ram: Ram = Ram::new(&params, word_size, &decomp_n, max_addr);

    // Allocates some dummy data
    let mut data: Vec<u32> = vec![0u32; ram.max_addr()];
    for i in data.iter_mut() {
        *i = source.next_u32() & mask;
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
    let mut addr: Address<Vec<u8>> = Address::alloc_from_params(&params, ram.base_2d());

    // Random index
    let idx: u32 = source.next_u32() % max_addr as u32;

    // Encrypts random index
    addr.encrypt_sk(
        &params,
        idx,
        &sk,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    // Reads from the FHE-RAM
    let start: Instant = Instant::now();
    let ct: Vec<GLWE<Vec<u8>>> = ram.read(params.module(), &addr, &keys_prepared, scratch.borrow());
    let duration: std::time::Duration = start.elapsed();
    println!("READ Elapsed time: {} ms", duration.as_millis());

    let want = data[idx as usize];
    // Checks correctness
    for i in 0..word_size {
        let bit = want.bit(i) as i64;
        let (decrypted_value, noise) = decrypt_glwe(&params, &ct[i], bit, &sk_prep);
        assert_eq!(decrypted_value, bit);
        println!("noise: {}", noise);
        assert!(
            noise < -(params.k_glwe_pt().as_usize() as f64 + 1.0),
            "{} >= {}",
            noise,
            (params.k_glwe_pt().as_usize() as f64 + 1.0)
        );
    }

    let start: Instant = Instant::now();
    let ct: Vec<GLWE<Vec<u8>>> =
        ram.read_prepare_write(params.module(), &addr, &keys_prepared, scratch.borrow());
    let duration: std::time::Duration = start.elapsed();
    println!(
        "READ_PREPARE_WRITE Elapsed time: {} ms",
        duration.as_millis()
    );

    // Checks correctness
    for i in 0..word_size {
        let bit: i64 = want.bit(i) as i64;
        let (decrypted_value, noise) = decrypt_glwe(&params, &ct[i], bit, &sk_prep);
        assert_eq!(decrypted_value, bit);
        println!("noise: {}", noise);
        assert!(
            noise < -(params.k_glwe_pt().as_usize() as f64 + 1.0),
            "{} >= {}",
            noise,
            (params.k_glwe_pt().as_usize() as f64 + 1.0)
        );
    }

    // Value to write on the FHE-RAM
    let value: u32 = source.next_u32() & mask;

    // Encryptes value to write on the FHE-RAM
    let ct_w: Vec<GLWE<Vec<u8>>> = (0..word_size)
        .map(|i| encrypt_glwe(&params, value.bit(i), &sk_prep))
        .collect_vec();

    // Updates plaintext ram
    data[idx as usize] = value;

    // Writes on the FHE-RAM
    let start: Instant = Instant::now();
    ram.write(
        params.module(),
        &ct_w,
        &addr,
        &keys_prepared,
        scratch.borrow(),
    );
    let duration: std::time::Duration = start.elapsed();
    println!("WRITE Elapsed time: {} ms", duration.as_millis());

    // Reads back at the written index
    let ct: Vec<GLWE<Vec<u8>>> = ram.read(params.module(), &addr, &keys_prepared, scratch.borrow());

    // Checks correctness
    for i in 0..word_size {
        let bit: i64 = value.bit(i) as i64;
        let (decrypted_value, noise) = decrypt_glwe(&params, &ct[i], bit, &sk_prep);
        assert_eq!(decrypted_value, bit);
        println!("noise: {}", noise);
        assert!(
            noise < -(params.k_glwe_pt().as_usize() as f64 + 1.0),
            "{} >= {}",
            noise,
            (params.k_glwe_pt().as_usize() as f64 + 1.0)
        );
    }
}

#[test]
fn test_fhe_ram_read_to_fheuint() {
    println!("Starting!");

    let seed_xs: [u8; 32] = [0u8; 32];
    let seed_xa: [u8; 32] = [0u8; 32];
    let seed_xe: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);
    let mut source_xa: Source = Source::new(seed_xa);
    let mut source_xe: Source = Source::new(seed_xe);

    // See parameters.rs for configuration
    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk.fill_ternary_prob(0.5, &mut source_xs);

    let keys: RAMKeys<Vec<u8>> = RAMKeys::encrypt_sk(&params, &sk, &mut source_xa, &mut source_xe);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 24);

    let mut sk_prep: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc(params.module(), sk.rank());
    sk_prep.prepare(params.module(), &sk);

    let mut keys_prepared: RAMKeysPrepared<Vec<u8>, FFT64Ref> = RAMKeysPrepared::alloc(&params);
    keys_prepared.prepare(params.module(), &keys, scratch.borrow());

    // Some deterministic randomness
    let mut source: Source = Source::new([5u8; 32]);

    // Word-size
    let word_size: usize = 32;
    let max_addr: usize = 1 << 6;
    let decomp_n: Vec<u8> = DECOMP_N.into();

    let mask: u32 = ((1u64 << word_size) - 1) as u32;

    // Instantiates the FHE-RAM
    let mut ram: Ram = Ram::new(&params, word_size, &decomp_n, max_addr);

    // Allocates some dummy data
    let mut data: Vec<u32> = vec![0u32; ram.max_addr()];
    for i in data.iter_mut() {
        *i = source.next_u32() & mask;
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
    let mut addr: Address<Vec<u8>> = Address::alloc_from_params(&params, ram.base_2d());

    // Random index
    let idx: u32 = 0;

    // Encrypts random index
    addr.encrypt_sk(
        &params,
        idx,
        &sk,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    // Reads from the FHE-RAM
    let start: Instant = Instant::now();
    let ct: Vec<GLWE<Vec<u8>>> = ram.read(params.module(), &addr, &keys_prepared, scratch.borrow());
    let duration: std::time::Duration = start.elapsed();
    println!("READ Elapsed time: {} ms", duration.as_millis());

    let want = data[idx as usize];
    // Checks correctness
    for i in 0..word_size {
        let bit = want.bit(i) as i64;
        let (decrypted_value, noise) = decrypt_glwe(&params, &ct[i], bit, &sk_prep);
        assert_eq!(decrypted_value, bit);
        println!("noise: {}", noise);
        assert!(
            noise < -(params.k_glwe_pt().as_usize() as f64 + 1.0),
            "{} >= {}",
            noise,
            (params.k_glwe_pt().as_usize() as f64 + 1.0)
        );
    }

    let mut res: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&params.glwe_ct_infos());
    res.pack(params.module(), ct, &keys_prepared, scratch.borrow());

    let decrypted_value: u32 = res.decrypt(params.module(), &sk_prep, scratch.borrow());
    assert_eq!(decrypted_value, want);
}

#[test]
fn test_fheuint_pack() {
    let seed_xs: [u8; 32] = [0u8; 32];
    let seed_xa: [u8; 32] = [0u8; 32];
    let seed_xe: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);
    let mut source_xa: Source = Source::new(seed_xa);
    let mut source_xe: Source = Source::new(seed_xe);

    // See parameters.rs for configuration
    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk.fill_ternary_prob(0.5, &mut source_xs);

    let keys: RAMKeys<Vec<u8>> = RAMKeys::encrypt_sk(&params, &sk, &mut source_xa, &mut source_xe);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 24);

    let mut sk_prep: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc(params.module(), sk.rank());
    sk_prep.prepare(params.module(), &sk);

    let mut keys_prepared: RAMKeysPrepared<Vec<u8>, FFT64Ref> = RAMKeysPrepared::alloc(&params);
    keys_prepared.prepare(params.module(), &keys, scratch.borrow());

    let mut ct: Vec<GLWE<Vec<u8>>> = Vec::new();
    let value: u32 = 1;
    for i in 0..32 {
        ct.push(encrypt_glwe(&params, value.bit(i), &sk_prep));
    }

    let mut res: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&params.glwe_ct_infos());
    res.pack(params.module(), ct, &keys_prepared, scratch.borrow());

    let decrypted_value: u32 = res.decrypt(params.module(), &sk_prep, scratch.borrow());
    assert_eq!(decrypted_value, value);
}

fn encrypt_glwe<B: Backend>(
    params: &CryptographicParameters<B>,
    value: u8,
    sk: &GLWESecretPrepared<Vec<u8>, B>,
) -> GLWE<Vec<u8>>
where
    ScratchOwned<B>: ScratchOwnedAlloc<B> + ScratchOwnedBorrow<B>,
    Module<B>: GLWEEncryptSk<B>,
    Scratch<B>: ScratchTakeCore<B>,
{
    let module: &Module<B> = params.module();

    let glwe_infos: GLWELayout = params.glwe_ct_infos();
    let pt_infos: GLWELayout = params.glwe_pt_infos();

    let mut ct_w: GLWE<Vec<u8>> = GLWE::alloc_from_infos(&glwe_infos);
    let mut pt_w: GLWEPlaintext<Vec<u8>> = GLWEPlaintext::alloc_from_infos(&pt_infos);
    pt_w.encode_coeff_i64(value as i64, pt_infos.k(), 0);
    let mut scratch: ScratchOwned<B> =
        ScratchOwned::alloc(GLWE::encrypt_sk_tmp_bytes(module, &glwe_infos));
    let mut source_xa: Source = Source::new([1u8; 32]); // TODO: Create from random seed
    let mut source_xe: Source = Source::new([1u8; 32]); // TODO: Create from random seed
    ct_w.encrypt_sk(
        module,
        &pt_w,
        sk,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );
    ct_w
}

fn decrypt_glwe<B: Backend>(
    params: &CryptographicParameters<B>,
    ct: &GLWE<Vec<u8>>,
    want: i64,
    sk: &GLWESecretPrepared<Vec<u8>, B>,
) -> (i64, f64)
where
    ScratchOwned<B>: ScratchOwnedAlloc<B> + ScratchOwnedBorrow<B>,
    Module<B>: GLWEDecrypt<B>,
    Scratch<B>: ScratchTakeBasic,
{
    let module: &Module<B> = params.module();

    let mut pt: GLWEPlaintext<Vec<u8>> = GLWEPlaintext::alloc_from_infos(ct);
    let mut scratch: ScratchOwned<B> = ScratchOwned::alloc(GLWE::decrypt_tmp_bytes(module, ct));

    ct.decrypt(module, &mut pt, sk, scratch.borrow());

    let log_scale: usize = pt.k().as_usize() - params.k_glwe_pt().as_usize();
    let decrypted_value_before_scale: i64 = pt.decode_coeff_i64(pt.k(), 0);

    let diff: i64 = decrypted_value_before_scale - (want << log_scale);
    let noise: f64 = (diff.abs() as f64).log2() - pt.k().as_usize() as f64;
    let decrypted_value: i64 =
        (decrypted_value_before_scale as f64 / f64::exp2(log_scale as f64)).round() as i64;
    (decrypted_value, noise)
}
