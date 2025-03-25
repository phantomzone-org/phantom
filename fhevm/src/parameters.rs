pub const LOGN_PBS: usize = 12;
pub const LOGN_LWE: usize = 11;
pub const RLWE_COLS: usize = 4;
pub const LOGBASE2K: usize = 9;
pub const VMPPMAT_ROWS: usize = 4;
pub const VMPPMAT_COLS: usize = 5;
pub const LOGK: usize = u32::BITS as _;
pub const DECOMPOSE_INSTRUCTIONS: [u8; 6] = [5, 5, 5, 6, 5, 5];
pub const DECOMPOSE_ARITHMETIC: [u8; 8] = [4, 4, 4, 4, 4, 4, 4, 4];
pub const DECOMPOSE_BYTEOFFSET: [u8; 1] = [2];

// PROGRAM COUNTER
pub const ADDRESS_PC_DECOMP: [u8; 6] = [4, 4, 4, 4, 4, 4];
pub const ADDRESS_PC_N1: usize = 2;
pub const ADDRESS_PC_N2: usize = 3;
pub const MAX_PC_ADDRESS: usize = 32;

// MEMORY
pub const ADDRESS_MEM_DECOMP: [u8; 6] = [4, 4, 4, 4, 4, 4];
pub const ADDRESS_MEM_N1: usize = 2;
pub const ADDRESS_MEM_N2: usize = 3;
pub const MAX_MEMORY_ADDRESS: usize = 32;

// REGISTERS
pub const REGISTERSCOUNT: usize = 32;
pub const ADDRESS_REGISTER_DECOMP: [u8; 1] = [5];
pub const ADDRESS_REGISTER_N1: usize = 1;
pub const ADDRESS_REGISTER_N2: usize = 1;

pub fn get_pc_address_decomp() -> Vec<Vec<u8>> {
    let mut address_decomp: Vec<Vec<u8>> = Vec::new();
    for i in 0..ADDRESS_PC_N1 {
        let mut v_inner = Vec::new();
        for j in 0..ADDRESS_PC_N2 {
            v_inner.push(ADDRESS_PC_DECOMP[i * ADDRESS_PC_N2 + j])
        }
        address_decomp.push(v_inner);
    }
    address_decomp
}

pub fn get_mem_address_decomp() -> Vec<Vec<u8>> {
    let mut address_decomp: Vec<Vec<u8>> = Vec::new();
    for i in 0..ADDRESS_MEM_N1 {
        let mut v_inner = Vec::new();
        for j in 0..ADDRESS_MEM_N2 {
            v_inner.push(ADDRESS_MEM_DECOMP[i * ADDRESS_MEM_N2 + j])
        }
        address_decomp.push(v_inner);
    }
    address_decomp
}

pub fn get_register_address_decomp() -> Vec<Vec<u8>> {
    vec![vec![5]]
}
