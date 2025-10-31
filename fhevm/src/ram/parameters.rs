use poulpy_core::layouts::{
    Base2K, Dnum, Dsize, GGLWELayout, GGSWLayout, GLWEAutomorphismKeyLayout, GLWELayout,
    GLWETensorKeyLayout, GLWEToLWEKeyLayout, LWEInfos, LWELayout, LWEToGLWEKeyLayout, Rank,
    TorusPrecision,
};
use poulpy_hal::{
    api::ModuleNew,
    layouts::{Backend, Module},
};
use poulpy_schemes::tfhe::{
    blind_rotation::BlindRotationKeyLayout, circuit_bootstrapping::CircuitBootstrappingKeyLayout,
};

use crate::{Base2D, get_base_2d};

const LOG_N: u32 = 12;
const BASE2K: u32 = 17;
const RANK: u32 = 1;
const K_GLWE_PT: u32 = 3; //u8::BITS;
const K_GLWE_CT: u32 = BASE2K * 3;
const K_GGSW_ADDR: u32 = BASE2K * 4;
const K_EVK_TRACE: u32 = BASE2K * 4;
const K_EVK_GGSW_INV: u32 = BASE2K * 5;
pub const DECOMP_N: [u8; 4] = [3, 3, 3, 3];
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

    pub(crate) fn k_ggsw_addr(&self) -> TorusPrecision {
        self.k_ggsw_addr
    }

    pub fn rank(&self) -> Rank {
        self.rank
    }

    pub(crate) fn k_evk_trace(&self) -> TorusPrecision {
        self.k_evk_trace
    }

    pub(crate) fn k_evk_ggsw_inv(&self) -> TorusPrecision {
        self.k_evk_ggsw_inv
    }

    pub(crate) fn dnum_ct(&self) -> Dnum {
        self.k_glwe_ct().div_ceil(self.basek()).into()
    }

    pub(crate) fn dnum_ggsw(&self) -> Dnum {
        self.k_ggsw_addr().div_ceil(self.basek()).into()
    }
}

pub struct Parameters<B: Backend> {
    pub cryptographic_parameters: CryptographicParameters<B>, // Cryptographic parameters.
    pub max_addr: usize,                                      // Maximum supported address.
    pub decomp_n: Vec<u8>,                                    // Digit decomposition of N.
    pub word_size: usize,                                     // Digit decomposition of a Ram word.
}

impl<B: Backend> Default for Parameters<B>
where
    Module<B>: ModuleNew<B>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend> Parameters<B>
where
    Module<B>: ModuleNew<B>,
{
    pub fn new() -> Self {
        assert!(DECOMP_N.iter().sum::<u8>() == LOG_N as u8);

        Self {
            cryptographic_parameters: CryptographicParameters::new(),
            max_addr: MAX_ADDR,
            decomp_n: DECOMP_N.to_vec(),
            word_size: WORDSIZE,
        }
    }
}

impl<B: Backend> Parameters<B> {
    pub fn glwe_pt_infos(&self) -> GLWELayout {
        GLWELayout {
            n: self.cryptographic_parameters.module.n().into(),
            k: self.k_glwe_pt(),
            base2k: self.basek(),
            rank: self.rank(),
        }
    }

    pub fn glwe_ct_infos(&self) -> GLWELayout {
        GLWELayout {
            n: self.cryptographic_parameters.module.n().into(),
            k: self.k_glwe_ct(),
            base2k: self.basek(),
            rank: self.rank(),
        }
    }

    pub fn evk_glwe_infos(&self) -> GGLWELayout {
        GGLWELayout {
            n: self.cryptographic_parameters.module.n().into(),
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
            n: self.cryptographic_parameters.module.n().into(),
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
            n: self.cryptographic_parameters.module.n().into(),
            base2k: self.basek(),
            k: self.k_ggsw_addr(),
            rank: self.rank(),
            dnum: self.dnum_ct(),
            dsize: Dsize(1),
        }
    }

    pub fn lwe_ct_infos(&self) -> LWELayout {
        LWELayout {
            n: self.glwe_ct_infos().n(),
            k: self.glwe_ct_infos().k(),
            base2k: self.glwe_ct_infos().base2k(),
        }
    }

    pub fn lwe_pt_infos(&self) -> LWELayout {
        LWELayout {
            n: self.glwe_ct_infos().n(),
            k: self.glwe_pt_infos().k() + 1,
            base2k: self.glwe_ct_infos().base2k(),
        }
    }

