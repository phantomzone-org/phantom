//use crate::gadget::Gadget;
use crate::address::Address;
use crate::circuit_bootstrapping::{bootstrap_address_tmp_bytes, CircuitBootstrapper};
use crate::decompose::Decomposer;
use crate::instructions::memory::{load, prepare_address, store};
use crate::instructions::{
    decompose, reconstruct, InstructionsParser, LOAD_OPS_LIST, PC_OPS_LIST, RD_OPS_LIST,
    STORE_OPS_LIST,
};
use crate::memory::{read_tmp_bytes, Memory};
use crate::parameters::{
    DECOMPOSE_ARITHMETIC, DECOMPOSE_INSTRUCTIONS, LOGBASE2K, LOGK, LOGN_DECOMP, LOGN_LWE,
    MAXMEMORYADDRESS, MAXOPSADDRESS, RLWE_COLS, VMPPMAT_COLS, VMPPMAT_ROWS,
};
use base2k::{alloc_aligned, Module};
use itertools::izip;

pub struct Interpreter {
    pub pc: Address,
    pub imm: Memory,
    pub instructions: Memory,
    pub registers: Memory,
    pub memory: Memory,
    pub ret: bool,
    pub ram_offset: u32,
    pub pc_recomposition: Memory,
    pub circuit_btp: CircuitBootstrapper,
    pub decomposer_instructions: Decomposer,
    pub decomposer_arithmetic: Decomposer,
    pub tmp_bytes: Vec<u8>,
    pub tmp_address_instructions: Address,
    pub tmp_address_memory: Address,
    pub tmp_address_register: Address,

    pub tmp_address_memory_state: bool,
}

impl Interpreter {
    pub fn new(module_pbs: &Module, module_lwe: &Module) -> Self {
        let log_n: usize = module_lwe.log_n();
        assert_eq!(
            log_n, LOGN_LWE,
            "invalid module_lwe: module_lwe.log_n()={} != LOGN_LWE={}",
            log_n, LOGN_LWE
        );

        let log_k: usize = LOGBASE2K * (VMPPMAT_COLS - 1) - 5;
        let cols: usize = (log_k + LOGBASE2K - 1) / LOGBASE2K;
        let mut pc_recomposition: Memory = Memory::new(module_lwe, LOGBASE2K, cols, MAXOPSADDRESS);
        let mut data: Vec<i64> = vec![i64::default(); MAXOPSADDRESS];
        data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);
        pc_recomposition.set(&data, log_k);

