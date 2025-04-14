use base2k::Module;

use crate::decompose::Decomp;

pub const LOGN_PBS: usize = 12;
pub const LOGN_LWE: usize = 11;
pub const LOGBASE2K: usize = 9;
pub const RLWE_COLS: usize = 4;
pub const VMPPMAT_ROWS: usize = RLWE_COLS;
pub const VMPPMAT_COLS: usize = RLWE_COLS + 1;
pub const WORD_BITS: usize = u32::BITS as _;
pub const WORD_SPLIT: usize = 1;
pub const LOGK: usize = WORD_BITS / WORD_SPLIT;
pub const DECOMPOSE_INSTRUCTIONS: [u8; 6] = [5, 5, 6, 5, 5, 5];
pub const DECOMPOSE_ARITHMETIC: [u8; 8] = [4, 4, 4, 4, 4, 4, 4, 4];

// PROGRAM COUNTER
pub const ADDR_PC_N1: usize = 4;
pub const ADDR_PC_N2: usize = 1;
pub const ADDR_PC_N1_DECOMP: [u8; ADDR_PC_N1] = [3, 3, 3, 2];

// MEMORY
pub const ADDR_MEM_N1: usize = 4;
pub const ADDR_MEM_N2: usize = 1;
pub const ADDR_MEM_N1_DECOMP: [u8; ADDR_MEM_N1] = [3, 3, 3, 2];
pub const ADDR_MEM_SIZE_U8: u32 = 1024;
//pub const MEM_SIZE_u32: u32 = MEM_SIZE_U8 >>2;

// U2
pub const ADDR_U2_N1: usize = 1;
pub const ADDR_U2_N2: usize = 1;
pub const ADDR_U2_N1_DECOMP: [u8; ADDR_U2_N1] = [2];

// U4
pub const ADDR_U4_N1: usize = 1;
pub const ADDR_U4_N2: usize = 1;
pub const ADDR_U4_N1_DECOMP: [u8; ADDR_U4_N1] = [4];

// U5
pub const ADDR_U5_N1: usize = 1;
pub const ADDR_U5_N2: usize = 1;
pub const ADDR_U5_N1_DECOMP: [u8; ADDR_U5_N1] = [5];

// U6
pub const ADDR_U6_N1: usize = 1;
pub const ADDR_U6_N2: usize = 1;
pub const ADDR_U6_N1_DECOMP: [u8; ADDR_U6_N1] = [6];

// U32

pub struct Parameters {
    pub module_lwe: Module,
    pub module_pbs: Module,
    pub addr_pc_decomp: Decomp,
    pub addr_mem_decomp: Decomp,
    pub addr_u2_decomp: Decomp,
    pub addr_u4_decomp: Decomp,
    pub addr_u5_decomp: Decomp,
    pub addr_u6_decomp: Decomp,
}

impl Parameters {
    pub fn new() -> Self {
        Parameters {
            module_lwe: Module::new(1 << LOGN_LWE, base2k::MODULETYPE::FFT64),
            module_pbs: Module::new(1 << LOGN_PBS, base2k::MODULETYPE::FFT64),
            addr_pc_decomp: Decomp {
                n1: ADDR_PC_N1,
                n2: ADDR_PC_N2,
                base: ADDR_PC_N1_DECOMP.to_vec(),
            },
            addr_mem_decomp: Decomp {
                n1: ADDR_MEM_N1,
                n2: ADDR_MEM_N2,
                base: ADDR_MEM_N1_DECOMP.to_vec(),
            },
            addr_u2_decomp: Decomp {
                n1: ADDR_U2_N1,
                n2: ADDR_U2_N2,
                base: ADDR_U2_N1_DECOMP.to_vec(),
            },
            addr_u4_decomp: Decomp {
                n1: ADDR_U4_N1,
                n2: ADDR_U4_N2,
                base: ADDR_U4_N1_DECOMP.to_vec(),
            },
            addr_u5_decomp: Decomp {
                n1: ADDR_U5_N1,
                n2: ADDR_U5_N2,
                base: ADDR_U5_N1_DECOMP.to_vec(),
            },
            addr_u6_decomp: Decomp {
                n1: ADDR_U6_N1,
                n2: ADDR_U6_N2,
                base: ADDR_U6_N1_DECOMP.to_vec(),
            },
        }
    }

    pub fn module_lwe(&self) -> &Module {
        &self.module_lwe
    }

    pub fn module_pbs(&self) -> &Module {
        &self.module_pbs
    }

    pub fn addr_pc_max(&self) -> usize {
        self.addr_pc_decomp.max()
    }

    pub fn addr_mem_max(&self) -> usize {
        self.addr_mem_decomp.max()
    }

    pub fn addr_u2_max(&self) -> usize {
        self.addr_u5_decomp.max()
    }

    pub fn addr_u4_max(&self) -> usize {
        self.addr_u4_decomp.max()
    }

    pub fn addr_u5_max(&self) -> usize {
        self.addr_u5_decomp.max()
    }

    pub fn addr_u6_max(&self) -> usize {
        self.addr_u6_decomp.max()
    }
}
