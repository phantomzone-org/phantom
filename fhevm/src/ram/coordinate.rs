use itertools::izip;
use poulpy_hal::{
    api::{ModuleN, ScratchTakeBasic},
    layouts::{Backend, Data, DataMut, Module, Scratch, ZnxViewMut, ZnxZero},
    source::Source,
};

use poulpy_core::{
    layouts::{
        GGSWInfos, GGSWLayout, GLWEInfos, GLWELayout, GLWESecretPreparedFactory,
        GLWESecretPreparedToRef, LWEInfos, GGSW, GLWE,
    },
    GGSWEncryptSk, GLWEExternalProduct, GetDistribution, ScratchTakeCore,
};

use crate::{parameters::CryptographicParameters, ram::base::Base1D};

/// Coordinate stores Vec<GGSW(X^a_i)> such that prod X^{a_i} = X^a.
/// This provides a second decomposition over the one in base N to
/// to ensure that the digits are small enough to enable HE operation
/// over the digits (e.g. 2-4 bits digits instead of log(N)-bits digits).
pub struct Coordinate<D: Data> {
    pub value: Vec<GGSW<D>>,
    pub base1d: Base1D,
}

impl<D: Data> LWEInfos for Coordinate<D> {
    fn base2k(&self) -> poulpy_core::layouts::Base2K {
        self.value[0].base2k()
    }

    fn k(&self) -> poulpy_core::layouts::TorusPrecision {
        self.value[0].k()
    }

    fn n(&self) -> poulpy_core::layouts::Degree {
        self.value[0].n()
    }
}

impl<D: Data> GLWEInfos for Coordinate<D> {
    fn rank(&self) -> poulpy_core::layouts::Rank {
        self.value[0].rank()
    }
}

impl<D: Data> GGSWInfos for Coordinate<D> {
    fn dnum(&self) -> poulpy_core::layouts::Dnum {
        self.value[0].dnum()
    }

    fn dsize(&self) -> poulpy_core::layouts::Dsize {
        self.value[0].dsize()
    }
}

impl Coordinate<Vec<u8>> {
    #[allow(dead_code)]
    /// Allocates a new [Coordinate].
    /// * `base1d`: digit decomposition of the coordinate (e.g. [12], [6, 6], [4, 4, 4] or [3, 3, 3, 3] for LogN = 12).
    pub(crate) fn alloc<A>(infos: &A, base1d: &Base1D) -> Self
    where
        A: GGSWInfos,
    {
        Self {
            value: base1d
                .0
                .iter()
                .map(|_| GGSW::alloc_from_infos(infos))
                .collect(),
            base1d: base1d.clone(),
        }
    }
    #[allow(dead_code)]
    /// Scratch space required to evaluate GGSW(X^{i}) * GLWE(m).
    pub(crate) fn product_scratch_space<B: Backend>(params: &CryptographicParameters<B>) -> usize
    where
        Module<B>: GLWEExternalProduct<B>,
    {
        let module: &Module<B> = params.module();
        let glwe_infos: &GLWELayout = &params.glwe_ct_infos();
        let ggsw_infos: &GGSWLayout = &params.ggsw_infos();
        GLWE::external_product_tmp_bytes(module, glwe_infos, glwe_infos, ggsw_infos)
    }
}

pub(crate) trait TakeCoordinate {
    fn take_coordinate<A>(
        &mut self,
        infos: &A,
        base1d: &Base1D,
    ) -> (Coordinate<&mut [u8]>, &mut Self)
    where
        A: GGSWInfos;
}

impl<BE: Backend> TakeCoordinate for Scratch<BE>
where
    Self: ScratchTakeCore<BE>,
{
    fn take_coordinate<A>(
        &mut self,
        infos: &A,
        base1d: &Base1D,
    ) -> (Coordinate<&mut [u8]>, &mut Self)
    where
        A: GGSWInfos,
    {
        let (ggsws, scratch) = self.take_ggsw_slice(base1d.0.len(), infos);
        (
            Coordinate {
                value: ggsws,
                base1d: base1d.clone(),
            },
            scratch,
        )
    }
}

impl<D: DataMut> Coordinate<D> {
    /// Encrypts a value in [-N+1, N-1] as GGSW(X^{value}).
    ///
    /// # Arguments
    ///
    /// * `value`: value to encrypt.
    /// * `module`: FFT/NTT tables.
    /// * `sk_dft`: secret in Fourier domain.
    /// * `source_xa`: random coins generator for public polynomials.
    /// * `source_xe`: random coins generator for noise.
    /// * `sigma`: standard deviation of the noise.
    /// * `scratch`: scratch space provider.
    pub(crate) fn encrypt_sk<S, M, B: Backend>(
        &mut self,
        value: i64,
        module: &M,
        sk: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<B>,
    ) where
        M: GLWESecretPreparedFactory<B> + ModuleN + GGSWEncryptSk<B>,
        S: GLWESecretPreparedToRef<B> + GLWEInfos + GetDistribution,
        Scratch<B>: ScratchTakeCore<B>,
    {
        let n: usize = module.n();

        assert!(value.abs() < n as i64);

        let (mut scalar, scratch_1) = scratch.take_scalar_znx(module.n(), 1);
        scalar.zero();

        let sign: i64 = value.signum();
        let gap: usize = 1; // self.base1d.gap(module.log_n());

        let mut remain: usize = value.unsigned_abs() as usize;
        let mut tot_base: u8 = 0;

        for (coordinate, base) in izip!(self.value.iter_mut(), self.base1d.0.iter()) {
            let mask: usize = (1 << base) - 1;

            let chunk: usize = ((remain & mask) << tot_base) * gap;

            if sign < 0 && chunk != 0 {
                scalar.raw_mut()[n - chunk] = -1; // (X^i)^-1 = X^{2n-i} = -X^{n-i}
            } else {
                scalar.raw_mut()[chunk] = 1;
            }

            coordinate.encrypt_sk(module, &scalar, sk, source_xa, source_xe, scratch_1);

            if sign < 0 && chunk != 0 {
                scalar.raw_mut()[n - chunk] = 0;
            } else {
                scalar.raw_mut()[chunk] = 0;
            }

            remain >>= base;
            tot_base += base;
        }
    }
}
