use std::time::Instant;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
use poulpy_backend::FFT64Avx as BackendImpl;

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
use poulpy_backend::FFT64Ref as BackendImpl;

use poulpy_core::{
    layouts::{
        prepared::GLWESecretPrepared, GLWEInfos, GLWELayout, GLWEPlaintext, GLWESecret, LWEInfos,
        LWESecret, GLWE,
    },
    GLWEDecrypt, GLWEEncryptSk, ScratchTakeCore,
};
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow, ScratchTakeBasic},
    layouts::{Backend, Module, Scratch, ScratchOwned},
    source::Source,
};

use fhevm::{
    Address, CryptographicParameters, EvaluationKeys, EvaluationKeysPrepared, Parameters, Ram,
    TEST_BDD_KEY_LAYOUT,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{BDDKey, BDDKeyPrepared, FheUint},
    blind_rotation::CGGI,
};
use rand_core::RngCore;

fn cast_u8_to_signed(value: u8, bit_length: usize) -> i64 {
    assert!(
        (1..=8).contains(&bit_length),
        "bit_length must be between 1 and 8"
    );
    let shift = 8 - bit_length;
    ((value << shift) as i8 as i64) >> shift
}

fn encrypt_glwe<B: Backend>(
    params: &Parameters<B>,
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
    params: &Parameters<B>,
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

#[test]
fn test_fhe_ram_read() {
    println!("Starting!");

    let seed_xs: [u8; 32] = [0u8; 32];
    let seed_xa: [u8; 32] = [0u8; 32];
    let seed_xe: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);
    let mut source_xa: Source = Source::new(seed_xa);
    let mut source_xe: Source = Source::new(seed_xe);

    // See parameters.rs for configuration
    let params: Parameters<BackendImpl> = Parameters::<BackendImpl>::new();

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk.fill_ternary_prob(0.5, &mut source_xs);

    let keys: EvaluationKeys<Vec<u8>> =
        EvaluationKeys::encrypt_sk(&params, &sk, &mut source_xa, &mut source_xe);

    let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 24);

    let mut sk_prep: GLWESecretPrepared<Vec<u8>, BackendImpl> =
        GLWESecretPrepared::alloc(params.module(), sk.rank());
    sk_prep.prepare(params.module(), &sk);

    let mut keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl> =
        EvaluationKeysPrepared::alloc(&params);
    keys_prepared.prepare(params.module(), &keys, scratch.borrow());

    // Some deterministic randomness
    let mut source: Source = Source::new([5u8; 32]);

    // Word-size
    let ws: usize = params.word_size();

    // Allocates some dummy data
    let mut data: Vec<u8> = vec![0u8; params.max_addr() * ws];
    source.fill_bytes(data.as_mut_slice());

    // Instantiates the FHE-RAM
    let mut ram: Ram<BackendImpl> = Ram::new();

    // Populates the FHE-RAM
    ram.encrypt_sk(&data, &sk, &mut source_xa, &mut source_xe);

    // Allocates an encrypted address.
    let mut addr: Address<Vec<u8>> = Address::alloc_from_params(&params);

    // Random index
    let idx: u32 = source.next_u32() % params.max_addr() as u32;

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
    let ct: Vec<GLWE<Vec<u8>>> = ram.read(&addr, &keys_prepared);
    let duration: std::time::Duration = start.elapsed();
    println!("READ Elapsed time: {} ms", duration.as_millis());

    // Checks correctness
    (0..ws).for_each(|i| {
        let want = cast_u8_to_signed(data[i + ws * idx as usize], params.k_glwe_pt().as_usize()); // 8-bit signed integer
        let (decrypted_value, noise) = decrypt_glwe(&params, &ct[i], want, &sk_prep);
        assert_eq!(decrypted_value, want);
        println!("noise: {}", noise);
        assert!(
            noise < -(params.k_glwe_pt().as_usize() as f64 + 1.0),
            "{} >= {}",
            noise,
            (params.k_glwe_pt().as_usize() as f64 + 1.0)
        );
    });
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
    // let params: Parameters<BackendImpl> = Parameters::<BackendImpl>::new();
    let params = Parameters {
        cryptographic_parameters: CryptographicParameters::<BackendImpl>::new(),
        max_addr: 1 << 11,
        decomp_n: vec![3, 3, 3, 3],
        word_size: 32,
    };

    let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 24);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    // sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    sk_glwe.fill_zero();

    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.module().n().into());
    // sk_lwe.fill_binary_block(8, &mut source_xs);
    sk_lwe.fill_zero();

    let mut bdd_key: BDDKey<Vec<u8>, CGGI> = BDDKey::alloc_from_infos(&TEST_BDD_KEY_LAYOUT);
    bdd_key.encrypt_sk(
        params.module(),
        &sk_lwe,
        &sk_glwe,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let mut bdd_key_prepared: BDDKeyPrepared<Vec<u8>, CGGI, BackendImpl> =
        BDDKeyPrepared::alloc_from_infos(params.module(), &TEST_BDD_KEY_LAYOUT);
    bdd_key_prepared.prepare(params.module(), &bdd_key, scratch.borrow());

    let keys: EvaluationKeys<Vec<u8>> =
        EvaluationKeys::encrypt_sk(&params, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BackendImpl> =
        GLWESecretPrepared::alloc(params.module(), sk_glwe.rank());
    sk_glwe_prepared.prepare(params.module(), &sk_glwe);

    let mut keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl> =
        EvaluationKeysPrepared::alloc(&params);
    keys_prepared.prepare(params.module(), &keys, scratch.borrow());

    // Some deterministic randomness
    let mut source: Source = Source::new([5u8; 32]);

    // Word-size
    let ws: usize = params.word_size();

    // Allocates some dummy data
    let data: Vec<u8> = (0..params.max_addr() * ws)
        .map(|_| if source.next_u32() & 1 == 0 { 0u8 } else { 1u8 })
        .collect();

    // Instantiates the FHE-RAM
    let mut ram: Ram<BackendImpl> = Ram::new();

    // Populates the FHE-RAM
    ram.encrypt_sk(&data, &sk_glwe, &mut source_xa, &mut source_xe);

    // Allocates an encrypted address.
    let mut addr: Address<Vec<u8>> = Address::alloc_from_params(&params);

    // Random index
    let idx: u32 = source.next_u32() % params.max_addr() as u32;

    // Encrypts random index
    addr.encrypt_sk(
        &params,
        idx,
        &sk_glwe,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    // Reads from the FHE-RAM
    let start: Instant = Instant::now();
    let ct_fheuint: FheUint<Vec<u8>, u32> =
        ram.read_to_fheuint(&addr, &keys_prepared, &bdd_key_prepared);
    let duration: std::time::Duration = start.elapsed();
    println!("READ Elapsed time: {} ms", duration.as_millis());

    let decrypted_value: u32 =
        ct_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    // println!("decrypted_value: {}", decrypted_value);
    // println!("should be: {:?}", &data[idx as usize * ws..(idx as usize + 1) * ws]);

    // TODO: put the correct assertion
}

#[test]
fn test_fhe_ram_read_prepare_write() {
    println!("Starting!");

    let seed_xs: [u8; 32] = [0u8; 32];
    let seed_xa: [u8; 32] = [0u8; 32];
    let seed_xe: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);
    let mut source_xa: Source = Source::new(seed_xa);
    let mut source_xe: Source = Source::new(seed_xe);

    // See parameters.rs for configuration
    let params: Parameters<BackendImpl> = Parameters::<BackendImpl>::new();

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk.fill_ternary_prob(0.5, &mut source_xs);

    let keys: EvaluationKeys<Vec<u8>> =
        EvaluationKeys::encrypt_sk(&params, &sk, &mut source_xa, &mut source_xe);

    let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 24);

    let mut sk_prep: GLWESecretPrepared<Vec<u8>, BackendImpl> =
        GLWESecretPrepared::alloc(params.module(), sk.rank());
    sk_prep.prepare(params.module(), &sk);

    let mut keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl> =
        EvaluationKeysPrepared::alloc(&params);
    keys_prepared.prepare(params.module(), &keys, scratch.borrow());

    // Some deterministic randomness
    let mut source: Source = Source::new([5u8; 32]);

    // Word-size
    let ws: usize = params.word_size();

    // Allocates some dummy data
    let mut data: Vec<u8> = vec![0u8; params.max_addr() * ws];
    source.fill_bytes(data.as_mut_slice());

    // Instantiates the FHE-RAM
    let mut ram: Ram<BackendImpl> = Ram::new();

    // Populates the FHE-RAM
    ram.encrypt_sk(&data, &sk, &mut source_xa, &mut source_xe);

    // Allocates an encrypted address.
    let mut addr: Address<Vec<u8>> = Address::alloc_from_params(&params);

    // Random index
    let idx: u32 = source.next_u32() % params.max_addr() as u32;

    // Encrypts random index
    addr.encrypt_sk(
        &params,
        idx,
        &sk,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    // Reads from the FHE-RAM (with preparing for write)
    let start: Instant = Instant::now();
    let ct: Vec<GLWE<Vec<u8>>> = ram.read_prepare_write(&addr, &keys_prepared);
    let duration: std::time::Duration = start.elapsed();
    println!(
        "READ_PREPARE_WRITE Elapsed time: {} ms",
        duration.as_millis()
    );

    // Checks correctness
    (0..ws).for_each(|i| {
        let want = cast_u8_to_signed(data[i + ws * idx as usize], params.k_glwe_pt().as_usize()); // 8-bit signed integer
        let (decrypted_value, noise) = decrypt_glwe(&params, &ct[i], want, &sk_prep);
        assert_eq!(decrypted_value, want);
        println!("noise: {}", noise);
        assert!(
            noise < -(params.k_glwe_pt().as_usize() as f64 + 1.0),
            "{} >= {}",
            noise,
            (params.k_glwe_pt().as_usize() as f64 + 1.0)
        );
    });
}

#[test]
fn test_fhe_ram_write() {
    println!("Starting!");

    let seed_xs: [u8; 32] = [0u8; 32];
    let seed_xa: [u8; 32] = [0u8; 32];
    let seed_xe: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);
    let mut source_xa: Source = Source::new(seed_xa);
    let mut source_xe: Source = Source::new(seed_xe);

    // See parameters.rs for configuration
    let params: Parameters<BackendImpl> = Parameters::<BackendImpl>::new();

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk.fill_ternary_prob(0.5, &mut source_xs);

    let keys: EvaluationKeys<Vec<u8>> =
        EvaluationKeys::encrypt_sk(&params, &sk, &mut source_xa, &mut source_xe);

    let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 24);

    let mut sk_prep: GLWESecretPrepared<Vec<u8>, BackendImpl> =
        GLWESecretPrepared::alloc(params.module(), sk.rank());
    sk_prep.prepare(params.module(), &sk);

    let mut keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl> =
        EvaluationKeysPrepared::alloc(&params);
    keys_prepared.prepare(params.module(), &keys, scratch.borrow());

    // Some deterministic randomness
    let mut source: Source = Source::new([5u8; 32]);

    // Word-size
    let ws: usize = params.word_size();

    // Allocates some dummy data
    let mut data: Vec<u8> = vec![0u8; params.max_addr() * ws];
    source.fill_bytes(data.as_mut_slice());

    // Instantiates the FHE-RAM
    let mut ram: Ram<BackendImpl> = Ram::new();

    // Populates the FHE-RAM
    ram.encrypt_sk(&data, &sk, &mut source_xa, &mut source_xe);

    // Allocates an encrypted address.
    let mut addr: Address<Vec<u8>> = Address::alloc_from_params(&params);

    // Random index
    let idx: u32 = source.next_u32() % params.max_addr() as u32;

    // Encrypts random index
    addr.encrypt_sk(
        &params,
        idx,
        &sk,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    // Reads from the FHE-RAM (with preparing for write)
    let start: Instant = Instant::now();
    let ct: Vec<GLWE<Vec<u8>>> = ram.read_prepare_write(&addr, &keys_prepared);
    let duration: std::time::Duration = start.elapsed();
    println!(
        "READ_PREPARE_WRITE Elapsed time: {} ms",
        duration.as_millis()
    );

    // Checks correctness
    (0..ws).for_each(|i| {
        let want = cast_u8_to_signed(data[i + ws * idx as usize], params.k_glwe_pt().as_usize()); // 8-bit signed integer
        let (decrypted_value, noise) = decrypt_glwe(&params, &ct[i], want, &sk_prep);
        assert_eq!(decrypted_value, want);
        println!("noise: {}", noise);
        assert!(
            noise < -(params.k_glwe_pt().as_usize() as f64 + 1.0),
            "{} >= {}",
            noise,
            (params.k_glwe_pt().as_usize() as f64 + 1.0)
        );
    });

    // Value to write on the FHE-RAM
    let mut value: Vec<u8> = vec![0u8; ws];
    source.fill_bytes(value.as_mut_slice());

    // Encryptes value to write on the FHE-RAM
    let ct_w = value
        .iter()
        .map(|wi| encrypt_glwe(&params, *wi, &sk_prep))
        .collect::<Vec<_>>();

    // Writes on the FHE-RAM
    let start: Instant = Instant::now();
    ram.write(&ct_w, &addr, &keys_prepared);
    let duration: std::time::Duration = start.elapsed();
    println!("WRITE Elapsed time: {} ms", duration.as_millis());

    // Updates plaintext ram
    (0..ws).for_each(|i| {
        data[i + ws * idx as usize] = value[i];
    });

    // Reads back at the written index
    let ct: Vec<GLWE<Vec<u8>>> = ram.read(&addr, &keys_prepared);

    // Checks correctness
    (0..ws).for_each(|i| {
        let want = cast_u8_to_signed(data[i + ws * idx as usize], params.k_glwe_pt().as_usize()); // 8-bit signed integer
        let (decrypted_value, noise) = decrypt_glwe(&params, &ct[i], want, &sk_prep);
        assert_eq!(decrypted_value, want);
        println!("noise: {}", noise);
        assert!(
            noise < -(params.k_glwe_pt().as_usize() as f64 + 1.0),
            "{} >= {}",
            noise,
            (params.k_glwe_pt().as_usize() as f64 + 1.0)
        );
    });
}
