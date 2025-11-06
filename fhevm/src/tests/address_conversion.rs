use crate::{
    get_base_2d,
    keys::{VMKeys, VMKeysPrepared},
    parameters::{CryptographicParameters, DECOMP_N},
    Address, Base2D, Ram,
};
use poulpy_backend::FFT64Ref;
use poulpy_core::{
    layouts::{GGSWLayout, GLWESecret, GLWESecretPrepared, LWEInfos, LWESecret},
    SIGMA,
};
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow, VecZnxRotateInplace},
    layouts::{ScalarZnx, ScratchOwned, ZnxViewMut},
    source::Source,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{FheUint, FheUintPrepared},
    blind_rotation::CGGI,
};

#[test]
fn test_fhe_uint_blocks_to_address() {
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

    let k: u32 = source.next_u32() % 1024;

    let ggsw_infos: &GGSWLayout = &params.ggsw_infos();

    let mut fheuint: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::<Vec<u8>, u32, FFT64Ref>::alloc_from_infos(params.module(), ggsw_infos);
    fheuint.encrypt_sk(
        params.module(),
        k,
        &sk_glwe_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let max_noise = |col_i: usize| {
        let mut noise: f64 = -((ggsw_infos.size() - 1) as f64
            * ggsw_infos.base2k().as_usize() as f64)
            + SIGMA.log2()
            + 3.0;
        noise += 0.5 * ggsw_infos.log_n() as f64;
        if col_i != 0 {
            noise += 0.5 * ggsw_infos.log_n() as f64
        }
        noise
    };

    let max_addr = 1024;

    let base_2d_ram: Base2D = get_base_2d(max_addr as u32, &DECOMP_N.to_vec());

    let mut address: Address<Vec<u8>> = Address::alloc_from_infos(ggsw_infos, &base_2d_ram);

    address.set_from_fheuint_prepared(params.module(), &fheuint, scratch.borrow());

    let mut bit_rsh: usize = 0;
    for coordinate in address.coordinates.iter_mut() {
        let mut bit_lsh: usize = 0;
        let base1d: crate::Base1D = coordinate.base1d.clone();

        for (ggsw, bit_mask) in coordinate.value.iter_mut().zip(base1d.0) {
            let mask: u32 = (1 << bit_mask) - 1;

            let mut pt: ScalarZnx<Vec<u8>> = ScalarZnx::alloc(params.n_glwe().into(), 1);
            pt.raw_mut()[0] = 1;

            let rot: i64 = (((k >> bit_rsh) & mask) << bit_lsh) as i64;

            params.module().vec_znx_rotate_inplace(
                -rot,
                &mut pt.as_vec_znx_mut(),
                0,
                scratch.borrow(),
            );

            ggsw.print_noise(params.module(), &sk_glwe_prep, &pt);

            ggsw.assert_noise(params.module(), &sk_glwe_prep, &pt, &max_noise);
            bit_lsh += bit_mask as usize;
            bit_rsh += bit_mask as usize;
        }
    }

    let mut dummy_ram = Ram::new(&params, 32, &DECOMP_N.to_vec(), max_addr);
    let mut dummy_data = vec![0u32; max_addr];
    dummy_data
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = i as u32);

    dummy_ram.encrypt_sk(
        params.module(),
        &dummy_data,
        &sk_glwe_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let mut res: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&params.glwe_ct_infos());

    let key: VMKeys<Vec<u8>, CGGI> =
        VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

    let mut key_prepared: VMKeysPrepared<Vec<u8>, CGGI, FFT64Ref> = VMKeysPrepared::alloc(&params);
    key_prepared.prepare(params.module(), &key, scratch.borrow());

    dummy_ram.read_to_fheuint(
        params.module(),
        &mut res,
        &address,
        &key_prepared,
        scratch.borrow(),
    );

    println!("noise: {}", res.noise(params.module(), k, &sk_glwe_prep, scratch.borrow()));

    assert_eq!(
        k,
        res.decrypt(params.module(), &sk_glwe_prep, scratch.borrow())
    )
}
