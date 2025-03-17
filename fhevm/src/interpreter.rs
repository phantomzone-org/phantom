//use crate::gadget::Gadget;
use crate::address::Address;
use crate::circuit_bootstrapping::{bootstrap_address_tmp_bytes, CircuitBootstrapper};
use crate::instructions::decompose;
use crate::instructions::r_type::add::Add;
use crate::memory::{read_tmp_bytes, Memory};
use crate::parameters::{
    LOGBASE2K, LOGN_DECOMP, LOGN_LWE, MAXMEMORYADDRESS, MAXOPSADDRESS, RLWE_COLS, VMPPMAT_COLS,
    VMPPMAT_ROWS,
};
use base2k::Module;

pub struct Interpreter {
    pub pc: Address,
    pub imm: Memory,
    pub rs1: Memory,
    pub rs2: Memory,
    pub rd: Memory,
    pub rd_w: Memory,
    pub mem_w: Memory,
    pub pc_w: Memory,
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
            rs1: Memory::new(module, LOGBASE2K, RLWE_COLS, MAXOPSADDRESS),
            rs2: Memory::new(module, LOGBASE2K, RLWE_COLS, MAXOPSADDRESS),
            rd: Memory::new(module, LOGBASE2K, RLWE_COLS, MAXOPSADDRESS),
            rd_w: Memory::new(module, LOGBASE2K, RLWE_COLS, MAXOPSADDRESS),
            mem_w: Memory::new(module, LOGBASE2K, RLWE_COLS, MAXOPSADDRESS),
            pc_w: Memory::new(module, LOGBASE2K, RLWE_COLS, MAXOPSADDRESS),
            register: Memory::new(module, LOGBASE2K, RLWE_COLS, 32),
            memory: Memory::new(module, LOGBASE2K, RLWE_COLS, MAXMEMORYADDRESS),
            ret: false,
        }
    }

    pub fn next(&mut self, module_pbs: &Module, module_lwe: &Module, tmp_bytes: &mut [u8]) {
        assert!(tmp_bytes.len() >= next_tmp_bytes(module_pbs, module_lwe));

        let circuit_bootstrapper: CircuitBootstrapper =
            CircuitBootstrapper::new(&module_pbs, module_lwe.log_n(), LOGBASE2K, VMPPMAT_COLS);

        // 1) Retrieve inputs
        let mut tmp_address_input: Address = Address::new(
            module_lwe,
            LOGN_DECOMP,
            MAXOPSADDRESS,
            VMPPMAT_ROWS,
            VMPPMAT_COLS,
        );
        let imm_lwe: [u8; 8] = decompose(get_inputs(
            module_pbs,
            module_lwe,
            &self.imm,
            &self.register,
            &self.pc,
            &circuit_bootstrapper,
            &mut tmp_address_input,
            tmp_bytes,
        ));
        let rs1_lwe: [u8; 8] = decompose(get_inputs(
            module_pbs,
            module_lwe,
            &self.rs2,
            &self.register,
            &self.pc,
            &circuit_bootstrapper,
            &mut tmp_address_input,
            tmp_bytes,
        ));
        let rs2_lwe: [u8; 8] = decompose(get_inputs(
            module_pbs,
            module_lwe,
            &self.rs1,
            &self.register,
            &self.pc,
            &circuit_bootstrapper,
            &mut tmp_address_input,
            tmp_bytes,
        ));

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
