//use crate::gadget::Gadget;
use crate::address::Address;
use crate::circuit_bootstrapping::{bootstrap_address_tmp_bytes, CircuitBootstrapper};
use crate::decompose::Decomposer;
use crate::instructions::decompose;
use crate::memory::{read_tmp_bytes, Memory};
use crate::parameters::{
    DECOMPOSE_ARITHMETIC, DECOMPOSE_INSTRUCTIONS, LOGBASE2K, LOGN_DECOMP, LOGN_LWE,
    MAXMEMORYADDRESS, MAXOPSADDRESS, RLWE_COLS, VMPPMAT_COLS, VMPPMAT_ROWS,
};
use base2k::Module;

pub struct Interpreter {
    pub pc: Address,
    pub imm: Memory,
    pub instructions: Memory,
    pub register: Memory,
    pub memory: Memory,
    pub ret: bool,
}

impl Interpreter {
    pub fn new(module: &Module) -> Self {
        let log_n: usize = module.log_n();
        assert_eq!(
            log_n, LOGN_LWE,
            "invalid module: module.log_n()={} != LOGN_LWE={}",
            log_n, LOGN_LWE
        );
        Self {
            pc: Address::new(
                module,
                LOGN_DECOMP,
                MAXOPSADDRESS,
                VMPPMAT_ROWS,
                VMPPMAT_COLS,
            ),
            imm: Memory::new(module, LOGBASE2K, RLWE_COLS, MAXOPSADDRESS),
            instructions: Memory::new(module, LOGBASE2K, RLWE_COLS, MAXOPSADDRESS),
            register: Memory::new(module, LOGBASE2K, RLWE_COLS, 32),
            memory: Memory::new(module, LOGBASE2K, RLWE_COLS, MAXMEMORYADDRESS),
            ret: false,
        }
    }

    pub fn next(&mut self, module_pbs: &Module, module_lwe: &Module, tmp_bytes: &mut [u8]) {
        assert!(tmp_bytes.len() >= next_tmp_bytes(module_pbs, module_lwe));

        let circuit_bootstrapper: CircuitBootstrapper =
            CircuitBootstrapper::new(&module_pbs, module_lwe.log_n(), LOGBASE2K, VMPPMAT_COLS);

        let mut decomposer_instructions: Decomposer = Decomposer::new(
            module_pbs,
            &DECOMPOSE_INSTRUCTIONS.to_vec(),
            LOGBASE2K,
            RLWE_COLS,
        );
        let mut decomposer_arithmetic: Decomposer = Decomposer::new(
            module_pbs,
            &DECOMPOSE_ARITHMETIC.to_vec(),
            LOGBASE2K,
            RLWE_COLS,
        );

        let (rs2_u5, rs1_u5, rd_u5, rd_w_u5, mem_w_u5, pc_w_u5) = get_instructions(
            module_pbs,
            module_lwe,
            &mut decomposer_instructions,
            &self.instructions,
            &self.pc,
            tmp_bytes,
        );

        // 1) Retrieve inputs (imm, rs2, rs1, pc)
        let mut tmp_address_input: Address = Address::new(
            module_lwe,
            LOGN_DECOMP,
            MAXOPSADDRESS,
            VMPPMAT_ROWS,
            VMPPMAT_COLS,
        );

        let imm_lwe: [u8; 8] = get_imm_lwe(
            module_pbs,
            module_lwe,
            &mut decomposer_arithmetic,
            &self.imm,
            &self.pc,
            tmp_bytes,
        );

        let rs2_lwe: [u8; 8] = get_input_from_register_lwe(
            module_pbs,
            module_lwe,
            &circuit_bootstrapper,
            &mut decomposer_arithmetic,
            &self.register,
            rs2_u5,
            &mut tmp_address_input,
            tmp_bytes,
        );
        let rs1_lwe: [u8; 8] = get_input_from_register_lwe(
            module_pbs,
            module_lwe,
            &circuit_bootstrapper,
            &mut decomposer_arithmetic,
            &self.register,
            rs1_u5,
            &mut tmp_address_input,
            tmp_bytes,
        );

        let pc_lwe: [u8; 8] = get_pc_lwe(
            module_pbs,
            module_lwe,
            &mut decomposer_arithmetic,
            &self.pc,
            tmp_bytes,
        );

        // 2) Evaluates all Arithmetic: apply(imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8];
        //let ari: Vec<_> = Vec::new();

        // 4) Evaluate all Types-B OPS f[i](rs1_lwe, rs2_lwe, imm_lwe, pc);

        // Packs all OPS, rs1, rs2, read register, read memory

        // Select from pack_ops

        // Updates program-counter
        let (tmp_bytes_read, tmp_bytes_bootstrap_address) = tmp_bytes.split_at_mut(read_tmp_bytes(
            module_lwe,
            RLWE_COLS,
            VMPPMAT_ROWS,
            VMPPMAT_COLS,
        ));
        let pc_offset: u32 = 0;
        circuit_bootstrapper.bootstrap_address(
            module_pbs,
            module_lwe,
            pc_offset,
            MAXOPSADDRESS,
            &mut self.pc,
            tmp_bytes_bootstrap_address,
        );
    }
}

