use poulpy_hal::layouts::{Backend, Data, DataMut, DataRef, Scratch};

use poulpy_core::{
    layouts::{
        GGSWInfos, GGSWPrepared, GGSWPreparedFactory, GLWEInfos, GLWEToMut, GLWEToRef, LWEInfos,
    },
    GLWEExternalProduct, ScratchTakeCore,
};

use crate::coordinate::Coordinate;

pub(crate) struct CoordinatePrepared<D: Data, B: Backend>(pub(crate) GGSWPrepared<D, B>);

impl<BE: Backend> CoordinatePrepared<Vec<u8>, BE> {
    #[allow(dead_code)]
    pub(crate) fn alloc_bytes<M, A>(module: &M, infos: &A) -> usize
    where
        A: GGSWInfos,
        M: GGSWPreparedFactory<BE>,
    {
        GGSWPrepared::bytes_of_from_infos(module, infos)
    }

    pub(crate) fn alloc_from_infos<M, A>(module: &M, infos: &A) -> Self
    where
        M: GGSWPreparedFactory<BE>,
        A: GGSWInfos,
    {
        Self(GGSWPrepared::alloc_from_infos(module, infos))
    }
}

impl<D: Data, B: Backend> LWEInfos for CoordinatePrepared<D, B> {
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

impl<D: Data, B: Backend> GLWEInfos for CoordinatePrepared<D, B> {
    fn rank(&self) -> poulpy_core::layouts::Rank {
        self.0.rank()
    }
}

impl<D: Data, B: Backend> GGSWInfos for CoordinatePrepared<D, B> {
    fn dnum(&self) -> poulpy_core::layouts::Dnum {
        self.0.dnum()
    }

    fn dsize(&self) -> poulpy_core::layouts::Dsize {
        self.0.dsize()
    }
}

impl<DM: DataMut, B: Backend> CoordinatePrepared<DM, B> {
    pub(crate) fn prepare<DR: DataRef, M>(
        &mut self,
        module: &M,
        other: &Coordinate<DR>,
        scratch: &mut Scratch<B>,
    ) where
        DR: DataRef,
        M: GGSWPreparedFactory<B>,
    {
        self.0.prepare(module, &other.0, scratch)
    }
}

impl<D: DataRef, B: Backend> CoordinatePrepared<D, B> {
    /// Evaluates GLWE(m) * GGSW(X^i).
    pub(crate) fn product<R, A, M>(&self, module: &M, res: &mut R, a: &A, scratch: &mut Scratch<B>)
    where
        R: GLWEToMut,
        A: GLWEToRef,
        M: GLWEExternalProduct<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        module.glwe_external_product(res, a, &self.0, scratch)
    }

    /// Evaluates GLWE(m) * GGSW(X^i).
    pub(crate) fn product_inplace<M, R>(&self, module: &M, res: &mut R, scratch: &mut Scratch<B>)
    where
        R: GLWEToMut,
        M: GLWEExternalProduct<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        module.glwe_external_product_inplace(res, &self.0, scratch);
    }
}
