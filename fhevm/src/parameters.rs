use poulpy_core::layouts::{
    Base2K, Dnum, Dsize, GGLWELayout, GGSWLayout, GLWELayout, Rank, TorusPrecision,
};
use poulpy_hal::{
    api::ModuleNew,
    layouts::{Backend, Module},
};

const LOG_N: u32 = 11;
const BASE2K: u32 = 17;
const RANK: u32 = 1;
const K_GLWE_PT: u32 = 3; //u8::BITS;
const K_GLWE_CT: u32 = BASE2K * 3;
const K_GGSW_ADDR: u32 = BASE2K * 4;
const K_EVK_TRACE: u32 = BASE2K * 4;
const K_EVK_GGSW_INV: u32 = BASE2K * 5;
pub const DECOMP_N: [u8; 2] = [6, 5];
const WORDSIZE: usize = 4;
const MAX_ADDR: usize = 1 << 14;

pub struct CryptographicParameters<B: Backend> {
    module: Module<B>,              // FFT/NTT tables.
    base2k: Base2K,                 // Torus 2^{-k} decomposition.
    rank: Rank,                     // GLWE/GGLWE/GGSW rank.
    k_glwe_pt: TorusPrecision,      // Ram plaintext (GLWE) Torus precision.
    k_glwe_ct: TorusPrecision,      // Ram ciphertext (GLWE) Torus precision.
    k_ggsw_addr: TorusPrecision,    // Ram address (GGSW) Torus precision.
    k_evk_trace: TorusPrecision,    // Ram trace evaluation key Torus precision
    k_evk_ggsw_inv: TorusPrecision, // Ram GGSW(X^i) -> GGSW(X^-i) evaluation key Torus precision
}

impl<B: Backend> CryptographicParameters<B>
where
    Module<B>: ModuleNew<B>,
{
    pub fn new() -> Self {
        Self {
            module: Module::<B>::new(1 << LOG_N),
            base2k: BASE2K.into(),
            rank: RANK.into(),
            k_glwe_ct: K_GLWE_CT.into(),
            k_glwe_pt: K_GLWE_PT.into(),
            k_ggsw_addr: K_GGSW_ADDR.into(),
            k_evk_trace: K_EVK_TRACE.into(),
            k_evk_ggsw_inv: K_EVK_GGSW_INV.into(),
        }
    }
}

impl<B: Backend> CryptographicParameters<B> {
    pub fn glwe_pt_infos(&self) -> GLWELayout {
        GLWELayout {
            n: self.module.n().into(),
            k: self.k_glwe_pt(),
            base2k: self.basek(),
            rank: self.rank(),
        }
    }

    pub fn glwe_ct_infos(&self) -> GLWELayout {
        GLWELayout {
            n: self.module.n().into(),
            k: self.k_glwe_ct(),
            base2k: self.basek(),
            rank: self.rank(),
        }
    }

    pub fn evk_glwe_infos(&self) -> GGLWELayout {
        GGLWELayout {
            n: self.module.n().into(),
            base2k: self.basek(),
            k: self.k_evk_trace(),
            rank_in: self.rank(),
            rank_out: self.rank(),
            dnum: self.dnum_ct(),
            dsize: Dsize(1),
        }
    }

    pub fn evk_ggsw_infos(&self) -> GGLWELayout {
        GGLWELayout {
            n: self.module.n().into(),
            base2k: self.basek(),
            k: self.k_evk_ggsw_inv(),
            rank_in: self.rank(),
            rank_out: self.rank(),
            dnum: self.dnum_ggsw(),
            dsize: Dsize(1),
        }
    }

    pub fn ggsw_infos(&self) -> GGSWLayout {
        GGSWLayout {
            n: self.module.n().into(),
            base2k: self.basek(),
            k: self.k_ggsw_addr(),
            rank: self.rank(),
            dnum: self.dnum_ct(),
            dsize: Dsize(1),
        }
    }

    pub fn module(&self) -> &Module<B> {
        &self.module
    }

    pub fn basek(&self) -> Base2K {
        self.base2k
    }

    pub fn k_glwe_ct(&self) -> TorusPrecision {
        self.k_glwe_ct
    }

    pub fn k_glwe_pt(&self) -> TorusPrecision {
        self.k_glwe_pt
    }

    pub fn k_ggsw_addr(&self) -> TorusPrecision {
        self.k_ggsw_addr
    }

    pub fn rank(&self) -> Rank {
        self.rank
    }

    pub fn k_evk_trace(&self) -> TorusPrecision {
        self.k_evk_trace
    }

    pub fn k_evk_ggsw_inv(&self) -> TorusPrecision {
        self.k_evk_ggsw_inv
    }

    pub fn dnum_ct(&self) -> Dnum {
        self.k_glwe_ct().div_ceil(self.basek()).into()
    }

    pub fn dnum_ggsw(&self) -> Dnum {
        self.k_ggsw_addr().div_ceil(self.basek()).into()
    }
}
