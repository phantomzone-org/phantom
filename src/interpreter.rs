//use crate::gadget::Gadget;
use crate::address::Address;
use crate::circuit_bootstrapping::CircuitBootstrapper;
use crate::memory::Memory;
use crate::parameters::{LOGN_DECOMP, VMPPMAT_COLS, VMPPMAT_ROWS};
use base2k::{Module, VecZnx, VecZnxOps};

pub struct Interpreter {
    pub log_k: usize,
    pub log_base2k: usize,
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
    pub fn new(
        module: &Module,
        log_base2k: usize,
        limbs: usize,
        max_counter: usize,
        max_address: usize,
        max_pc: usize,
    ) -> Self {
        let log_n: usize = module.log_n();
        let log_k: usize = log_base2k * limbs;
        Self {
            log_k: log_k,
            log_base2k: log_base2k,
            max_address: max_address,
            max_pc: max_pc,
            counter: Address::new(module, LOGN_DECOMP, max_counter, VMPPMAT_ROWS, limbs + 1),
            rs1_addresses: Memory::new(log_n, log_base2k, log_k),
            rs2_addresses: Memory::new(log_n, log_base2k, log_k),
            imm: Memory::new(log_n, log_base2k, log_k),
            op_id_register: Memory::new(log_n, log_base2k, log_k),
            op_it_memory: Memory::new(log_n, log_base2k, log_k),
            op_id_counter: Memory::new(log_n, log_base2k, log_k),
            rd: Memory::new(log_n, log_base2k, log_k),
            register: Memory::new(log_n, log_base2k, log_k),
            memory: Memory::new(log_n, log_base2k, log_k),
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

        let rs1_idx_lwe: i64 = self.rs1_addresses.read(module_lwe, &self.counter);
        let rs2_idx_lwe: i64 = self.rs2_addresses.read(module_lwe, &self.counter);
        let imm_lwe: i64 = self.imm.read(module_lwe, &self.counter);
        let op_id_register_lwe: i64 = self.op_id_register.read(module_lwe, &self.counter);
        let op_it_memory_lwe: i64 = self.op_it_memory.read(module_lwe, &self.counter);
        let op_id_counter_lwe: i64 = self.op_id_counter.read(module_lwe, &self.counter);

        let log_gap: usize = 6;
        let log_base2k = self.log_base2k;

        let circuit_bootstrapper: CircuitBootstrapper =
            CircuitBootstrapper::new(&module_pbs, module_lwe.log_n(), log_base2k, cols);

        let mut buf_pbs: VecZnx = module_pbs.new_vec_znx(cols);

        let mut rs1_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            rs1_idx_lwe as u32,
            &mut rs1_address,
            &mut buf_pbs,
        );

        let mut rs2_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            rs2_idx_lwe as u32,
            &mut rs2_address,
            &mut &mut buf_pbs,
        );

        let mut imm_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            imm_lwe as u32,
            &mut imm_address,
            &mut &mut buf_pbs,
        );

        let mut op_id_register_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            op_id_register_lwe as u32,
            &mut op_id_register_address,
            &mut &mut buf_pbs,
        );

        let mut op_it_memory_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            op_it_memory_lwe as u32,
            &mut op_it_memory_address,
            &mut &mut buf_pbs,
        );

        let mut op_id_counter_address: Address =
            Address::new(&module_lwe, log_n_decomp, self.max_address, rows, cols);
        circuit_bootstrapper.bootstrap_to_address(
            module_pbs,
            module_lwe,
            op_id_counter_lwe as u32,
            &mut op_id_counter_address,
            &mut &mut buf_pbs,
        );

        // Retrieves rs1 and rs2
        let rs1_lwe = self.register.read(module_lwe, &rs1_address);

        let rs2_lwe = self.register.read(module_lwe, &rs2_address);

        let imm_lwe = self.register.read(module_lwe, &imm_address);

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
            &mut &mut buf_pbs,
        );
    }
}
