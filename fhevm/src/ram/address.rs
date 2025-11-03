use itertools::izip;
use poulpy_hal::{
    layouts::{Backend, Data, DataMut, Module, Scratch},
    source::Source,
};

use poulpy_core::{
    layouts::{GGSWInfos, GLWEInfos, GLWESecret, GLWESecretPreparedFactory, LWEInfos},
    GGSWEncryptSk, ScratchTakeCore,
};

use crate::{parameters::CryptographicParameters, Base2D, Coordinate};

/// [Address] stores GGSW(X^{addr}) in decomposed
/// form. That is, given addr = prod X^{a_i}, then
/// it stores Vec<[Coordinate]:(X^{a_0}), [Coordinate]:(X^{a_1}), ...>.
/// where [a_0, a_1, ...] is the representation in base N of a.
///
/// Such decomposition is necessary if the ring degree
/// N is smaller than the maximum supported address.
pub struct Address<D: Data> {
    pub coordinates: Vec<Coordinate<D>>,
    pub base2d: Base2D,
}

impl<D: Data> LWEInfos for Address<D> {
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

impl<D: Data> GLWEInfos for Address<D> {
    fn rank(&self) -> poulpy_core::layouts::Rank {
        self.coordinates[0].rank()
    }
}

impl<D: Data> GGSWInfos for Address<D> {
    fn dnum(&self) -> poulpy_core::layouts::Dnum {
        self.coordinates[0].dnum()
    }

    fn dsize(&self) -> poulpy_core::layouts::Dsize {
        self.coordinates[0].dsize()
    }
}

impl Address<Vec<u8>> {
    /// Allocates a new [Address].
    pub fn alloc_from_params<B: Backend>(
        params: &CryptographicParameters<B>,
        base_2d: &Base2D,
    ) -> Self {
        Self::alloc_from_infos(&params.ggsw_infos(), base_2d)
    }

    pub fn alloc_from_infos<A>(infos: &A, base_2d: &Base2D) -> Self
    where
        A: GGSWInfos,
    {
        Self {
            coordinates: base_2d
                .0
                .iter()
                .map(|base1d| Coordinate::alloc(infos, base1d))
                .collect(),
            base2d: base_2d.clone(),
        }
    }

    pub fn encrypt_sk_tmp_bytes<B: Backend>(params: &CryptographicParameters<B>) -> usize
    where
        Module<B>: GLWESecretPreparedFactory<B> + GGSWEncryptSk<B>,
    {
        Coordinate::encrypt_sk_tmp_bytes(params)
    }
}

impl<D: DataMut> Address<D> {
    /// Encrypts an u32 value into an [Address] under the provided secret.
    pub fn encrypt_sk<B: Backend>(
        &mut self,
        params: &CryptographicParameters<B>,
        value: u32,
        sk: &GLWESecret<Vec<u8>>,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<B>,
    ) where
        Module<B>: GGSWEncryptSk<B> + GLWESecretPreparedFactory<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        debug_assert!(self.base2d.max() > value as usize);

        let module: &Module<B> = params.module();

        let mut remain: usize = value as _;
        izip!(self.coordinates.iter_mut(), self.base2d.0.iter()).for_each(|(coordinate, base1d)| {
            let max: usize = base1d.max();
            let k: usize = remain & (max - 1);
            coordinate.encrypt_sk(-(k as i64), module, sk, source_xa, source_xe, scratch);
            remain /= max;
        })
    }
}

impl<D: Data> Address<D> {
    pub(crate) fn n2(&self) -> usize {
        self.coordinates.len()
    }

    pub(crate) fn at(&self, i: usize) -> &Coordinate<D> {
        &self.coordinates[i]
    }
}
