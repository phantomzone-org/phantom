use std::time::Instant;

use poulpy_backend::FFT64Ref;
use poulpy_core::layouts::{
    prepared::GLWESecretPrepared, GLWEInfos, GLWELayout, GLWESecret, LWESecret,
};
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::ScratchOwned,
    source::Source,
};

use crate::{
    address_read::AddressRead,
    address_write::AddressWrite,
    keys::{VMKeys, VMKeysPrepared},
    parameters::{CryptographicParameters},
    ram::ram::Ram,
};

use poulpy_schemes::tfhe::{bdd_arithmetic::FheUint, blind_rotation::CGGI};
use rand_core::RngCore;

#[test]
fn test_fhe_ram() {
    println!("Starting!");

    let threads = 8;

    let seed_xs: [u8; 32] = [0u8; 32];
    let seed_xa: [u8; 32] = [0u8; 32];
    let seed_xe: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);
    let mut source_xa: Source = Source::new(seed_xa);
    let mut source_xe: Source = Source::new(seed_xe);

    // See parameters.rs for configuration
    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();
    let glwe_infos: &GLWELayout = &params.glwe_ct_infos();

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

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
    let max_addr: usize = 251;

    let mask: u32 = ((1u64 << word_size) - 1) as u32;

    // Instantiates the FHE-RAM
    let mut ram: Ram = Ram::new(&params, word_size, max_addr);

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
    let mut addr: AddressRead<Vec<u8>, FFT64Ref> =
        AddressRead::alloc_from_params(&params, (max_addr-1) as u32);

    // Random index
    let idx: u32 = 158 % max_addr as u32;

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
    let mut res: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(glwe_infos);
    ram.read(
        threads,
        params.module(),
        &mut res,
        &addr,
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
    assert!(
        res.noise(
            params.module(),
            data[idx as usize],
            &sk_prep,
            scratch.borrow()
        )
        .std()
        .log2()
            < -32.0
    );

    let start: Instant = Instant::now();
    ram.read_prepare_write(
        threads,
        params.module(),
        &mut res,
        &addr,
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
    assert!(
        res.noise(
            params.module(),
            data[idx as usize],
            &sk_prep,
            scratch.borrow()
        )
        .std()
        .log2()
            < -32.0
    );

    // Value to write on the FHE-RAM
    let value: u32 = source.next_u32() & mask;

    // Encryptes value to write on the FHE-RAM
    let mut ct_write: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(glwe_infos);
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

    let mut address_write = AddressWrite::alloc_from_params(&params, (max_addr-1) as u32);
    address_write.encrypt_sk(
        params.module(),
        idx,
        &sk_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    // Writes on the FHE-RAM
    let start: Instant = Instant::now();
    ram.write(
        threads,
        params.module(),
        &ct_write,
        &address_write,
        &keys_prepared,
        scratch.borrow(),
    );
    let duration: std::time::Duration = start.elapsed();
    println!("WRITE Elapsed time: {} ms", duration.as_millis());

    // Reads back at the written index
    ram.read(
        threads,
        params.module(),
        &mut res,
        &addr,
        &keys_prepared,
        scratch.borrow(),
    );

    // Checks correctness & noise
    assert_eq!(
        data[idx as usize],
        res.decrypt(params.module(), &sk_prep, scratch.borrow())
    );
    assert!(
        res.noise(
            params.module(),
            data[idx as usize],
            &sk_prep,
            scratch.borrow()
        )
        .std()
        .log2()
            < -32.0
    );

    let mut ram_decrypted: Vec<u32> = vec![0u32; ram.max_addr()];
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