        Self {
            pc: Address::new(
                module_lwe,
                LOGN_DECOMP,
                MAXOPSADDRESS,
                VMPPMAT_ROWS,
                VMPPMAT_COLS,
            ),
            imm: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, MAXOPSADDRESS),
            instructions: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, MAXOPSADDRESS),
            registers: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, 32),
            memory: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, MAXMEMORYADDRESS),
            ret: false,
            ram_offset: 0,
            pc_recomposition: pc_recomposition,
            circuit_btp: CircuitBootstrapper::new(
                &module_pbs,
                module_lwe.log_n(),
                LOGBASE2K,
                VMPPMAT_COLS,
            ),
            decomposer_arithmetic: Decomposer::new(
                module_pbs,
                &DECOMPOSE_ARITHMETIC.to_vec(),
                LOGBASE2K,
                RLWE_COLS,
            ),
            decomposer_instructions: Decomposer::new(
                module_pbs,
                &DECOMPOSE_INSTRUCTIONS.to_vec(),
                LOGBASE2K,
                RLWE_COLS,
            ),
            tmp_bytes: alloc_aligned(next_tmp_bytes(module_pbs, module_lwe)),
            tmp_address_instructions: Address::new(
                module_lwe,
                LOGN_DECOMP,
                MAXOPSADDRESS,
                VMPPMAT_ROWS,
                VMPPMAT_COLS,
            ),
            tmp_address_memory: Address::new(
                module_lwe,
                LOGN_DECOMP,
                MAXMEMORYADDRESS,
                VMPPMAT_ROWS,
                VMPPMAT_COLS,
            ),
            tmp_address_memory_state: false,
            tmp_address_register: Address::new(
                module_lwe,
                LOGN_DECOMP,
                32,
                VMPPMAT_ROWS,
                VMPPMAT_COLS,
            ),
        }
    }

    pub fn init_pc(&mut self, module_lwe: &Module) {
        self.pc.set(module_lwe, 0);
    }

    pub fn init_instructions(&mut self, instructions: InstructionsParser) {
        self.imm.set(&instructions.imm, LOGK);
        self.instructions.set(&instructions.instructions, LOGK);
    }

    pub fn init_registers(&mut self, registers: &[u32; 32]) {
        let mut registers_i64: [i64; 32] = [0i64; 32];
        izip!(registers_i64.iter_mut(), registers.iter()).for_each(|(a, b)| *a = *b as i64);
        self.registers.set(&registers_i64[..], LOGK);
    }

    pub fn init_memory(&mut self, memory: &Vec<u32>) {
        let mut memory_i64: Vec<i64> = vec![i64::default(); memory.len()];
        izip!(memory_i64.iter_mut(), memory.iter()).for_each(|(a, b)| *a = *b as i64);
        self.memory.set(&memory_i64[..], LOGK);
    }

    pub fn step(&mut self, module_pbs: &Module, module_lwe: &Module) {
        // 0) Fetches instructions selectors
        let (rs2_u5, rs1_u5, rd_u5, rd_w_u6, mem_w_u5, pc_w_u5) =
            self.get_instructions(module_pbs, module_lwe);

        // 1) Retrieve 8xLWE(u4) inputs (imm, rs2, rs1, pc)
        let (imm_lwe, rs2_lwe, rs1_lwe, pc_lwe) =
            self.get_lwe_inputs(module_pbs, module_lwe, rs2_u5, rs1_u5);

        // 2) Prepares memory address address read/write (x_rs1 + sext(imm)) & loads value from memory
        let loaded: [u8; 8] = self.read_memory(module_pbs, module_lwe, &imm_lwe, &rs1_lwe);

        // 3) Retrieves RD value from OPS(imm, rs1, rs2, pc, loaded)[rd_w_u6]
        let rd_lwe: [u8; 8] =
            self.evaluate_ops(&imm_lwe, &rs1_lwe, &rs2_lwe, &pc_lwe, &loaded, rd_w_u6);

        // 4) Updates memory from {RD|LOADED}[mem_w_u5]
        self.store_memory(module_lwe, &rd_lwe, &loaded, mem_w_u5);

        // 5) Updates registers from RD
        self.store_registers(module_lwe, &rd_lwe, rd_u5);

        // 6) Update PC from OPS(imm, rs1, rs2, pc)[pc_w_u5]
        self.update_pc(
            module_pbs, module_lwe, &imm_lwe, &rs1_lwe, &rs2_lwe, &pc_lwe, pc_w_u5,
        );

        // Reinitialize checks
        self.tmp_address_memory_state = false;
    }

    fn evaluate_ops(
        &mut self,
        imm_lwe: &[u8; 8],
        rs1_lwe: &[u8; 8],
        rs2_lwe: &[u8; 8],
        pc_lwe: &[u8; 8],
        loaded: &[u8; 8],
        rd_w_u6: u8,
    ) -> [u8; 8] {
        let mut rd_out: Vec<[u8; 8]> = vec![[0; 8]; RD_OPS_LIST.len() + LOAD_OPS_LIST.len()];

        // Evaluates all arithmetic operations
        RD_OPS_LIST.iter().for_each(|op| {
            let (idx, out) = op.apply(imm_lwe, rs1_lwe, rs2_lwe, pc_lwe);
            rd_out[idx] = out
        });

        // Selects correct loading mode
        LOAD_OPS_LIST.iter().for_each(|op| {
            let (idx, out) = op.apply(loaded);
            rd_out[idx] = out
        });

        rd_out[rd_w_u6 as usize]
    }

    fn read_memory(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm_lwe: &[u8; 8],
        rs1_lwe: &[u8; 8],
    ) -> [u8; 8] {
        self.prepare_memory_address(module_pbs, module_lwe, imm_lwe, rs1_lwe);
        load(
            module_lwe,
            &mut self.memory,
            &mut self.tmp_address_memory,
            &mut self.tmp_bytes,
        )
    }

    fn store_memory(
        &mut self,
        module_lwe: &Module,
        rd_lwe: &[u8; 8],
        loaded: &[u8; 8],
        mem_w_u5: u8,
    ) {
        assert_eq!(
            self.tmp_address_memory_state, true,
            "trying to store in memory but tmp_address_memory_state is false"
        );

        let mut mem_out: Vec<[u8; 8]> = vec![[0; 8]; STORE_OPS_LIST.len()];

        // Selects how to store the value
        STORE_OPS_LIST.iter().for_each(|op| {
            let (idx, out) = op.apply(rd_lwe);
            mem_out[idx] = out
        });

        mem_out[0] = *loaded; // if store op is identity

        let mem_lwe: [u8; 8] = mem_out[mem_w_u5 as usize];

        store(
            module_lwe,
            &mem_lwe,
            &mut self.memory,
            &mut self.tmp_address_memory,
            &mut self.tmp_bytes,
        );
    }

    fn get_lwe_inputs(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
        rs2_u5: u8,
        rs1_u5: u8,
    ) -> ([u8; 8], [u8; 8], [u8; 8], [u8; 8]) {
        let imm_lwe: [u8; 8] = self.get_imm_lwe(module_pbs, module_lwe);

        let rs2_lwe: [u8; 8] = self.get_input_from_register_lwe(module_pbs, module_lwe, rs2_u5);
        let rs1_lwe: [u8; 8] = self.get_input_from_register_lwe(module_pbs, module_lwe, rs1_u5);

        let pc_lwe: [u8; 8] = self.get_pc_lwe(module_pbs, module_lwe);

        (imm_lwe, rs2_lwe, rs1_lwe, pc_lwe)
    }

    fn update_pc(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm_lwe: &[u8; 8],
        rs1_lwe: &[u8; 8],
        rs2_lwe: &[u8; 8],
        pc_lwe: &[u8; 8],
        pc_w_u5: u8,
    ) {
        let mut pc_out: Vec<[u8; 8]> = vec![[0; 8]; PC_OPS_LIST.len()];

        PC_OPS_LIST.iter().for_each(|op| {
            let (idx, out) = op.apply(imm_lwe, rs1_lwe, rs2_lwe, pc_lwe);
            pc_out[idx] = out
        });

        let pc_lwe: [u8; 8] = pc_out[pc_w_u5 as usize]; // Select new PC

        self.circuit_btp.bootstrap_to_address(
            module_pbs,
            module_lwe,
            reconstruct(&pc_lwe),
            &mut self.pc,
            &mut self.tmp_bytes,
        );
    }

    fn store_registers(&mut self, module_lwe: &Module, rd_lwe: &[u8; 8], rd_u5: u8) {
        self.tmp_address_register.set(module_lwe, rd_u5 as u32); // TODO: bootstrap address
        self.registers.read_prepare_write(
            module_lwe,
            &self.tmp_address_register,
            &mut self.tmp_bytes,
        );
        store(
            module_lwe,
            rd_lwe,
            &mut self.registers,
            &mut self.tmp_address_register,
            &mut self.tmp_bytes,
        );
    }

    fn prepare_memory_address(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm_lwe: &[u8; 8],
        rs1_lwe: &[u8; 8],
    ) {
        assert_eq!(
            self.tmp_address_memory_state, false,
            "trying to prepare address rs1 + imm but state indicates it has already been done"
        );
        prepare_address(
            module_pbs,
            module_lwe,
            imm_lwe,
            rs1_lwe,
            &self.circuit_btp,
            &mut self.tmp_address_memory,
            &mut self.tmp_bytes,
        );
        self.tmp_address_memory_state = true;
    }

    fn get_pc_lwe(&mut self, module_pbs: &Module, module_lwe: &Module) -> [u8; 8] {
        let pc_u32: u32 = self
            .pc_recomposition
            .read(module_lwe, &self.pc, &mut self.tmp_bytes);
        decompose_1xu32_to_8xu4(module_pbs, &mut self.decomposer_arithmetic, pc_u32)
    }

    fn get_input_from_register_lwe(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
        address: u8,
    ) -> [u8; 8] {
        let (tmp_bytes_read, tmp_bytes_bootstrap_address) = self.tmp_bytes.split_at_mut(
            read_tmp_bytes(module_lwe, RLWE_COLS, VMPPMAT_ROWS, VMPPMAT_COLS),
        );
        let tmp_address = &mut self.tmp_address_instructions;
        self.circuit_btp.bootstrap_to_address(
            module_pbs,
            module_lwe,
            address as u32,
            tmp_address,
            tmp_bytes_bootstrap_address,
        );
        let value: u32 = self.registers.read(module_lwe, tmp_address, tmp_bytes_read);
        decompose_1xu32_to_8xu4(module_pbs, &mut self.decomposer_arithmetic, value)
    }

    fn get_instructions(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
    ) -> (u8, u8, u8, u8, u8, u8) {
        let (tmp_bytes_read, tmp_bytes_bootstrap_address) = self.tmp_bytes.split_at_mut(
            read_tmp_bytes(module_lwe, RLWE_COLS, VMPPMAT_ROWS, VMPPMAT_COLS),
        );
        let instructions: u32 = self.instructions.read(module_lwe, &self.pc, tmp_bytes_read);
        let ii: Vec<i64> = self
            .decomposer_instructions
            .decompose(module_pbs, instructions);
        (
            ii[5] as u8,
            ii[4] as u8,
            ii[3] as u8,
            ii[2] as u8,
            ii[1] as u8,
            ii[0] as u8,
        )
    }

    fn get_imm_lwe(&mut self, module_pbs: &Module, module_lwe: &Module) -> [u8; 8] {
        let imm_u32: u32 = self.imm.read(module_lwe, &self.pc, &mut self.tmp_bytes);
        decompose_1xu32_to_8xu4(module_pbs, &mut self.decomposer_arithmetic, imm_u32)
    }
}

pub fn next_tmp_bytes(module_pbs: &Module, module_lwe: &Module) -> usize {
    read_tmp_bytes(module_lwe, RLWE_COLS, VMPPMAT_ROWS, VMPPMAT_COLS)
        + bootstrap_address_tmp_bytes(module_pbs, module_lwe, VMPPMAT_COLS)
}

pub fn get_lwe_tmp_bytes(module_pbs: &Module, module_lwe: &Module) -> usize {
    next_tmp_bytes(module_pbs, module_lwe)
}

pub fn decompose_1xu32_to_8xu4(
    module_pbs: &Module,
    decomposer: &mut Decomposer,
    value: u32,
) -> [u8; 8] {
    let value_u8: Vec<i64> = decomposer.decompose(module_pbs, value);
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
