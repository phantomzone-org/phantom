use fhevm::{Address, Base1D, Base2D};
use poulpy_backend::FFT64Ref;
use poulpy_core::{
    layouts::{
        Base2K, Degree, Dnum, Dsize, GGSWLayout, GLWESecret, GLWESecretPrepared, LWEInfos, Rank,
        TorusPrecision,
    },
    SIGMA,
};
use poulpy_hal::{
    api::{ModuleNew, ScratchOwnedAlloc, ScratchOwnedBorrow, VecZnxRotateInplace},
    layouts::{Module, ScalarZnx, ScratchOwned, ZnxViewMut},
    source::Source,
};
use poulpy_schemes::tfhe::bdd_arithmetic::FheUintPrepared;

#[test]
fn test_fhe_uint_blocks_to_address() {
    use rand_core::RngCore;

    let n: Degree = Degree(1 << 10);
    let base2k: Base2K = Base2K(13);
    let rank: Rank = Rank(2);
    let k_ggsw_res: TorusPrecision = TorusPrecision(39);
    let k_ggsw_apply: TorusPrecision = TorusPrecision(52);

    let ggsw_res_infos: GGSWLayout = GGSWLayout {
        n,
        base2k,
        k: k_ggsw_res,
        rank,
        dnum: Dnum(2),
        dsize: Dsize(1),
    };

    let ggsw_k_infos: GGSWLayout = GGSWLayout {
        n,
        base2k,
        k: k_ggsw_apply,
        rank,
        dnum: Dnum(3),
        dsize: Dsize(1),
    };

    let n_glwe: usize = n.into();

    let module: Module<FFT64Ref> = Module::<FFT64Ref>::new(n_glwe as u64);
    let mut source: Source = Source::new([6u8; 32]);
    let mut source_xs: Source = Source::new([1u8; 32]);
    let mut source_xa: Source = Source::new([2u8; 32]);
    let mut source_xe: Source = Source::new([3u8; 32]);

    let mut scratch: ScratchOwned<FFT64Ref> = ScratchOwned::alloc(1 << 22);

    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc(n, rank);
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    let mut sk_glwe_prep: GLWESecretPrepared<Vec<u8>, FFT64Ref> =
        GLWESecretPrepared::alloc(&module, rank);
    sk_glwe_prep.prepare(&module, &sk_glwe);

    let mut scalar: ScalarZnx<Vec<u8>> = ScalarZnx::alloc(n_glwe, 1);
    scalar
        .raw_mut()
        .iter_mut()
        .enumerate()
        .for_each(|(i, x)| *x = i as i64);

    let k: u32 = source.next_u32();

    let mut fheuint: FheUintPrepared<Vec<u8>, u32, FFT64Ref> =
        FheUintPrepared::<Vec<u8>, u32, FFT64Ref>::alloc_from_infos(&module, &ggsw_k_infos);
    fheuint.encrypt_sk(
        &module,
        k,
        &sk_glwe_prep,
        &mut source_xa,
        &mut source_xe,
        scratch.borrow(),
    );

    let base: [usize; 2] = [5, 5];

    assert_eq!(base.iter().sum::<usize>(), module.log_n());

    let max_noise = |col_i: usize| {
        let mut noise: f64 =
            -(ggsw_res_infos.size() as f64 * base2k.as_usize() as f64) + SIGMA.log2() + 3.0;
        noise += 0.5 * ggsw_res_infos.log_n() as f64;
        if col_i != 0 {
            noise += 0.5 * ggsw_res_infos.log_n() as f64
        }
        noise
    };

    let base_2d: Base2D = Base2D([Base1D([5, 5].into()), Base1D([5, 5].into())].into());

    let mut address: Address<Vec<u8>> = Address::alloc_from_infos(&ggsw_res_infos, &base_2d);

    address.set_from_fheuint_prepared(&module, &fheuint, scratch.borrow());

    let mut bit_rsh: usize = 0;
    for coordinate in address.coordinates.iter_mut() {
        let mut bit_lsh: usize = 0;
        let base1d: crate::Base1D = coordinate.base1d.clone();

        for (ggsw, bit_mask) in coordinate.value.iter_mut().zip(base1d.0) {
            let mask: u32 = (1 << bit_mask) - 1;

            let mut pt: ScalarZnx<Vec<u8>> = ScalarZnx::alloc(module.n(), 1);
            pt.raw_mut()[0] = 1;

            let rot: i64 = (((k >> bit_rsh) & mask) << bit_lsh) as i64;

            module.vec_znx_rotate_inplace(rot, &mut pt.as_vec_znx_mut(), 0, scratch.borrow());

            ggsw.assert_noise(&module, &sk_glwe_prep, &pt, &max_noise);
            bit_lsh += bit_mask as usize;
            bit_rsh += bit_mask as usize;
        }
    }
}
