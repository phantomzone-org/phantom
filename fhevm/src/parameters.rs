use poulpy_bin_fhe::{
    bdd_arithmetic::BDDKeyLayout, blind_rotation::BlindRotationKeyLayout,
    circuit_bootstrapping::CircuitBootstrappingKeyLayout,
};
use poulpy_core::layouts::{
    Base2K, Degree, Dnum, Dsize, GGLWELayout, GGLWEToGGSWKeyLayout, GGSWLayout,
    GLWEAutomorphismKeyLayout, GLWELayout, GLWESwitchingKeyLayout, GLWEToLWEKeyLayout, Rank,
    TorusPrecision,
};
use poulpy_hal::{
    api::ModuleNew,
    layouts::{Backend, Module},
};

/*
654 -> 3 -> 218
Average Cycle Time: 4.632201675s
- Prepare instruction components: 1.041632572s
- Prepare registers: 1.147898431s
- PC prepare: 224.262389ms
664 - 4 -> 166
- Prepare instruction components: 960.442533ms
- Prepare registers: 1.074004374s
- PC prepare: 207.734246ms
670 - 5 -> 135
- Prepare instruction components: 933.208184ms
- Prepare registers: 1.01560135s
- PC prepare: 203.287622ms
672 - 6 -> 112
- Prepare instruction components: 911.384349ms
- Prepare registers: 1.018545983s
- PC prepare: 188.22003ms
679 - 7 -> 97
- Prepare instruction components: 901.502526ms
- Read registers: 11.888516ms
- Prepare registers: 987.220313ms
- PC prepare: 186.559494ms
*/

const LOGN_GLWE: u32 = 10;
const N_GLWE: u32 = 1 << LOGN_GLWE;
const N_LWE: u32 = 679;
const K_LWE: u32 = 16;
const LWE_BLOCK_SIZE: u32 = 7;
const BASE2K_FHE_UINT: u32 = 15;
const BASE2K_CBT_BRK: u32 = 15;
const BASE2K_CBT_ATK: u32 = 15;
const BASE2K_CBT_TSK: u32 = 15;
const BASE2K_GLWE_TO_GLWE_KSK: u32 = 13;
const BASE2K_GLWE_TO_LWE_KSK: u32 = 4;
const RANK: u32 = 2;
const K_GLWE_PT: u32 = 2;

const K_ROM: u32 = BASE2K_FHE_UINT * 2;
const K_RAM: u32 = BASE2K_FHE_UINT * 2;
const K_FHE_UINT: u32 = BASE2K_FHE_UINT * 2;

const K_EVK_RAM_READ: u32 = BASE2K_FHE_UINT * 3;
const K_FHE_UINT_PREPARED: u32 = BASE2K_FHE_UINT * 3;
const K_PBS: u32 = BASE2K_FHE_UINT * 4;

const K_GLWE_TO_GLWE_KSK: u32 = 27;
const K_GLWE_TO_LWE_KSK: u32 = 16;

/// Auxiliary guard precision of a gadget key (GGSW / GGLWE) of total precision
/// `k` whose `dnum` digits of `dsize` limbs each cover `dnum * dsize * base2k`
/// bits.
///
/// Since `poulpy` 0.8 a gadget key is parameterized by `(dnum, dsize, k_aux)`
/// with `k = dnum * dsize * base2k + k_aux` and the invariant
/// `k_aux >= dsize * base2k`: the guard must hold at least one full digit.
/// Keys whose former `k` left less than that below the gadget region are
/// widened to the minimum guard (which does not change the number of digits
/// used by the decomposition, only the precision of the key material).
fn key_aux(k: TorusPrecision, base2k: Base2K, dnum: Dnum, dsize: Dsize) -> TorusPrecision {
    let digits: u32 = dnum.0 * dsize.0 * base2k.0;
    let min_aux: u32 = dsize.0 * base2k.0;
    TorusPrecision(k.0.saturating_sub(digits).max(min_aux))
}

pub struct CryptographicParameters<B: Backend> {
    module: Module<B>,
    n_lwe: usize,
    lwe_block_size: usize,
    base2k_fhe_uint: Base2K,
    base2k_cbt_brk: Base2K,
    base2k_cbt_atk: Base2K,
    base2k_cbt_tsk: Base2K,
    base2k_glwe_to_glwe_ksk: Base2K,
    base2k_glwe_to_lwe_ksk: Base2K,
    rank: Rank,
    k_pt: TorusPrecision,
    k_lwe: TorusPrecision,

    k_fhe_uint_prepared: TorusPrecision,
    k_fhe_uint: TorusPrecision,

