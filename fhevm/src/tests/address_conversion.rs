use crate::{
    address_read::AddressRead, address_write::AddressWrite, parameters::CryptographicParameters,
};
use poulpy_backend::FFT64Ref;
use poulpy_core::{
    layouts::{
        GGSWLayout, GLWELayout, GLWEPlaintext, GLWESecret, GLWESecretPrepared, LWESecret,
        TorusPrecision, GLWE,
    },
    GLWERotate,
};
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{ScalarZnx, ScratchOwned, ZnxViewMut},
    source::Source,
};
use poulpy_schemes::tfhe::bdd_arithmetic::FheUintPrepared;

#[test]
fn test_fhe_uint_prepared_to_address_read() {
    use rand_core::RngCore;

    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();

    let mut source: Source = Source::new([6u8; 32]);
    let mut source_xs: Source = Source::new([4u8; 32]);
    let mut source_xa: Source = Source::new([5u8; 32]);
    let mut source_xe: Source = Source::new([6u8; 32]);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 22);

    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc(params.n_glwe(), params.rank());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    let mut sk_glwe_prep: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc(params.module(), params.rank());
    sk_glwe_prep.prepare(params.module(), &sk_glwe);
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

    let mut scalar: ScalarZnx<Vec<u8>> = ScalarZnx::alloc(params.n_glwe().into(), 1);
    scalar
        .raw_mut()
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = i as i64);

    let max_addr: u32 = params.n_glwe().as_u32() << 4;

    let k: u32 = source.next_u32() % max_addr;

    let address_infos: &GGSWLayout = &params.address_ram_infos();
    let fhe_uint_infos: &GLWELayout = &params.fhe_uint_infos();
    let fhe_uint_prepared_infos: &GGSWLayout = &params.fhe_uint_prepared_infos();

    let mut fhe_uint_prepared: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::<Vec<u8>, u32, FFT64Ref>::alloc_from_infos(
            params.module(),
            fhe_uint_prepared_infos,
        );
    fhe_uint_prepared.encrypt_sk(
        params.module(),
        k,
        &sk_glwe_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let mut address: AddressRead<Vec<u8>, FFT64Ref> =
        AddressRead::alloc_from_infos(params.module(), address_infos, max_addr - 1);

    address.set_from_fhe_uint_prepared(params.module(), &fhe_uint_prepared, 2, scratch.borrow());

    let mask: u32 = (1 << params.module().log_n()) - 1;

    let mut bit_rsh: usize = 2;

    for coordinate in address.coordinates.iter_mut() {
        let mut pt_want: GLWEPlaintext<Vec<u8>> = GLWEPlaintext::alloc_from_infos(fhe_uint_infos);
        pt_want.encode_coeff_i64(1, TorusPrecision(2), 0);

        let mut glwe: GLWE<Vec<u8>> = GLWE::alloc_from_infos(fhe_uint_infos);

        glwe.encrypt_sk(
            params.module(),
            &pt_want,
            &sk_glwe_prep,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );

        let rot: i64 = ((k >> bit_rsh) & mask) as i64;

        params
            .module()
            .glwe_rotate_inplace(-rot, &mut pt_want, scratch.borrow());

        coordinate.product_inplace(params.module(), &mut glwe, scratch.borrow());

        let mut pt_have: GLWEPlaintext<Vec<u8>> = GLWEPlaintext::alloc_from_infos(fhe_uint_infos);
        glwe.decrypt(
            params.module(),
            &mut pt_have,
            &sk_glwe_prep,
            scratch.borrow(),
        );

        println!(
            "noise: {}",
            glwe.noise(params.module(), &sk_glwe_prep, &pt_want, scratch.borrow())
                .std()
                .log2()
        );

        glwe.assert_noise(
            params.module(),
            &sk_glwe_prep,
            &pt_want,
            -(params.base2k().as_u32() as f64),
        );

        bit_rsh += params.module().log_n() as usize;
    }
}

#[test]
fn test_fhe_uint_prepared_to_address_write() {
    use rand_core::RngCore;

    let params: CryptographicParameters<FFT64Ref> = CryptographicParameters::<FFT64Ref>::new();

    let mut source: Source = Source::new([6u8; 32]);
    let mut source_xs: Source = Source::new([4u8; 32]);
    let mut source_xa: Source = Source::new([5u8; 32]);
    let mut source_xe: Source = Source::new([6u8; 32]);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 22);

    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc(params.n_glwe(), params.rank());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    let mut sk_glwe_prep: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc(params.module(), params.rank());
    sk_glwe_prep.prepare(params.module(), &sk_glwe);
    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.n_lwe());
    sk_lwe.fill_binary_block(params.lwe_block_size(), &mut source_xs);

    let mut scalar: ScalarZnx<Vec<u8>> = ScalarZnx::alloc(params.n_glwe().into(), 1);
    scalar
        .raw_mut()
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = i as i64);

    let max_addr: u32 = params.n_glwe().as_u32() << 4;

    let k: u32 = source.next_u32() % max_addr;

    let address_infos: &GGSWLayout = &params.address_ram_infos();
    let fhe_uint_infos: &GLWELayout = &params.fhe_uint_infos();
    let fhe_uint_prepared_infos: &GGSWLayout = &params.fhe_uint_prepared_infos();

    let mut fheuint: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::<Vec<u8>, u32, FFT64Ref>::alloc_from_infos(
            params.module(),
            fhe_uint_prepared_infos,
        );
    fheuint.encrypt_sk(
        params.module(),
        k,
        &sk_glwe_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let mut address: AddressWrite<Vec<u8>, FFT64Ref> =
        AddressWrite::alloc_from_infos(params.module(), address_infos, max_addr - 1);

    address.set_from_fhe_uint_prepared(params.module(), &fheuint, 0, scratch.borrow());

    let mask: u32 = (1 << params.module().log_n()) - 1;

    let mut bit_rsh: usize = 0;
    for coordinate in address.coordinates.iter_mut() {
        let mut pt_want: GLWEPlaintext<Vec<u8>> = GLWEPlaintext::alloc_from_infos(fhe_uint_infos);
        pt_want.encode_coeff_i64(1, TorusPrecision(2), 0);

        let mut glwe: GLWE<Vec<u8>> = GLWE::alloc_from_infos(fhe_uint_infos);

        glwe.encrypt_sk(
            params.module(),
            &pt_want,
            &sk_glwe_prep,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );

        let rot: i64 = ((k >> bit_rsh) & mask) as i64;

        params
            .module()
            .glwe_rotate_inplace(rot, &mut pt_want, scratch.borrow());

        coordinate.product_inplace(params.module(), &mut glwe, scratch.borrow());

        let mut pt_have: GLWEPlaintext<Vec<u8>> = GLWEPlaintext::alloc_from_infos(fhe_uint_infos);
        glwe.decrypt(
            params.module(),
            &mut pt_have,
            &sk_glwe_prep,
            scratch.borrow(),
        );

        println!(
            "noise: {}",
            glwe.noise(params.module(), &sk_glwe_prep, &pt_want, scratch.borrow())
                .std()
                .log2()
        );

        glwe.assert_noise(
            params.module(),
            &sk_glwe_prep,
            &pt_want,
            -(params.base2k().as_u32() as f64),
        );
        bit_rsh += params.module().log_n() as usize;
    }
}