pub fn next_tmp_bytes(module_pbs: &Module, module_lwe: &Module) -> usize {
    read_tmp_bytes(module_lwe, RLWE_COLS, VMPPMAT_ROWS, VMPPMAT_COLS)
        + bootstrap_address_tmp_bytes(module_pbs, module_lwe, RLWE_COLS)
}

pub fn get_pc_lwe(
    module_pbs: &Module,
    module_lwe: &Module,
    decomposer_arithmetic: &mut Decomposer,
    pc: &Address,
    tmp_bytes: &mut [u8],
) -> [u8; 8] {
    let log_k: usize = LOGBASE2K * (VMPPMAT_COLS - 1) - 5;
    let cols: usize = (log_k + LOGBASE2K - 1) / LOGBASE2K;
    let mut mem: Memory = Memory::new(module_lwe, LOGBASE2K, cols, MAXOPSADDRESS);
    let mut data: Vec<i64> = vec![i64::default(); MAXOPSADDRESS];
    data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);
    mem.set(&data, log_k);

    let pc_u32: u32 = mem.read(module_lwe, pc, tmp_bytes);
    let pc_u8: Vec<i64> = decomposer_arithmetic.decompose(module_pbs, pc_u32);
    [
        pc_u8[0] as u8,
        pc_u8[1] as u8,
        pc_u8[2] as u8,
        pc_u8[3] as u8,
        pc_u8[4] as u8,
        pc_u8[5] as u8,
        pc_u8[6] as u8,
        pc_u8[7] as u8,
    ]
}

pub fn get_imm_lwe(
    module_pbs: &Module,
    module_lwe: &Module,
    decomposer_arithmetic: &mut Decomposer,
    location: &Memory,
    pc: &Address,
    tmp_bytes: &mut [u8],
) -> [u8; 8] {
    let imm_u32: u32 = location.read(module_lwe, pc, tmp_bytes);
    let imm_u8: Vec<i64> = decomposer_arithmetic.decompose(module_pbs, imm_u32);
    [
        imm_u8[0] as u8,
        imm_u8[1] as u8,
        imm_u8[2] as u8,
        imm_u8[3] as u8,
        imm_u8[4] as u8,
        imm_u8[5] as u8,
        imm_u8[6] as u8,
        imm_u8[7] as u8,
    ]
}

