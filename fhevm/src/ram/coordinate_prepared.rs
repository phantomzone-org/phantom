use itertools::Itertools;
use poulpy_hal::layouts::{Backend, Data, DataMut, DataRef, Scratch};

use poulpy_core::{
    layouts::{
        GGSWInfos, GGSWPrepared, GGSWPreparedFactory, GLWEInfos, GLWEToMut, GLWEToRef, LWEInfos,
    },
    GLWEExternalProduct, ScratchTakeCore,
};

use crate::{ram::base::Base1D, ram::coordinate::Coordinate};

pub(crate) struct CoordinatePrepared<D: Data, B: Backend> {
    pub(crate) value: Vec<GGSWPrepared<D, B>>,
    pub(crate) base1d: Base1D,
}

impl<BE: Backend> CoordinatePrepared<Vec<u8>, BE> {
    #[allow(dead_code)]
    pub(crate) fn alloc_bytes<M, A>(module: &M, infos: &A, size: usize) -> usize
    where
        A: GGSWInfos,
        M: GGSWPreparedFactory<BE>,
    {
        size * GGSWPrepared::bytes_of_from_infos(module, infos)
    }

    pub(crate) fn alloc<M, A>(module: &M, infos: &A, base1d: &Base1D) -> Self
    where
        M: GGSWPreparedFactory<BE>,
        A: GGSWInfos,
    {
        Self {
            value: (0..base1d.0.len())
                .map(|_| GGSWPrepared::alloc_from_infos(module, infos))
                .collect_vec(),
            base1d: base1d.clone(),
        }
    }
}

impl<D: Data, B: Backend> LWEInfos for CoordinatePrepared<D, B> {
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

impl<D: Data, B: Backend> GLWEInfos for CoordinatePrepared<D, B> {
    fn rank(&self) -> poulpy_core::layouts::Rank {
        self.value[0].rank()
    }
}

impl<D: Data, B: Backend> GGSWInfos for CoordinatePrepared<D, B> {
    fn dnum(&self) -> poulpy_core::layouts::Dnum {
        self.value[0].dnum()
    }

    fn dsize(&self) -> poulpy_core::layouts::Dsize {
        self.value[0].dsize()
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
        assert_eq!(self.base1d, other.base1d);
        for (el_prep, el) in self.value.iter_mut().zip(other.value.iter()) {
            el_prep.prepare(module, el, scratch)
        }
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
        for (i, coordinate) in self.value.iter().enumerate() {
            if i == 0 {
                module.glwe_external_product(res, a, coordinate, scratch);
            } else {
                module.glwe_external_product_inplace(res, coordinate, scratch);
            }
        }
    }

    /// Evaluates GLWE(m) * GGSW(X^i).
    pub(crate) fn product_inplace<M, R>(&self, module: &M, res: &mut R, scratch: &mut Scratch<B>)
    where
        R: GLWEToMut,
        M: GLWEExternalProduct<B>,
        Scratch<B>: ScratchTakeCore<B>,
    {
        for coordinate in self.value.iter() {
            module.glwe_external_product_inplace(res, coordinate, scratch);
        }
    }
}
