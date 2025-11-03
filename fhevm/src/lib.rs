pub mod address_conversion;
pub mod arithmetic;
pub(crate) mod codegen;
pub mod instructions;
pub mod interpreter;
pub mod keys;
pub mod parameters;
pub mod ram;
pub mod store;

// Re-export the main functionality
pub use instructions::*;
pub use interpreter::*;
use poulpy_core::layouts::{
    Base2K, Degree, Dnum, Dsize, GGSWLayout, GLWEAutomorphismKeyLayout, GLWELayout,
    GLWETensorKeyLayout, GLWEToLWEKeyLayout, Rank, TorusPrecision,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::BDDKeyLayout, blind_rotation::BlindRotationKeyLayout,
    circuit_bootstrapping::CircuitBootstrappingKeyLayout,
};
pub use ram::*;

const LOG_N: u32 = 12;
const N_GLWE: u32 = 1 << LOG_N;
const BASE2K: u32 = 17;
const RANK: u32 = 1;
const K_GLWE_CT: u32 = BASE2K * 3;
const K_GGSW_ADDR: u32 = BASE2K * 4;

pub static TEST_GLWE_INFOS: GLWELayout = GLWELayout {
    n: Degree(N_GLWE),
    base2k: Base2K(BASE2K),
    k: TorusPrecision(K_GLWE_CT),
    rank: Rank(RANK),
};

pub static TEST_GGSW_INFOS: GGSWLayout = GGSWLayout {
    n: Degree(N_GLWE),
    base2k: Base2K(BASE2K),
    k: TorusPrecision(K_GGSW_ADDR),
    rank: Rank(RANK),
    dnum: Dnum(2),
    dsize: Dsize(1),
};

pub static TEST_BDD_KEY_LAYOUT: BDDKeyLayout = BDDKeyLayout {
    cbt: CircuitBootstrappingKeyLayout {
        layout_brk: BlindRotationKeyLayout {
            n_glwe: Degree(N_GLWE),
            n_lwe: Degree(N_GLWE),
            base2k: Base2K(BASE2K),
            k: TorusPrecision(52),
            dnum: Dnum(3),
            rank: Rank(RANK),
        },
        layout_atk: GLWEAutomorphismKeyLayout {
            n: Degree(N_GLWE),
            base2k: Base2K(BASE2K),
            k: TorusPrecision(52),
            rank: Rank(RANK),
            dnum: Dnum(3),
            dsize: Dsize(1),
        },
        layout_tsk: GLWETensorKeyLayout {
            n: Degree(N_GLWE),
            base2k: Base2K(BASE2K),
            k: TorusPrecision(52),
            rank: Rank(RANK),
            dnum: Dnum(3),
            dsize: Dsize(1),
        },
    },
    ks: GLWEToLWEKeyLayout {
        n: Degree(N_GLWE),
        base2k: Base2K(BASE2K),
        k: TorusPrecision(39),
        rank_in: Rank(RANK),
        dnum: Dnum(2),
    },
};
