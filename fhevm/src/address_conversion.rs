use poulpy_core::{
    layouts::{GGSWInfos, GLWEInfos},
    ScratchTakeCore,
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
        scratch: &mut Scratch<BE>,
    ) where
        F: DataRef + DataMut,
        T: UnsignedInteger,
        M: FheUintBlocksPreparedFactory<T, BE>
            + FheUintBlocksPrepare<BRA, T, BE>
            + FHEUintBlocksToAddress<T, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let mut fheuint_prepared = FheUintPrepared::alloc_from_infos(module, self);
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
