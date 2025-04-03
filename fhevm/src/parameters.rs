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
pub const DECOMPOSE_INSTRUCTIONS: [u8; 6] = [5, 5, 5, 6, 5, 5];
pub const DECOMPOSE_ARITHMETIC: [u8; 8] = [4, 4, 4, 4, 4, 4, 4, 4];
pub const DECOMPOSE_BYTEOFFSET: [u8; 1] = [2];

// PROGRAM COUNTER
pub const PC_N1: usize = 2;
pub const PC_N2: usize = 2;
pub const PC_N1_DECOMP: [u8; PC_N1] = [3, 3];

// MEMORY
pub const MEM_N1: usize = 2;
pub const MEM_N2: usize = 2;
pub const MEM_N1_DECOMP: [u8; MEM_N1] = [3, 3];

// REGISTERS
pub const REGISTERS_N1: usize = 1;
pub const REGISTERS_N2: usize = 1;
pub const REGISTERS_N1_DECOMP: [u8; REGISTERS_N1] = [5];

pub struct Parameters {
    pub module_lwe: Module,
    pub module_pbs: Module,
    pub pc_decomp: Decomp,
    pub mem_decomp: Decomp,
    pub registers_decomp: Decomp,
}

impl Parameters {
    pub fn new() -> Self {
        Parameters {
            module_lwe: Module::new(1 << LOGN_LWE, base2k::MODULETYPE::FFT64),
            module_pbs: Module::new(1 << LOGN_PBS, base2k::MODULETYPE::FFT64),
            pc_decomp: Decomp {
                n1: PC_N1,
                n2: PC_N2,
                base: PC_N1_DECOMP.to_vec(),
            },
            mem_decomp: Decomp {
                n1: MEM_N1,
                n2: MEM_N2,
                base: MEM_N1_DECOMP.to_vec(),
            },
            registers_decomp: Decomp {
                n1: REGISTERS_N1,
                n2: REGISTERS_N2,
                base: REGISTERS_N1_DECOMP.to_vec(),
            },
        }
    }

    pub fn module_lwe(&self) -> &Module {
        &self.module_lwe
    }

    pub fn module_pbs(&self) -> &Module {
        &self.module_pbs
    }

    pub fn pc_max(&self) -> usize {
        self.pc_decomp.max()
    }

    pub fn mem_max(&self) -> usize {
        self.mem_decomp.max()
    }

    pub fn registers_max(&self) -> usize {
        self.registers_decomp.max()
    }
}
