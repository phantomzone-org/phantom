use poulpy_hal::{
    api::ModuleN,
    layouts::{Backend, Data, DataMut, DataRef, Module, ScalarZnx, Scratch, ZnxViewMut},
    source::Source,
};

use poulpy_core::{
    layouts::{
        GGSWInfos, GGSWPreparedFactory, GLWEInfos, GLWESecretPreparedFactory,
        GLWESecretPreparedToRef, LWEInfos,
    },
    GGSWEncryptSk, GetDistribution, ScratchTakeCore,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{
        BDDKeyHelper, FheUint, FheUintPrepare, FheUintPrepared, FheUintPreparedFactory,
        GGSWBlindRotation, UnsignedInteger,
    },
    blind_rotation::BlindRotationAlgo,
};

use crate::{
    parameters::CryptographicParameters, ram::base::Base2D, ram::coordinate::TakeCoordinate,
    ram::coordinate_prepared::CoordinatePrepared,
};

/// [Address] stores GGSW(X^{addr}) in d
/// form. That is, given addr = prod X^{a_i}, then
/// it stores Vec<[Coordinate]:(X^{a_0}), [Coordinate]:(X^{a_1}), ...>.
/// where [a_0, a_1, ...] is the representation in base N of a.
///
/// Such decomposition is necessary if the ring degree
/// N is smaller than the maximum supported address.
pub(crate) struct AddressRead<D: Data, BE: Backend> {
    pub coordinates: Vec<CoordinatePrepared<D, BE>>,
    pub base2d: Base2D,
}

impl<D: Data, BE: Backend> LWEInfos for AddressRead<D, BE> {
    fn base2k(&self) -> poulpy_core::layouts::Base2K {
        self.coordinates[0].base2k()
    }

    fn k(&self) -> poulpy_core::layouts::TorusPrecision {
        self.coordinates[0].k()
    }

    fn n(&self) -> poulpy_core::layouts::Degree {
        self.coordinates[0].n()
    }
}

impl<D: Data, BE: Backend> GLWEInfos for AddressRead<D, BE> {
    fn rank(&self) -> poulpy_core::layouts::Rank {
        self.coordinates[0].rank()
    }
}

impl<D: Data, BE: Backend> GGSWInfos for AddressRead<D, BE> {
    fn dnum(&self) -> poulpy_core::layouts::Dnum {
        self.coordinates[0].dnum()
    }

    fn dsize(&self) -> poulpy_core::layouts::Dsize {
        self.coordinates[0].dsize()
    }
}

impl<BE: Backend> AddressRead<Vec<u8>, BE> {
    #[allow(dead_code)]
    /// Allocates a new [Address].
    pub fn alloc_from_params(params: &CryptographicParameters<BE>, base_2d: &Base2D) -> Self
    where
        Module<BE>: GGSWPreparedFactory<BE>,
    {
        Self::alloc_from_infos(params.module(), &params.ggsw_infos(), base_2d)
    }

    pub fn alloc_from_infos<M, A>(module: &M, infos: &A, base_2d: &Base2D) -> Self
    where
        M: GGSWPreparedFactory<BE>,
        A: GGSWInfos,
    {
        Self {
            coordinates: base_2d
                .0
                .iter()
                .map(|base1d| CoordinatePrepared::alloc(module, infos, base1d))
                .collect(),
            base2d: base_2d.clone(),
        }
    }
}

impl<D: DataMut, BE: Backend> AddressRead<D, BE> {
    #[allow(dead_code)]
    /// Encrypts an u32 value into an [AddressRead] under the provided secret.
    pub fn encrypt_sk<M, S>(
        &mut self,
        module: &M,
        value: u32,
        sk: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleN + GGSWEncryptSk<BE> + GLWESecretPreparedFactory<BE> + GGSWPreparedFactory<BE>,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos + GetDistribution,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        debug_assert!(self.base2d.max() > value as usize);

        let mut remain: usize = value as _;

        for i in 0..self.base2d.0.len() {
            let base1d = &self.base2d.0[i];
            let max: usize = base1d.max();
            let k: usize = remain & (max - 1);
            let (mut tmp, scratch_1) = scratch.take_coordinate(self.at(i), base1d);
            tmp.encrypt_sk(-(k as i64), module, sk, source_xa, source_xe, scratch_1);
            self.at_mut(i).prepare(module, &tmp, scratch_1);
            remain /= max;
        }
    }

    pub fn set_from_fhe_uint<F, T, M, DK, BRA: BlindRotationAlgo, K>(
        &mut self,
        module: &M,
        fheuint: &FheUint<F, T>,
        bit_start: usize,
        bit_end: usize,
        keys: &K,
        scratch: &mut Scratch<BE>,
    ) where
        F: DataRef + DataMut,
        DK: DataRef,
        K: BDDKeyHelper<DK, BRA, BE>,
        T: UnsignedInteger,
        M: ModuleN
            + FheUintPreparedFactory<T, BE>
            + FheUintPrepare<BRA, T, BE>
            + GGSWBlindRotation<T, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let mut fheuint_prepared = FheUintPrepared::alloc_from_infos(module, self);
        fheuint_prepared.prepare_custom(module, &fheuint, bit_start, bit_end, keys, scratch);
        self.set_from_fhe_uint_prepared(module, &fheuint_prepared, 0, scratch);
    }

    pub fn set_from_fhe_uint_prepared<DR, M, T>(
        &mut self,
        module: &M,
        fheuint: &FheUintPrepared<DR, T, BE>,
        bit_start: usize,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleN + GGSWBlindRotation<T, BE> + GGSWPreparedFactory<BE>,
        DR: DataRef,
        T: UnsignedInteger,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        // X^0
        let mut test_vector: ScalarZnx<Vec<u8>> = ScalarZnx::alloc(module.n(), 1);
        test_vector.raw_mut()[0] = 1;

        let (mut ggsw, scratch_1) = scratch.take_ggsw(self);

        let mut bit_rsh: usize = bit_start;
        for coordinate in self.coordinates.iter_mut() {
            let mut bit_lsh: usize = 0;
            let base1d = coordinate.base1d.clone();

            for (ggsw_prepared, bit_mask) in coordinate.value.iter_mut().zip(base1d.0) {
                // X^{(fheuint>>bit_rsh) % 2^bit_mask)<<bit_lsh}
                module.scalar_to_ggsw_blind_rotation(
                    &mut ggsw,
                    &test_vector,
                    fheuint,
                    false,
                    bit_rsh,
                    bit_mask as usize,
                    bit_lsh,
                    scratch_1,
                );

                ggsw_prepared.prepare(module, &ggsw, scratch_1);

                bit_lsh += bit_mask as usize;
                bit_rsh += bit_mask as usize;
            }
        }
    }
}

impl<D: Data, BE: Backend> AddressRead<D, BE> {
    pub(crate) fn n2(&self) -> usize {
        self.coordinates.len()
    }

    pub(crate) fn at(&self, i: usize) -> &CoordinatePrepared<D, BE> {
        &self.coordinates[i]
    }

    pub(crate) fn at_mut(&mut self, i: usize) -> &mut CoordinatePrepared<D, BE> {
        &mut self.coordinates[i]
    }
}