    pub fn cbt_infos(&self) -> CircuitBootstrappingKeyLayout {
        // TODO: is K_EVK_TRACE correct?
        let cbt_infos: CircuitBootstrappingKeyLayout = CircuitBootstrappingKeyLayout {
            layout_brk: BlindRotationKeyLayout {
                n_glwe: self.module().n().into(),
                n_lwe: self.module().n().into(),
                base2k: self.basek().into(),
                k: K_EVK_TRACE.into(),
                dnum: K_EVK_TRACE.div_ceil(BASE2K).into(),
                rank: RANK.into(),
            },
            layout_atk: GLWEAutomorphismKeyLayout {
                n: self.module().n().into(),
                base2k: self.basek().into(),
                k: K_EVK_TRACE.into(),
                dnum: K_EVK_TRACE.div_ceil(BASE2K).into(),
                dsize: 1_u32.into(),
                rank: RANK.into(),
            },
            layout_tsk: GLWETensorKeyLayout {
                n: self.module().n().into(),
                base2k: self.basek().into(),
                k: K_EVK_TRACE.into(),
                dnum: K_EVK_TRACE.div_ceil(BASE2K).into(),
                dsize: 1_u32.into(),
                rank: RANK.into(),
            },
        };
        cbt_infos
    }

    pub fn lwe_to_glwe_switching_key_infos(&self) -> LWEToGLWEKeyLayout {
        LWEToGLWEKeyLayout {
            n: self.module().n().into(),
            base2k: BASE2K.into(),
            k: K_GLWE_CT.into(),
            dnum: K_GLWE_CT.div_ceil(BASE2K).into(),
            rank_out: RANK.into(),
        }
    }

    pub fn glwe_to_lwe_key_infos(&self) -> GLWEToLWEKeyLayout {
        GLWEToLWEKeyLayout {
            n: self.module().n().into(),
            base2k: BASE2K.into(),
            k: K_GLWE_CT.into(),
            dnum: K_GLWE_CT.div_ceil(BASE2K).into(),
            rank_in: RANK.into(),
        }
    }

    pub fn max_addr(&self) -> usize {
        self.max_addr
    }

    pub fn module(&self) -> &Module<B> {
        &self.cryptographic_parameters.module
    }

    pub fn basek(&self) -> Base2K {
        self.cryptographic_parameters.base2k
    }

    pub fn k_glwe_ct(&self) -> TorusPrecision {
        self.cryptographic_parameters.k_glwe_ct
    }

    pub fn k_glwe_pt(&self) -> TorusPrecision {
        self.cryptographic_parameters.k_glwe_pt
    }

    pub(crate) fn k_ggsw_addr(&self) -> TorusPrecision {
        self.cryptographic_parameters.k_ggsw_addr
    }

    pub fn rank(&self) -> Rank {
        self.cryptographic_parameters.rank
    }

    pub fn word_size(&self) -> usize {
        self.word_size
    }

    pub(crate) fn k_evk_trace(&self) -> TorusPrecision {
        self.cryptographic_parameters.k_evk_trace
    }

    pub(crate) fn k_evk_ggsw_inv(&self) -> TorusPrecision {
        self.cryptographic_parameters.k_evk_ggsw_inv
    }

    pub(crate) fn dnum_ct(&self) -> Dnum {
        self.k_glwe_ct().div_ceil(self.basek()).into()
    }

    pub(crate) fn dnum_ggsw(&self) -> Dnum {
        self.k_ggsw_addr().div_ceil(self.basek()).into()
    }

    pub(crate) fn decomp_n(&self) -> Vec<u8> {
        self.decomp_n.clone()
    }

    pub(crate) fn base2d(&self) -> Base2D {
        get_base_2d(self.max_addr() as u32, self.decomp_n())
    }
}

#[cfg(test)]
mod tests {
    use poulpy_backend::FFT64Ref;

    use super::*;

    #[test]
    fn new_sets_expected_defaults() {
        let p: Parameters<FFT64Ref> = Parameters::<FFT64Ref>::new();

        // Public accessors
        assert_eq!(p.basek(), BASE2K);
        assert_eq!(p.k_glwe_ct(), K_GLWE_CT);
        assert_eq!(p.k_glwe_pt(), K_GLWE_PT);
        assert_eq!(p.rank(), RANK);
        assert_eq!(p.word_size(), WORDSIZE);
        assert_eq!(p.max_addr(), MAX_ADDR);

        // Module size
        assert_eq!(p.module().n(), 1 << LOG_N);

        // Crate-visible accessors/fields
        assert_eq!(p.k_ggsw_addr(), K_GGSW_ADDR);
        assert_eq!(p.k_evk_ggsw_inv(), K_EVK_GGSW_INV);
        assert_eq!(p.k_evk_trace(), K_EVK_TRACE);

        // Decomposition of N
        assert_eq!(p.decomp_n(), DECOMP_N.to_vec());
        assert_eq!(p.decomp_n().iter().copied().sum::<u8>(), LOG_N as u8);

        let expected_rows_ct = K_GLWE_CT.div_ceil(BASE2K);

        assert_eq!(p.dnum_ct(), expected_rows_ct);
    }
}
