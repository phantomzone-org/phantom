use base2k::Module;

use crate::decompose::{Base1D, Base2D};

pub const LOGN_PBS: usize = 12;
pub const LOGN_LWE: usize = 11;
pub const LOGBASE2K: usize = 9;
pub const RLWE_COLS: usize = 4;
pub const VMPPMAT_ROWS: usize = RLWE_COLS;
pub const VMPPMAT_COLS: usize = RLWE_COLS + 1;
pub const WORD_BITS: usize = u32::BITS as _;
pub const WORD_SPLIT: usize = 1;
pub const LOGK: usize = WORD_BITS / WORD_SPLIT;

pub const DECOMP_ROM: [u8; 6] = [5, 5, 6, 5, 5, 5];
pub const DECOMP_U2: [u8; 1] = [2];
pub const DECOMP_U4: [u8; 1] = [4];
pub const DECOMP_U5: [u8; 1] = [5];
pub const DECOMP_U6: [u8; 1] = [6];
pub const DECOMP_N: [u8; 3] = [4, 4, 3];
pub const DECOMP_U32: [u8; 8] = [4, 4, 4, 4, 4, 4, 4, 4];

pub struct Parameters {
    pub module_lwe: Module,
    pub module_pbs: Module,
    pub rom_size: usize,
    pub ram_size: usize,
    pub addr_rom_decomp: Base2D,
    pub addr_ram_decomp: Base2D,
    pub instr_decomp: Base1D,
    pub u2_decomp: Base1D,
    pub u4_decomp: Base1D,
    pub u5_decomp: Base1D,
    pub u6_decomp: Base1D,
    pub u32_decomp: Base1D,
}

impl Parameters {
    pub fn new(rom_size: u32, ram_size: u32) -> Self {
        assert_eq!(
            DECOMP_N.iter().map(|&x| x as usize).sum::<usize>(),
            LOGN_LWE
        );

        println!("rom_size: {}", rom_size);
        println!("ram_size: {}", ram_size);

        Parameters {
            module_lwe: Module::new(1 << LOGN_LWE, base2k::MODULETYPE::FFT64),
            module_pbs: Module::new(1 << LOGN_PBS, base2k::MODULETYPE::FFT64),
            rom_size: rom_size as usize,
            ram_size: ram_size as usize,
            addr_rom_decomp: get_base_2d(rom_size, DECOMP_N.to_vec()),
            addr_ram_decomp: get_base_2d(ram_size, DECOMP_N.to_vec()),
            instr_decomp: Base1D(DECOMP_ROM.to_vec()),
            u2_decomp: Base1D(DECOMP_U2.to_vec()),
            u4_decomp: Base1D(DECOMP_U4.to_vec()),
            u5_decomp: Base1D(DECOMP_U5.to_vec()),
            u6_decomp: Base1D(DECOMP_U6.to_vec()),
            u32_decomp: Base1D(DECOMP_U32.to_vec()),
        }
    }

    pub fn module_lwe(&self) -> &Module {
        &self.module_lwe
    }

    pub fn module_pbs(&self) -> &Module {
        &self.module_pbs
    }

    pub fn addr_rom_max(&self) -> usize {
        self.addr_rom_decomp.max()
    }

    pub fn addr_ram_max(&self) -> usize {
        self.addr_ram_decomp.max()
    }

    pub fn u2_max(&self) -> usize {
        self.u2_decomp.max()
    }

    pub fn u4_max(&self) -> usize {
        self.u4_decomp.max()
    }

    pub fn u5_max(&self) -> usize {
        self.u5_decomp.max()
    }

    pub fn u6_max(&self) -> usize {
        self.u6_decomp.max()
    }
}

fn get_base_2d(value: u32, base: Vec<u8>) -> Base2D {
    let mut out: Vec<Base1D> = Vec::new();
    let mut value_bit_size: u32 = 32 - (value - 1).leading_zeros();

    'outer: while value_bit_size != 0 {
        let mut v = Vec::new();
        for i in 0..base.len() {
            if base[i] as u32 <= value_bit_size {
                v.push(base[i]);
                value_bit_size -= base[i] as u32;
            } else {
                v.push(value_bit_size as u8);
                out.push(Base1D(v));
                break 'outer;
            }
        }
        out.push(Base1D(v))
    }

    println!("value: {}", value);
    println!("out: {:?}", out);

    Base2D(out)
}
