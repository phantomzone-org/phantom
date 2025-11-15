use poulpy_hal::{
    api::{ModuleN, ScratchTakeBasic},
    layouts::{Backend, Data, DataMut, Scratch, ZnxViewMut, ZnxZero},
    source::Source,
};

use poulpy_core::{
    layouts::{
        GGSWInfos, GLWEInfos, GLWESecretPreparedToRef, LWEInfos, GGSW, GLWE,
    },
    GGSWEncryptSk, GLWEExternalProduct, GetDistribution, ScratchTakeCore,
};

/// Coordinate stores Vec<GGSW(X^a_i)> such that prod X^{a_i} = X^a.
/// This provides a second decomposition over the one in base N to
/// to ensure that the digits are small enough to enable HE operation
/// over the digits (e.g. 2-4 bits digits instead of log(N)-bits digits).
pub struct Coordinate<D: Data>(pub(crate) GGSW<D>);

impl<D: Data> LWEInfos for Coordinate<D> {
    fn base2k(&self) -> poulpy_core::layouts::Base2K {
        self.0.base2k()
    }

    fn k(&self) -> poulpy_core::layouts::TorusPrecision {
        self.0.k()
    }

    fn n(&self) -> poulpy_core::layouts::Degree {
        self.0.n()
    }
}

impl<D: Data> GLWEInfos for Coordinate<D> {
    fn rank(&self) -> poulpy_core::layouts::Rank {
        self.0.rank()
    }
}

impl<D: Data> GGSWInfos for Coordinate<D> {
    fn dnum(&self) -> poulpy_core::layouts::Dnum {
        self.0.dnum()
    }

    fn dsize(&self) -> poulpy_core::layouts::Dsize {
        self.0.dsize()
    }
}

impl Coordinate<Vec<u8>> {
    #[allow(dead_code)]
    /// Allocates a new [Coordinate].
    /// * `base1d`: digit decomposition of the coordinate (e.g. [12], [6, 6], [4, 4, 4] or [3, 3, 3, 3] for LogN = 12).
    pub(crate) fn alloc<A>(infos: &A) -> Self
    where
        A: GGSWInfos,
    {
        Self(GGSW::alloc_from_infos(infos))
    }
    #[allow(dead_code)]
    /// Scratch space required to evaluate GGSW(X^{i}) * GLWE(m).
    pub(crate) fn product_scratch_space<A, B, M, BE: Backend>(module: &M, ram_infos: &A, addr_infos: &B) -> usize
    where
        A: GLWEInfos,
        B: GGSWInfos,
        M: GLWEExternalProduct<BE>,
    {
        GLWE::external_product_tmp_bytes(module, ram_infos, ram_infos, addr_infos)
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
        M: ModuleN + GGSWEncryptSk<B>,
        S: GLWESecretPreparedToRef<B> + GLWEInfos + GetDistribution,
        Scratch<B>: ScratchTakeCore<B>,
    {
        let n: usize = module.n();
        let val_abs: usize = value.abs() as usize;

        assert!(value.abs() < n as i64);

        let (mut scalar, scratch_1) = scratch.take_scalar_znx(module.n(), 1);
        scalar.zero();

        if value.signum() < 0 {
            scalar.raw_mut()[n - val_abs] = -1; // (X^i)^-1 = X^{2n-i} = -X^{n-i}
        } else {
            scalar.raw_mut()[val_abs] = 1;
        }

        self.0
            .encrypt_sk(module, &scalar, sk, source_xa, source_xe, scratch_1);
    }
}
