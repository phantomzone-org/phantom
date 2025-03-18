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
use base2k::Module;
use itertools::izip;

pub struct Interpreter {
    pub pc: Address,
    pub imm: Memory,
    pub instructions: Memory,
    pub register: Memory,
    pub memory: Memory,
    pub ret: bool,
}

impl Interpreter {
    pub fn new(module_lwe: &Module) -> Self {
        let log_n: usize = module_lwe.log_n();
        assert_eq!(
            log_n, LOGN_LWE,
            "invalid module_lwe: module_lwe.log_n()={} != LOGN_LWE={}",
            log_n, LOGN_LWE
        );
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
            register: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, 32),
            memory: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, MAXMEMORYADDRESS),
            ret: false,
        }
    }

    pub fn init_instructions(&mut self, instructions: InstructionsParser) {
        self.imm.set(&instructions.imm, LOGK);
        self.instructions.set(&instructions.instructions, LOGK);
    }

    pub fn init_registers(&mut self, registers: &[u32; 32]) {
        let mut registers_i64: [i64; 32] = [0i64; 32];
        izip!(registers_i64.iter_mut(), registers.iter()).for_each(|(a, b)| *a = *b as i64);
        self.register.set(&registers_i64[..], LOGK);
    }

    pub fn init_memory(&mut self, memory: &Vec<u32>) {
        let mut memory_i64: Vec<i64> = vec![i64::default(); memory.len()];
        izip!(memory_i64.iter_mut(), memory.iter()).for_each(|(a, b)| *a = *b as i64);
        self.memory.set(&memory_i64[..], LOGK);
    }

    pub fn next(&mut self, module_pbs: &Module, module_lwe: &Module, tmp_bytes: &mut [u8]) {
        assert!(tmp_bytes.len() >= next_tmp_bytes(module_pbs, module_lwe));

        let circuit_btp: CircuitBootstrapper =
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

        let mut tmp_address_instructions: Address = Address::new(
            module_lwe,
            LOGN_DECOMP,
            MAXOPSADDRESS,
            VMPPMAT_ROWS,
            VMPPMAT_COLS,
        );

        let mut tmp_address_memory: Address = Address::new(
            module_lwe,
            LOGN_DECOMP,
            MAXMEMORYADDRESS,
            VMPPMAT_ROWS,
            VMPPMAT_COLS,
        );

        let mut tmp_address_register: Address =
            Address::new(module_lwe, LOGN_DECOMP, 32, VMPPMAT_ROWS, VMPPMAT_COLS);

        // 1) Retrieve LWE 8xu4 inputs (imm, rs2, rs1, pc)
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
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
            &circuit_btp,
            &mut decomposer_arithmetic,
            &self.register,
            rs2_u5,
            &mut tmp_address_instructions,
            tmp_bytes,
        );
        let rs1_lwe: [u8; 8] = get_input_from_register_lwe(
            module_pbs,
            module_lwe,
            &circuit_btp,
            &mut decomposer_arithmetic,
            &self.register,
            rs1_u5,
            &mut tmp_address_instructions,
            tmp_bytes,
        );

        let pc_lwe: [u8; 8] = get_pc_lwe(
            module_pbs,
            module_lwe,
            &mut decomposer_arithmetic,
            &self.pc,
            tmp_bytes,
        );

        // 2) RD UPDATE OPS
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        let mut rd_out: Vec<[u8; 8]> = vec![[0; 8]; RD_OPS_LIST.len() + LOAD_OPS_LIST.len()];

        RD_OPS_LIST.iter().for_each(|op| {
            let (idx, out) = op.apply(&imm_lwe, &rs1_lwe, &rs2_lwe, &pc_lwe);
            rd_out[idx] = out
        });

        // address read/write = x_rs1 + sext(imm)
        prepare_address(
            module_pbs,
            module_lwe,
            &imm_lwe,
            &rs1_lwe,
            &circuit_btp,
            &mut tmp_address_memory,
            tmp_bytes,
        );

        let loaded: [u8; 8] = load(
            module_lwe,
            &mut self.memory,
            &mut tmp_address_memory,
            tmp_bytes,
        );

        LOAD_OPS_LIST.iter().for_each(|op| {
            let (idx, out) = op.apply(&loaded);
            rd_out[idx] = out
        });

        let rd_lwe: [u8; 8] = rd_out[rd_u5 as usize]; // Select new RD

        // 3) UPDATE MEMORY
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        let mut mem_out: Vec<[u8; 8]> = vec![[0; 8]; STORE_OPS_LIST.len()];

        // Selects how to store the value
        STORE_OPS_LIST.iter().for_each(|op| {
            let (idx, out) = op.apply(&rd_lwe);
            mem_out[idx] = out
        });

        mem_out[0] = loaded; // if store op is identity

        let mem_lwe: [u8; 8] = mem_out[mem_w_u5 as usize];

        store(
            module_lwe,
            &mem_lwe,
            &mut self.memory,
            &mut tmp_address_memory,
            tmp_bytes,
        );

        // 4) UPDATE REGISTERS
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        tmp_address_register.set(module_lwe, rd_w_u5 as u32); // TODO: bootstrap address

        // dummy read
        self.register
            .read_prepare_write(module_lwe, &tmp_address_register, tmp_bytes);
        store(
            module_lwe,
            &rd_lwe,
            &mut self.register,
            &mut tmp_address_register,
            tmp_bytes,
        );

        // 5) UPDATE PC
        // >>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>
        let mut pc_out: Vec<[u8; 8]> = vec![[0; 8]; PC_OPS_LIST.len()];

        PC_OPS_LIST.iter().for_each(|op| {
            let (idx, out) = op.apply(&imm_lwe, &rs1_lwe, &rs2_lwe, &pc_lwe);
            pc_out[idx] = out
        });

        let pc_lwe: [u8; 8] = pc_out[pc_w_u5 as usize]; // Select new PC

        circuit_btp.bootstrap_to_address(
            module_pbs,
            module_lwe,
            reconstruct(&pc_lwe),
            &mut self.pc,
            tmp_bytes,
        );
    }
}

pub fn next_tmp_bytes(module_pbs: &Module, module_lwe: &Module) -> usize {
    read_tmp_bytes(module_lwe, RLWE_COLS, VMPPMAT_ROWS, VMPPMAT_COLS)
        + bootstrap_address_tmp_bytes(module_pbs, module_lwe, VMPPMAT_COLS)
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
    circuit_btp: &CircuitBootstrapper,
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
    circuit_btp.bootstrap_to_address(
        module_pbs,
        module_lwe,
        address as u32,
        tmp_address,
        tmp_bytes_bootstrap_address,
    );
    let value: u32 = registers.read(module_lwe, tmp_address, tmp_bytes_read);
    let value_u8: Vec<i64> = decomposer_arithmetic.decompose(module_pbs, value);
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
    circuit_btp: &CircuitBootstrapper,
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
    circuit_btp.bootstrap_to_address(
        module_pbs,
        module_lwe,
        idx_lwe,
        tmp_address,
        tmp_bytes_bootstrap_address,
    );
    registers.read(module_lwe, tmp_address, tmp_bytes_read)
}

pub fn get_lwe_tmp_bytes(module_pbs: &Module, module_lwe: &Module) -> usize {
    next_tmp_bytes(module_pbs, module_lwe)
}