    k_rom: TorusPrecision,
    k_ram: TorusPrecision,
    k_evk_ram: TorusPrecision,
    k_glwe_to_glwe_ksk: TorusPrecision,
    k_glwe_to_lwe_ksk: TorusPrecision,

    k_pbs: TorusPrecision,
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
            base2k_cbt_atk: BASE2K_CBT_ATK.into(),
            base2k_cbt_brk: BASE2K_CBT_BRK.into(),
            base2k_cbt_tsk: BASE2K_CBT_TSK.into(),
            base2k_fhe_uint: BASE2K_FHE_UINT.into(),
            base2k_glwe_to_glwe_ksk: BASE2K_GLWE_TO_GLWE_KSK.into(),
            base2k_glwe_to_lwe_ksk: BASE2K_GLWE_TO_LWE_KSK.into(),
            rank: RANK.into(),
            k_pt: K_GLWE_PT.into(),
            k_fhe_uint_prepared: K_FHE_UINT_PREPARED.into(),
            k_fhe_uint: K_FHE_UINT.into(),
            k_rom: K_ROM.into(),
            k_ram: K_RAM.into(),
            k_evk_ram: K_EVK_RAM_READ.into(),
            k_pbs: K_PBS.into(),
            k_lwe: K_LWE.into(),
            k_glwe_to_glwe_ksk: K_GLWE_TO_GLWE_KSK.into(),
            k_glwe_to_lwe_ksk: K_GLWE_TO_LWE_KSK.into(),
        }
    }
}

impl<B: Backend> Default for CryptographicParameters<B>
where
    Module<B>: ModuleNew<B>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<B: Backend> CryptographicParameters<B> {
    pub fn glwe_pt_infos(&self) -> GLWELayout {
        GLWELayout {
            n: self.n_glwe(),
            k: self.k_pt(),
            base2k: self.base2k_fhe_uint(),
            rank: self.rank(),
        }
    }

    pub fn fhe_uint_infos(&self) -> GLWELayout {
        GLWELayout {
            n: self.n_glwe(),
            k: self.k_fhe_uint(),
            base2k: self.base2k_fhe_uint(),
            rank: self.rank(),
        }
    }

    pub fn fhe_uint_prepared_infos(&self) -> GGSWLayout {
        let base2k: Base2K = self.base2k_fhe_uint();
        let dnum: Dnum = self.k_fhe_uint().div_ceil(base2k).into();
        let dsize: Dsize = Dsize(1);
        GGSWLayout {
            n: self.n_glwe(),
            base2k,
            k_aux: key_aux(self.k_fhe_uint_prepared(), base2k, dnum, dsize),
            rank: self.rank(),
            dnum,
            dsize,
        }
    }

    pub fn rom_infos(&self) -> GLWELayout {
        GLWELayout {
            n: self.n_glwe(),
            k: self.k_rom(),
            base2k: self.base2k_fhe_uint(),
            rank: self.rank(),
        }
    }

    pub fn ram_infos(&self) -> GLWELayout {
        GLWELayout {
            n: self.n_glwe(),
            k: self.k_ram(),
            base2k: self.base2k_fhe_uint(),
            rank: self.rank(),
        }
    }

    pub fn evk_ram_infos(&self) -> GGLWELayout {
        let base2k: Base2K = self.base2k_fhe_uint();
        let dnum: Dnum = self.k_ram().div_ceil(base2k).into();
        let dsize: Dsize = Dsize(1);
        GGLWELayout {
            n: self.n_glwe(),
            base2k,
            k_aux: key_aux(self.k_evk_ram(), base2k, dnum, dsize),
            rank_in: self.rank(),
            rank_out: self.rank(),
            dnum,
            dsize,
            stride: 1,
        }
    }

    pub fn module(&self) -> &Module<B> {
        &self.module
    }

    pub fn n_lwe(&self) -> Degree {
        Degree(self.n_lwe as u32)
    }

    pub fn n_glwe(&self) -> Degree {
        Degree(self.module.n() as u32)
    }

    pub fn lwe_block_size(&self) -> usize {
        self.lwe_block_size
    }

    pub fn base2k_cbt_brk(&self) -> Base2K {
        self.base2k_cbt_brk
    }

    pub fn base2k_cbt_atk(&self) -> Base2K {
        self.base2k_cbt_atk
    }

    pub fn base2k_cbt_tsk(&self) -> Base2K {
        self.base2k_cbt_tsk
    }

    pub fn base2k_fhe_uint(&self) -> Base2K {
        self.base2k_fhe_uint
    }

    pub fn base2k_glwe_to_glwe_ksk(&self) -> Base2K {
        self.base2k_glwe_to_glwe_ksk
    }

    pub fn base2k_glwe_to_lwe_ksk(&self) -> Base2K {
        self.base2k_glwe_to_lwe_ksk
    }

    pub fn k_pt(&self) -> TorusPrecision {
        self.k_pt
    }

    pub fn k_rom(&self) -> TorusPrecision {
        self.k_rom
    }

    pub fn k_fhe_uint(&self) -> TorusPrecision {
        self.k_fhe_uint
    }

    pub fn k_fhe_uint_prepared(&self) -> TorusPrecision {
        self.k_fhe_uint_prepared
    }

    pub fn k_ram(&self) -> TorusPrecision {
        self.k_ram
    }

    pub fn k_evk_ram(&self) -> TorusPrecision {
        self.k_evk_ram
    }

    pub fn k_pbs(&self) -> TorusPrecision {
        self.k_pbs
    }

    pub fn k_lwe(&self) -> TorusPrecision {
        self.k_lwe
    }

    pub fn k_glwe_to_glwe_ksk(&self) -> TorusPrecision {
        self.k_glwe_to_glwe_ksk
    }

    pub fn k_glwe_to_lwe_ksk(&self) -> TorusPrecision {
        self.k_glwe_to_lwe_ksk
    }

    pub fn rank(&self) -> Rank {
        self.rank
    }
}

