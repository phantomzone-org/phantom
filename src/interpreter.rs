//use crate::gadget::Gadget;
use crate::address::Address;
use crate::circuit_bootstrapping::{bootstrap_address_tmp_bytes, CircuitBootstrapper};
use crate::memory::{read_tmp_bytes, Memory};
use crate::parameters::{
    LIMBS, LOGBASE2K, LOGN_DECOMP, LOGN_LWE, MAXMEMORYADDRESS, MAXOPSADDRESS, VMPPMAT_COLS,
    VMPPMAT_ROWS,
};
use base2k::Module;

pub struct Interpreter {
    pub max_address: usize,
    pub max_pc: usize,
    pub counter: Address,
    pub rs1_addresses: Memory,
    pub rs2_addresses: Memory,
    pub imm: Memory,
    pub op_id_register: Memory,
    pub op_it_memory: Memory,
    pub op_id_counter: Memory,
    pub rd: Memory,
    pub register: Memory,
    pub memory: Memory,
    pub ret: bool,
}

impl Interpreter {
    pub fn new(module: &Module, max_counter: usize, max_address: usize, max_pc: usize) -> Self {
        let log_n: usize = module.log_n();
        assert_eq!(
            log_n, LOGN_LWE,
            "invalid module: module.log_n()={} != LOGN_LWE={}",
            log_n, LOGN_LWE
        );
        Self {
            max_address: max_address,
            max_pc: max_pc,
            counter: Address::new(module, LOGN_DECOMP, max_counter, VMPPMAT_ROWS, VMPPMAT_COLS),
            rs1_addresses: Memory::new(module, LOGBASE2K, LIMBS, MAXOPSADDRESS),
            rs2_addresses: Memory::new(module, LOGBASE2K, LIMBS, MAXOPSADDRESS),
            imm: Memory::new(module, LOGBASE2K, LIMBS, MAXOPSADDRESS),
            op_id_register: Memory::new(module, LOGBASE2K, LIMBS, MAXOPSADDRESS),
            op_it_memory: Memory::new(module, LOGBASE2K, LIMBS, MAXOPSADDRESS),
            op_id_counter: Memory::new(module, LOGBASE2K, LIMBS, MAXOPSADDRESS),
            rd: Memory::new(module, LOGBASE2K, LIMBS, MAXOPSADDRESS),
            register: Memory::new(module, LOGBASE2K, LIMBS, 32),
            memory: Memory::new(module, LOGBASE2K, LIMBS, MAXMEMORYADDRESS),
            ret: false,
        }
    }

    pub fn init(&mut self, module: &Module) {
        self.counter.set(module, 0);
    }

    pub fn next(&mut self, module_pbs: &Module, module_lwe: &Module) {
        let log_n_decomp: usize = LOGN_DECOMP;
        let rows: usize = VMPPMAT_ROWS;
        let cols: usize = VMPPMAT_COLS;

        let mut tmp_bytes_read: Vec<u8> =
            vec![u8::default(); read_tmp_bytes(module_lwe, LIMBS, VMPPMAT_ROWS, VMPPMAT_COLS)];
        let mut tmp_bytes_bootstrap_address: Vec<u8> =
            vec![u8::default(); bootstrap_address_tmp_bytes(module_pbs, module_lwe, LIMBS)];

        let rs1_idx_lwe: u32 =
            self.rs1_addresses
                .read(module_lwe, &self.counter, &mut tmp_bytes_read);
        let rs2_idx_lwe: u32 =
            self.rs2_addresses
                .read(module_lwe, &self.counter, &mut tmp_bytes_read);
        let imm_lwe: u32 = self
            .imm
            .read(module_lwe, &self.counter, &mut tmp_bytes_read);
        let op_id_register_lwe: u32 =
            self.op_id_register
                .read(module_lwe, &self.counter, &mut tmp_bytes_read);
        let op_it_memory_lwe: u32 =
            self.op_it_memory
                .read(module_lwe, &self.counter, &mut tmp_bytes_read);
        let op_id_counter_lwe: u32 =
            self.op_id_counter
                .read(module_lwe, &self.counter, &mut tmp_bytes_read);

        //let log_gap: usize = 6;

        let circuit_bootstrapper: CircuitBootstrapper =
            CircuitBootstrapper::new(&module_pbs, module_lwe.log_n(), LOGBASE2K, cols);

        let mut rs1_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            rs1_idx_lwe as u32,
            &mut rs1_address,
            &mut tmp_bytes_bootstrap_address,
        );

        let mut rs2_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            rs2_idx_lwe as u32,
            &mut rs2_address,
            &mut tmp_bytes_bootstrap_address,
        );

        let mut imm_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            imm_lwe as u32,
            &mut imm_address,
            &mut tmp_bytes_bootstrap_address,
        );

        let mut op_id_register_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            op_id_register_lwe as u32,
            &mut op_id_register_address,
            &mut tmp_bytes_bootstrap_address,
        );

        let mut op_it_memory_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            op_it_memory_lwe as u32,
            &mut op_it_memory_address,
            &mut tmp_bytes_bootstrap_address,
        );

        let mut op_id_counter_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            op_id_counter_lwe as u32,
            &mut op_id_counter_address,
            &mut tmp_bytes_bootstrap_address,
        );

        // Retrieves rs1 and rs2
        let _rs1_lwe: u32 = self
            .register
            .read(module_lwe, &rs1_address, &mut tmp_bytes_read);

        let _rs2_lwe: u32 = self
            .register
            .read(module_lwe, &rs2_address, &mut tmp_bytes_read);

        let _imm_lwe: u32 = self
            .register
            .read(module_lwe, &imm_address, &mut tmp_bytes_read);

        // TODO Evaluates all OPS f[i](rs1_lwe, rs2_lwe, imm_lwe);

        // Packs all OPS, rs1, rs2, read register, read memory

        // Select from pack_ops

        // Updates program-counter
        let pc_offset: u32 = 0;
        circuit_bootstrapper.bootstrap_address(
            module_pbs,
            module_lwe,
            pc_offset,
            self.max_pc,
            &mut self.counter,
            &mut tmp_bytes_bootstrap_address,
        );
    }
}