pub fn get_instructions(
    module_pbs: &Module,
    module_lwe: &Module,
    instructions_decomposer: &mut Decomposer,
    location: &Memory,
    pc: &Address,
    tmp_bytes: &mut [u8],
) -> (u8, u8, u8, u8, u8, u8) {
    let (tmp_bytes_read, tmp_bytes_bootstrap_address) = tmp_bytes.split_at_mut(read_tmp_bytes(
        module_lwe,
        RLWE_COLS,
        VMPPMAT_ROWS,
        VMPPMAT_COLS,
    ));
    let instructions: u32 = location.read(module_lwe, pc, tmp_bytes_read);
    let ii: Vec<i64> = instructions_decomposer.decompose(module_pbs, instructions);
    (
        ii[0] as u8,
        ii[1] as u8,
        ii[2] as u8,
        ii[3] as u8,
        ii[4] as u8,
        ii[5] as u8,
    )
}

pub fn get_input_from_register_lwe(
    module_pbs: &Module,
    module_lwe: &Module,
    circuit_bootstrapper: &CircuitBootstrapper,
    decomposer_arithmetic: &mut Decomposer,
    registers: &Memory,
    address: u8,
    tmp_address: &mut Address,
    tmp_bytes: &mut [u8],
) -> [u8; 8] {
    let (tmp_bytes_read, tmp_bytes_bootstrap_address) = tmp_bytes.split_at_mut(read_tmp_bytes(
        module_lwe,
        RLWE_COLS,
        VMPPMAT_ROWS,
        VMPPMAT_COLS,
    ));
    circuit_bootstrapper.bootstrap_to_address(
        module_pbs,
        module_lwe,
        address as u32,
        tmp_address,
        tmp_bytes_bootstrap_address,
    );
    let value: u32 = registers.read(module_lwe, tmp_address, tmp_bytes_read);
    let value_u8 = decomposer_arithmetic.decompose(module_pbs, value);
    [
        value_u8[0] as u8,
        value_u8[1] as u8,
        value_u8[2] as u8,
        value_u8[3] as u8,
        value_u8[4] as u8,
        value_u8[5] as u8,
        value_u8[6] as u8,
        value_u8[7] as u8,
    ]
}

pub fn get_inputs(
    module_pbs: &Module,
    module_lwe: &Module,
    location: &Memory,
    registers: &Memory,
    pc: &Address,
    circuit_bootstrapper: &CircuitBootstrapper,
    tmp_address: &mut Address,
    tmp_bytes: &mut [u8],
) -> u32 {
    let (tmp_bytes_read, tmp_bytes_bootstrap_address) = tmp_bytes.split_at_mut(read_tmp_bytes(
        module_lwe,
        RLWE_COLS,
        VMPPMAT_ROWS,
        VMPPMAT_COLS,
    ));
    let idx_lwe: u32 = location.read(module_lwe, pc, tmp_bytes_read);
    circuit_bootstrapper.bootstrap_to_address(
        module_pbs,
        module_lwe,
        idx_lwe,
        tmp_address,
        tmp_bytes_bootstrap_address,
    );
    registers.read(module_lwe, tmp_address, tmp_bytes_read)
}

/*
pub fn get_opid(module_pbs: &Module, module_lwe: &Module, value: u32, address: &mut Address, circuit_bootstrapper: &CircuitBootstrapper, tmp_bytes: &mut [u8]){
    let (tmp_bytes_read, tmp_bytes_bootstrap_address) = tmp_bytes.split_at_mut(read_tmp_bytes(module_lwe, RLWE_COLS, VMPPMAT_ROWS, VMPPMAT_COLS));
    let op_id_register_lwe: u32 =
            self.op_id_register
                .read(module_lwe, &self.counter, &mut tmp_bytes_read);


    circuit_bootstrapper.bootstrap_to_address(
        module_pbs,
        module_lwe,
        op_id_register_lwe as u32,
        &mut op_id_register_address,
        &mut tmp_bytes_bootstrap_address,
    );
}
 */

pub fn get_lwe_tmp_bytes(module_pbs: &Module, module_lwe: &Module) -> usize {
    next_tmp_bytes(module_pbs, module_lwe)
}
