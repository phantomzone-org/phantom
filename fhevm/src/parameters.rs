use poulpy_core::layouts::{
    Base2K, Degree, Dsize, GGLWELayout, GGLWEToGGSWKeyLayout, GGSWLayout,
    GLWEAutomorphismKeyLayout, GLWELayout, GLWEToLWEKeyLayout, Rank, TorusPrecision,
};
use poulpy_hal::{
    api::ModuleNew,
    layouts::{Backend, Module},
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::BDDKeyLayout, blind_rotation::BlindRotationKeyLayout,
    circuit_bootstrapping::CircuitBootstrappingKeyLayout,
};

const N_GLWE: u32 = 1024;
const N_LWE: u32 = 574;
const LWE_BLOCK_SIZE: u32 = 7;
const BASE2K: u32 = 15;
const RANK: u32 = 2;
const K_GLWE_PT: u32 = 2;
const K_GLWE_CT: u32 = BASE2K * 3;
const K_EVK_GLWE: u32 = BASE2K * 4;
const K_GGSW: u32 = BASE2K * 4;
const K_EVK_GGSW: u32 = BASE2K * 5;
pub const DECOMP_N: [u8; 2] = [5, 5];

pub struct CryptographicParameters<B: Backend> {
    module: Module<B>, // FFT/NTT tables.
    n_lwe: usize,
    lwe_block_size: usize,
    base2k: Base2K,
    rank: Rank,
    k_glwe_pt: TorusPrecision,
    k_glwe_ct: TorusPrecision,
    k_evk_glwe: TorusPrecision,
    k_ggsw: TorusPrecision,
    k_evk_ggsw: TorusPrecision,
}

impl<B: Backend> CryptographicParameters<B>
where
    Module<B>: ModuleNew<B>,
{
    pub fn new() -> Self {
        Self {
            module: Module::<B>::new(N_GLWE as u64),
            n_lwe: N_LWE as usize,
            lwe_block_size: LWE_BLOCK_SIZE as usize,
            base2k: BASE2K.into(),
            rank: RANK.into(),
            k_glwe_ct: K_GLWE_CT.into(),
            k_glwe_pt: K_GLWE_PT.into(),
            k_evk_glwe: K_EVK_GLWE.into(),
            k_ggsw: K_GGSW.into(),
            k_evk_ggsw: K_EVK_GGSW.into(),
        }
    }
}

impl<B: Backend> CryptographicParameters<B> {
    pub fn glwe_pt_infos(&self) -> GLWELayout {
        GLWELayout {
            n: self.module.n().into(),
            k: self.k_glwe_pt(),
            base2k: self.base2k(),
            rank: self.rank(),
        }
    }

    pub fn glwe_ct_infos(&self) -> GLWELayout {
        GLWELayout {
            n: self.module.n().into(),
            k: self.k_glwe_ct(),
            base2k: self.base2k(),
            rank: self.rank(),
        }
    }

    pub fn evk_glwe_infos(&self) -> GGLWELayout {
        GGLWELayout {
            n: self.module.n().into(),
            base2k: self.base2k(),
            k: self.k_evk_glwe(),
            rank_in: self.rank(),
            rank_out: self.rank(),
            dnum: self.k_glwe_ct().div_ceil(self.base2k).into(),
            dsize: Dsize(1),
        }
    }

    pub fn ggsw_infos(&self) -> GGSWLayout {
        GGSWLayout {
            n: self.module.n().into(),
            base2k: self.base2k(),
            k: self.k_ggsw(),
            rank: self.rank(),
            dnum: self.k_glwe_ct().div_ceil(self.base2k).into(),
            dsize: Dsize(1),
        }
    }

    pub fn evk_ggsw_infos(&self) -> GGLWELayout {
        GGLWELayout {
            n: self.module.n().into(),
            base2k: self.base2k(),
            k: self.k_evk_ggsw(),
            rank_in: self.rank(),
            rank_out: self.rank(),
            dnum: self.k_ggsw().div_ceil(self.base2k).into(),
            dsize: Dsize(1),
        }
    }

    pub fn module(&self) -> &Module<B> {
        &self.module
    }

    pub fn n_lwe(&self) -> Degree {
        self.n_lwe.into()
    }

    pub fn n_glwe(&self) -> Degree {
        self.module().n().into()
    }

    pub fn lwe_block_size(&self) -> usize {
        self.lwe_block_size
    }

    pub fn base2k(&self) -> Base2K {
        self.base2k
    }

    pub fn k_glwe_pt(&self) -> TorusPrecision {
        self.k_glwe_pt
    }

    pub fn k_glwe_ct(&self) -> TorusPrecision {
        self.k_glwe_ct
    }

    pub fn k_evk_glwe(&self) -> TorusPrecision {
        self.k_evk_glwe
    }

    pub fn k_ggsw(&self) -> TorusPrecision {
        self.k_ggsw
    }

    pub fn k_evk_ggsw(&self) -> TorusPrecision {
        self.k_evk_ggsw
    }

    pub fn rank(&self) -> Rank {
        self.rank
    }
}

impl<B: Backend> CryptographicParameters<B> {
    pub fn cbt_key_layout(&self) -> CircuitBootstrappingKeyLayout {
        CircuitBootstrappingKeyLayout {
            layout_brk: BlindRotationKeyLayout {
                n_glwe: self.module.n().into(),
                n_lwe: self.n_lwe.into(),
                base2k: self.base2k,
                k: self.k_evk_ggsw(),
                dnum: self.k_ggsw().div_ceil(self.base2k()).into(),
                rank: self.rank(),
            },
            layout_atk: GLWEAutomorphismKeyLayout {
                n: self.module.n().into(),
                base2k: self.base2k,
                k: self.k_evk_ggsw(),
                rank: self.rank(),
                dnum: self.k_ggsw().div_ceil(self.base2k()).into(),
                dsize: Dsize(1),
            },
            layout_tsk: GGLWEToGGSWKeyLayout {
                n: self.module.n().into(),
                base2k: self.base2k,
                k: self.k_evk_ggsw(),
                rank: self.rank(),
                dnum: self.k_ggsw().div_ceil(self.base2k()).into(),
                dsize: Dsize(1),
            },
        }
    }

    pub fn glwe_to_lwe_key_layout(&self) -> GLWEToLWEKeyLayout {
        GLWEToLWEKeyLayout {
            n: self.module.n().into(),
            base2k: self.base2k,
            k: self.k_evk_glwe(),
            rank_in: self.rank(),
            dnum: self.k_glwe_ct().div_ceil(self.base2k()).into(),
        }
    }

    pub fn bdd_key_layout(&self) -> BDDKeyLayout {
        BDDKeyLayout {
            cbt: self.cbt_key_layout(),
            ks: self.glwe_to_lwe_key_layout(),
        }
    }
}
