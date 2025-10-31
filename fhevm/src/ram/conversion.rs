use poulpy_core::{
    ScratchTakeCore,
    layouts::{GGSWInfos, GGSWLayout, GLWEInfos},
};
use poulpy_hal::{
    api::ModuleN,
    layouts::{Backend, DataMut, DataRef, Module, ScalarZnx, Scratch, ZnxViewMut},
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{
        BDDKeyPrepared, FheUint, FheUintBlocksPrepare, FheUintBlocksPreparedFactory,
        FheUintPrepared, GGSWBlindRotation, UnsignedInteger,
    },
    blind_rotation::BlindRotationAlgo,
};

use crate::Address;

impl<T: UnsignedInteger, BE: Backend> FHEUintBlocksToAddress<T, BE> for Module<BE> where
    Self: ModuleN + GGSWBlindRotation<T, BE>
{
}

pub trait FHEUintBlocksToAddress<T: UnsignedInteger, BE: Backend>
where
    Self: ModuleN + GGSWBlindRotation<T, BE>,
{
    fn fhe_uint_blocks_to_address_tmp_bytes<A, B>(&self, res_infos: &A, fheuint_infos: &B) -> usize
    where
        A: GLWEInfos,
        B: GGSWInfos,
    {
        self.scalar_to_ggsw_blind_rotation_tmp_bytes(res_infos, fheuint_infos)
    }

    fn fhe_uint_blocks_to_address<DM, DR>(
        &self,
        res: &mut Address<DM>,
        fheuint: &FheUintPrepared<DR, T, BE>,
        scratch: &mut Scratch<BE>,
    ) where
        DM: DataMut,
        DR: DataRef,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        // X^0
        let mut test_vector: ScalarZnx<Vec<u8>> = ScalarZnx::alloc(self.n(), 1);
        test_vector.raw_mut()[0] = 1;

        let mut bit_rsh: usize = 0;
        for coordinate in res.coordinates.iter_mut() {
            let mut bit_lsh: usize = 0;
            let base1d: crate::Base1D = coordinate.base1d.clone();

            for (ggsw, bit_mask) in coordinate.value.iter_mut().zip(base1d.0) {
                // X^{(fheuint>>bit_rsh) % 2^bit_mask)<<bit_lsh}
                self.scalar_to_ggsw_blind_rotation(
                    ggsw,
                    &test_vector,
                    fheuint,
                    true,
                    bit_rsh,
                    bit_mask as usize,
                    bit_lsh,
                    scratch,
                );
                bit_lsh += bit_mask as usize;
                bit_rsh += bit_mask as usize;
            }
        }
    }
}

impl<D: DataMut> Address<D> {
    pub fn set_from_fheuint_prepared<F, T, M, BE: Backend>(
        &mut self,
        module: &M,
        fheuint_prepared: &FheUintPrepared<F, T, BE>,
        scratch: &mut Scratch<BE>,
    ) where
        F: DataRef,
        T: UnsignedInteger,
        M: FHEUintBlocksToAddress<T, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        module.fhe_uint_blocks_to_address(self, fheuint_prepared, scratch);
    }

    pub fn set_from_fheuint<F, T, M, BRA: BlindRotationAlgo, BE: Backend>(
        &mut self,
        module: &M,
        fheuint: &FheUint<F, T>,
        bdd_key_prepared: &BDDKeyPrepared<F, BRA, BE>,
        ggsw_infos: &GGSWLayout,
        scratch: &mut Scratch<BE>,
    ) where
        F: DataRef + DataMut,
        T: UnsignedInteger,
        M: FheUintBlocksPreparedFactory<T, BE>
            + FheUintBlocksPrepare<BRA, T, BE>
            + FHEUintBlocksToAddress<T, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let mut fheuint_prepared = FheUintPrepared::alloc_from_infos(module, ggsw_infos);
        fheuint_prepared.prepare(module, &fheuint, &bdd_key_prepared, scratch);

        self.set_from_fheuint_prepared(module, &fheuint_prepared, scratch);
    }
}

impl Address<Vec<u8>> {
    pub fn set_from_fheuint_tmp_bytes<A, B, M, T, BE: Backend>(
        module: &M,
        res_infos: &A,
        fheuint_infos: &B,
    ) -> usize
    where
        A: GLWEInfos,
        B: GGSWInfos,
        T: UnsignedInteger,
        M: FHEUintBlocksToAddress<T, BE>,
    {
        module.fhe_uint_blocks_to_address_tmp_bytes(res_infos, fheuint_infos)
    }
}

#[test]
fn test_fhe_uint_blocks_to_address() {
    use poulpy_backend::FFT64Ref;
    use poulpy_core::{
        SIGMA,
        layouts::{
            Base2K, Degree, Dnum, Dsize, GGSWLayout, GLWESecret, GLWESecretPrepared, LWEInfos,
            Rank, TorusPrecision,
        },
    };
    use poulpy_hal::{
        api::{ModuleNew, ScratchOwnedAlloc, ScratchOwnedBorrow, VecZnxRotateInplace},
        layouts::{Module, ScalarZnx, ScratchOwned, ZnxViewMut},
        source::Source,
    };

    use rand_core::RngCore;

    use crate::{Base1D, Base2D};

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