impl<B: Backend> CryptographicParameters<B> {
    pub fn cbt_key_layout(&self) -> CircuitBootstrappingKeyLayout {
        let dsize: Dsize = Dsize(1);
        let dnum_brk: Dnum = self
            .k_fhe_uint_prepared()
            .div_ceil(self.base2k_cbt_brk())
            .into();
        let dnum_atk: Dnum = self
            .k_fhe_uint_prepared()
            .div_ceil(self.base2k_cbt_atk())
            .into();
        let dnum_tsk: Dnum = self
            .k_fhe_uint_prepared()
            .div_ceil(self.base2k_cbt_tsk())
            .into();
        CircuitBootstrappingKeyLayout {
            brk_layout: BlindRotationKeyLayout {
                n_glwe: self.n_glwe(),
                n_lwe: self.n_lwe(),
                base2k: self.base2k_cbt_brk(),
                dnum: dnum_brk,
                k_aux: key_aux(self.k_pbs(), self.base2k_cbt_brk(), dnum_brk, dsize),
                rank: self.rank(),
            },
            atk_layout: GLWEAutomorphismKeyLayout {
                n: self.n_glwe(),
                base2k: self.base2k_cbt_atk(),
                dnum: dnum_atk,
                k_aux: key_aux(self.k_pbs(), self.base2k_cbt_atk(), dnum_atk, dsize),
                rank: self.rank(),
                dsize,
            },
            tsk_layout: GGLWEToGGSWKeyLayout {
                n: self.n_glwe(),
                base2k: self.base2k_cbt_tsk(),
                dnum: dnum_tsk,
                k_aux: key_aux(self.k_pbs(), self.base2k_cbt_tsk(), dnum_tsk, dsize),
                rank: self.rank(),
                dsize,
            },
        }
    }

    pub fn glwe_to_glwe_key_layout(&self) -> GLWESwitchingKeyLayout {
        let dnum: Dnum = Dnum(2);
        let dsize: Dsize = Dsize(1);
        GLWESwitchingKeyLayout {
            n: self.n_glwe(),
            base2k: self.base2k_glwe_to_glwe_ksk(),
            dnum,
            k_aux: key_aux(
                self.k_glwe_to_glwe_ksk(),
                self.base2k_glwe_to_glwe_ksk(),
                dnum,
                dsize,
            ),
            rank_in: self.rank(),
            rank_out: Rank(1),
            dsize,
        }
    }

    pub fn glwe_to_lwe_key_layout(&self) -> GLWEToLWEKeyLayout {
        let dnum: Dnum = Dnum(4);
        GLWEToLWEKeyLayout {
            n: self.n_glwe(),
            base2k: self.base2k_glwe_to_lwe_ksk(),
            dnum,
            k_aux: key_aux(
                self.k_glwe_to_lwe_ksk(),
                self.base2k_glwe_to_lwe_ksk(),
                dnum,
                Dsize(1),
            ),
            rank_in: Rank(1),
        }
    }

    pub fn bdd_key_layout(&self) -> BDDKeyLayout {
        BDDKeyLayout {
            cbt_layout: self.cbt_key_layout(),
            ks_glwe_layout: Some(self.glwe_to_glwe_key_layout()),
            ks_lwe_layout: self.glwe_to_lwe_key_layout(),
        }
    }
}
