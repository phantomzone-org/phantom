use poulpy_hal::{
    api::{ModuleLogN, ModuleN},
    layouts::{Backend, Data, DataMut, DataRef, Module, ScalarZnx, Scratch, ZnxViewMut},
    source::Source,
};

use poulpy_core::{
    layouts::{GGSWInfos, GGSWPreparedFactory, GLWEInfos, GLWESecretPreparedToRef, LWEInfos},
    GGSWEncryptSk, GetDistribution, ScratchTakeCore,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{
        BDDKeyHelper, BDDKeyInfos, FheUint, FheUintPrepare, FheUintPrepared,
        FheUintPreparedFactory, GGSWBlindRotation, UnsignedInteger,
    },
    blind_rotation::BlindRotationAlgo,
};

use crate::{
    coordinate::Coordinate, coordinate_prepared::CoordinatePrepared,
    parameters::CryptographicParameters,
};

/// [Address] stores GGSW(X^{addr}) in d
/// form. That is, given addr = prod X^{a_i}, then
/// it stores Vec<[Coordinate]:(X^{a_0}), [Coordinate]:(X^{a_1}), ...>.
/// where [a_0, a_1, ...] is the representation in base N of a.
///
/// Such decomposition is necessary if the ring degree
/// N is smaller than the maximum supported address.
pub(crate) struct AddressWrite<D: Data, BE: Backend> {
    pub coordinates: Vec<CoordinatePrepared<D, BE>>,
}

impl<D: Data, BE: Backend> LWEInfos for AddressWrite<D, BE> {
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

impl<D: Data, BE: Backend> GLWEInfos for AddressWrite<D, BE> {
    fn rank(&self) -> poulpy_core::layouts::Rank {
        self.coordinates[0].rank()
    }
}

impl<D: Data, BE: Backend> GGSWInfos for AddressWrite<D, BE> {
    fn dnum(&self) -> poulpy_core::layouts::Dnum {
        self.coordinates[0].dnum()
    }

    fn dsize(&self) -> poulpy_core::layouts::Dsize {
        self.coordinates[0].dsize()
    }
}

impl<BE: Backend> AddressWrite<Vec<u8>, BE> {
    #[allow(dead_code)]
    /// Allocates a new [Address].
    pub fn alloc_from_params(params: &CryptographicParameters<BE>, max_value: u32) -> Self
    where
        Module<BE>: GGSWPreparedFactory<BE>,
    {
        Self::alloc_from_infos(params.module(), &params.ggsw_infos(), max_value)
    }

    pub fn alloc_from_infos<M, A>(module: &M, infos: &A, max_value: u32) -> Self
    where
        M: ModuleLogN + GGSWPreparedFactory<BE>,
        A: GGSWInfos,
    {
        let max_value_bits =
            (u32::BITS - (max_value - 1).leading_zeros()).div_ceil(module.log_n() as u32);

        Self {
            coordinates: (0..max_value_bits)
                .map(|_| CoordinatePrepared::alloc_from_infos(module, infos))
                .collect(),
        }
    }
}

impl<D: Data, BE: Backend> AddressWrite<D, BE> {
    fn max_value(&self) -> u32 {
        (1 << (self.coordinates[0].log_n() * self.coordinates.len())) - 1
    }
}

impl<D: DataMut, BE: Backend> AddressWrite<D, BE> {
    #[allow(dead_code)]
    /// Encrypts an u32 value into an [AddressWrite] under the provided secret.
    pub fn encrypt_sk<M, S>(
        &mut self,
        module: &M,
        value: u32,
        sk: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleN + ModuleLogN + GGSWEncryptSk<BE> + GGSWPreparedFactory<BE>,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos + GetDistribution,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        debug_assert!(self.max_value() > value);

        let mut remain: usize = value as _;
        let log_n = module.log_n();
        let mask = (1 << log_n) - 1;

        let (tmp, scratch_1) = scratch.take_ggsw(self.at(0));
        let mut coordinate_tmp: Coordinate<&mut [u8]> = Coordinate(tmp);
        for coordinate in self.coordinates.iter_mut() {
            let k: usize = remain & mask;
            coordinate_tmp.encrypt_sk(k as i64, module, sk, source_xa, source_xe, scratch_1);
            coordinate.prepare(module, &coordinate_tmp, scratch_1);
            remain >>= log_n;
        }
    }

    pub fn set_from_fhe_uint<F, T, M, DK, BRA: BlindRotationAlgo, K>(
        &mut self,
        threads: usize,
        module: &M,
        fheuint: &FheUint<F, T>,
        bit_start: usize,
        bit_end: usize,
        keys: &K,
        scratch: &mut Scratch<BE>,
    ) where
        F: DataRef + DataMut,
        DK: DataRef,
        K: BDDKeyHelper<DK, BRA, BE> + BDDKeyInfos,
        T: UnsignedInteger,
        M: ModuleN
            + FheUintPreparedFactory<T, BE>
            + FheUintPrepare<BRA, BE>
            + GGSWBlindRotation<T, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let mut fheuint_prepared = FheUintPrepared::alloc_from_infos(module, self);
        fheuint_prepared.prepare_custom_multi_thread(
            threads, module, &fheuint, bit_start, bit_end, keys, scratch,
        );
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

        let log_n = self.log_n();

        let (mut ggsw, scratch_1) = scratch.take_ggsw(self);

        let mut bit_rsh: usize = bit_start;
        for coordinate in self.coordinates.iter_mut() {
            // X^{(fheuint>>bit_rsh) % 2^bit_mask)<<bit_lsh}
            module.scalar_to_ggsw_blind_rotation(
                &mut ggsw,
                &test_vector,
                fheuint,
                true,
                bit_rsh,
                log_n,
                0,
                scratch_1,
            );

            coordinate.0.prepare(module, &ggsw, scratch_1);
            bit_rsh += log_n as usize;
        }
    }
}

impl<D: Data, BE: Backend> AddressWrite<D, BE> {
    pub(crate) fn n2(&self) -> usize {
        self.coordinates.len()
    }

    pub(crate) fn at(&self, i: usize) -> &CoordinatePrepared<D, BE> {
        &self.coordinates[i]
    }

    #[allow(dead_code)]
    pub(crate) fn at_mut(&mut self, i: usize) -> &mut CoordinatePrepared<D, BE> {
        &mut self.coordinates[i]
    }
}
