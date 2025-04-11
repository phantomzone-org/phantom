//use std::time::Instant;

//use crate::gadget::Gadget;
use crate::address::Address;
use crate::circuit_bootstrapping::{bootstrap_address_tmp_bytes, CircuitBootstrapper};
use crate::decompose::{Decomposer, Precomp};
use crate::instructions::memory::{
    extract_from_byte_offset, load, prepare_address_floor_byte_offset, store,
};
use crate::instructions::{
    decompose, reconstruct, InstructionsParser, LOAD_OPS_LIST, PC_OPS_LIST, RD_OPS_LIST,
};
use crate::memory::{read_tmp_bytes, Memory};
use crate::parameters::{
    Parameters, DECOMPOSE_ARITHMETIC, DECOMPOSE_BYTEOFFSET, DECOMPOSE_INSTRUCTIONS, LOGBASE2K,
    LOGK, MEM_SIZE_U8, RLWE_COLS, VMPPMAT_COLS, VMPPMAT_ROWS,
};
use base2k::{alloc_aligned, Encoding, Module, VecZnxOps};
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
    pub decomposer: Decomposer,
    pub precomp_decompose_pc: Precomp,
    pub precomp_decompose_instructions: Precomp,
    pub precomp_decompose_arithmetic: Precomp,
    pub precomp_decompose_byte_offset: Precomp,
    pub precomp_decompose_memory: Precomp,
    pub precomp_decompose_register: Precomp,
    pub tmp_bytes: Vec<u8>,
    pub tmp_address_instructions: Address,
    pub tmp_address_memory: Address,
    pub tmp_address_register: Address,
    pub tmp_address_memory_state: bool,
}

impl Interpreter {
    pub fn new(params: &Parameters) -> Self {
        let module_lwe: &Module = params.module_lwe();
        let module_pbs: &Module = params.module_pbs();
        let pc_max_adr: usize = params.pc_max();
        let log_k: usize = LOGBASE2K * (VMPPMAT_COLS - 1) - 5;
        let cols: usize = (log_k + LOGBASE2K - 1) / LOGBASE2K;
        let mut pc_recomposition: Memory = Memory::new(module_lwe, LOGBASE2K, cols, pc_max_adr);
        let mut data: Vec<i64> = vec![i64::default(); pc_max_adr];
        data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);
        pc_recomposition.set(&data, log_k);

        Self {
            pc: Address::new(module_lwe, &params.pc_decomp, VMPPMAT_ROWS, VMPPMAT_COLS),
            imm: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, pc_max_adr),
            instructions: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, pc_max_adr),
            registers: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, params.registers_max()),
            memory: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, params.mem_max()),
            ret: false,
            ram_offset: 0,
            pc_recomposition: pc_recomposition,
            circuit_btp: CircuitBootstrapper::new(LOGBASE2K, VMPPMAT_COLS),
            decomposer: Decomposer::new(params.module_pbs(), RLWE_COLS),
            precomp_decompose_pc: Precomp::new(
                module_pbs.n(),
                &params.pc_decomp.basis_1d(),
                LOGBASE2K,
                RLWE_COLS,
            ),
            precomp_decompose_instructions: Precomp::new(
                module_pbs.n(),
                &DECOMPOSE_INSTRUCTIONS.to_vec(),
                LOGBASE2K,
                RLWE_COLS,
            ),
            precomp_decompose_arithmetic: Precomp::new(
                module_pbs.n(),
                &DECOMPOSE_ARITHMETIC.to_vec(),
                LOGBASE2K,
                RLWE_COLS,
            ),
            precomp_decompose_byte_offset: Precomp::new(
                module_pbs.n(),
                &DECOMPOSE_BYTEOFFSET.to_vec(),
                LOGBASE2K,
                RLWE_COLS,
            ),
            precomp_decompose_memory: Precomp::new(
                module_pbs.n(),
                &params.mem_decomp.basis_1d(),
                LOGBASE2K,
                RLWE_COLS,
            ),
            precomp_decompose_register: Precomp::new(
                module_pbs.n(),
                &params.registers_decomp.basis_1d(),
                LOGBASE2K,
                RLWE_COLS,
            ),
            tmp_bytes: alloc_aligned(next_tmp_bytes(module_pbs, module_lwe)),
            tmp_address_instructions: Address::new(
                module_lwe,
                &params.pc_decomp,
                VMPPMAT_ROWS,
                VMPPMAT_COLS,
            ),
            tmp_address_memory: Address::new(
                module_lwe,
                &params.mem_decomp,
                VMPPMAT_ROWS,
                VMPPMAT_COLS,
            ),
            tmp_address_memory_state: false,
            tmp_address_register: Address::new(
                module_lwe,
                &params.registers_decomp,
                VMPPMAT_ROWS,
                VMPPMAT_COLS,
            ),
        }
    }

    pub fn init_pc(&mut self, params: &Parameters) {
        self.pc.set(params.module_lwe(), 0);
    }

    pub fn init_instructions(&mut self, instructions: InstructionsParser) {
        self.imm.set(&instructions.imm, LOGK);
        self.instructions.set(&instructions.instructions, LOGK);
    }

    pub fn init_registers(&mut self, registers: &Vec<u32>) {
        let mut registers_i64: Vec<i64> = vec![0i64; registers.len()];
        izip!(registers_i64.iter_mut(), registers.iter()).for_each(|(a, b)| *a = *b as i64);
        self.registers.set(&registers_i64, LOGK);
    }

    pub fn init_memory(&mut self, memory: &Vec<u32>) {
        let mut memory_i64: Vec<i64> = vec![i64::default(); memory.len()];
        izip!(memory_i64.iter_mut(), memory.iter()).for_each(|(a, b)| *a = *b as i64);
        self.memory.set(&memory_i64[..], LOGK);
    }

    pub fn cycle(&mut self, params: &Parameters) {
        //println!("pc: {}", self.pc.debug_as_u32(params.module_lwe())<<2);
        //println!("REGS: {:?}", &self.registers.debug_as_u32());

        let module_lwe: &Module = params.module_lwe();
        let module_pbs: &Module = params.module_pbs();

        // 0) Fetches instructions selectors
        //let now: Instant = Instant::now();
        let (rs2_u5, rs1_u5, rd_u5, rd_w_u6, mem_w_u5, pc_w_u5) =
            self.get_instruction_selectors(module_pbs, module_lwe);
        //println!(
        //    "get_instruction_selectors: {} ms",
        //    now.elapsed().as_millis()
        //);

        //println!("rd_w_u6: {}", rd_w_u6);
        //println!("mem_w_u5: {}", mem_w_u5);
        //println!("pc_w_u5: {}", pc_w_u5);
        //println!("rs1_u5: {}", rs1_u5);
        //println!("rs2_u5: {}", rs2_u5);
        //println!("rd_u5: {}", rd_u5);

        // 1) Retrieve 8xLWE(u4) inputs (imm, rs2, rs1, pc)
        //let now: Instant = Instant::now();
        let (imm_lwe, rs2_lwe, rs1_lwe, pc_lwe) =
            self.get_lwe_inputs(module_pbs, module_lwe, rs2_u5, rs1_u5);
        //println!(
        //    "get_lwe_inputs           : {} ms",
        //    now.elapsed().as_millis()
        //);

        //println!("rs2_lwe: {}", reconstruct(&rs2_lwe));
        //println!("rs1_lwe: {}", reconstruct(&rs1_lwe));
        //println!("imm_lwe: {}", reconstruct(&imm_lwe));

        // 2) Prepares memory address read/write (x_rs1 + sext(imm) - offset) where offset = (x_rs1 + sext(imm))%4
        //let now: Instant = Instant::now();
        let offset: u8 = self.prepare_memory_address_floor_byte_offset(
            module_pbs,
            module_lwe,
            &imm_lwe,
            &rs1_lwe,
            MEM_SIZE_U8,
        );
        //println!(
        //    "prepare_memory_address   : {} ms",
        //    now.elapsed().as_millis()
        //);

        //println!("offset: {}", offset);

        // 3)  loads value from memory
        //let now: Instant = Instant::now();
        let loaded: [u8; 8] = self.read_memory(module_lwe);
        //println!(
        //    "read_memory              : {} ms",
        //    now.elapsed().as_millis()
        //);

        //println!("loaded_full: {:08x}", reconstruct(&loaded));

        // Selects [4, 2, 1] bytes from loaded value
        // according to offset.
        let loaded_offset: [u8; 8] = extract_from_byte_offset(&loaded, offset);

        // 4) Retrieves RD value from OPS(imm, rs1, rs2, pc, loaded)[rd_w_u6]
        //let now: Instant = Instant::now();
        let rd_lwe: [u8; 8] = self.evaluate_ops(
            &imm_lwe,
            &rs1_lwe,
            &rs2_lwe,
            &pc_lwe,
            &loaded_offset,
            rd_w_u6,
        );
        //println!(
        //    "evaluate_ops             : {} ms",
        //    now.elapsed().as_millis()
        //);

        // 5) Updates memory from {RD|LOADED}[mem_w_u5]
        //let now: Instant = Instant::now();
        self.store_memory(module_lwe, &rs2_lwe, &loaded, offset, mem_w_u5);
        //println!(
        //    "store_memory             : {} ms",
        //    now.elapsed().as_millis()
        //);

        // 6) Updates registers from RD
        //let now: Instant = Instant::now();
        self.store_registers(module_lwe, &rd_lwe, rd_u5);
        //println!(
        //    "store_registers          : {} ms",
        //    now.elapsed().as_millis()
        //);

        // 7) Update PC from OPS(imm, rs1, rs2, pc)[pc_w_u5]
        //let now: Instant = Instant::now();
        self.update_pc(
            module_pbs, module_lwe, &imm_lwe, &rs1_lwe, &rs2_lwe, &pc_lwe, pc_w_u5,
        );
        //println!(
        //    "update_pc                : {} ms",
        //    now.elapsed().as_millis()
        //);

        // Reinitialize checks
        self.tmp_address_memory_state = false;

        //println!("pc_out: {}", self.pc.debug_as_u32(params.module_lwe()));
        //println!();
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

    fn read_memory(&mut self, module_lwe: &Module) -> [u8; 8] {
        assert_eq!(
            self.tmp_address_memory_state, true,
            "trying to read memory but memory address hasn't been prepared"
        );
        //println!("address: {}", self.tmp_address_memory.debug_as_u32(module_lwe));
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
        rs2_lwe: &[u8; 8],
        loaded: &[u8; 8],
        offset: u8,
        mem_w_u5: u8,
    ) {
        assert_eq!(
            self.tmp_address_memory_state, true,
            "trying to store in memory but tmp_address_memory_state is false"
        );

        // Creates a list with all possible values to store.
        // rs2 = [a, b, c, d]
        // loaded = [e, f, g, h]

        // offset = 0
        // NONE: [e, f, g, h]
        // SB:   [a, f, g, h]
        // SH:   [a, b, g, h]
        // SW:   [a, b, c, d]
        //
        // offset = 1
        // NONE: [e, f, g, h]
        // SB:   [e, a, g, h]
        // SH:   [e, a, b, h]
        // SW:   [ INVALID  ]
        //
        // offset = 2
        // NONE: [e, f, g, h]
        // SB:   [e, f, g, a]
        // SH:   [e, f, a, b]
        // SW:   [ INVALID  ]
        //
        // offset = 3
        // NONE: [e, f, g, h]
        // SB:   [e, f, g, a]
        // SH:   [ INVALID  ]
        // SW:   [ INVALID  ]

        let list: [[u8; 8]; 16] = [
            *loaded,
            [
                rs2_lwe[0], rs2_lwe[1], loaded[2], loaded[3], loaded[4], loaded[5], loaded[6],
                loaded[7],
            ],
            [
                rs2_lwe[0], rs2_lwe[1], rs2_lwe[2], rs2_lwe[3], loaded[4], loaded[5], loaded[6],
                loaded[7],
            ],
            *rs2_lwe,
            *loaded,
            [
                loaded[0], loaded[1], rs2_lwe[0], rs2_lwe[1], loaded[4], loaded[5], loaded[6],
                loaded[7],
            ],
            [
                loaded[0], loaded[1], rs2_lwe[0], rs2_lwe[1], rs2_lwe[2], rs2_lwe[3], loaded[6],
                loaded[7],
            ],
            [0, 0, 0, 0, 0, 0, 0, 0],
            *loaded,
            [
                loaded[0], loaded[1], loaded[2], loaded[3], rs2_lwe[0], rs2_lwe[1], loaded[6],
                loaded[7],
            ],
            [
                loaded[0], loaded[1], loaded[2], loaded[3], rs2_lwe[0], rs2_lwe[1], rs2_lwe[2],
                rs2_lwe[3],
            ],
            [0, 0, 0, 0, 0, 0, 0, 0],
            *loaded,
            [
                loaded[0], loaded[1], loaded[2], loaded[3], loaded[4], loaded[5], rs2_lwe[0],
                rs2_lwe[1],
            ],
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0],
        ];

        // Creates a vector of VecZnx storing list.
        let mut vec_znx: base2k::VecZnx = module_lwe.new_vec_znx(RLWE_COLS);
        list.iter().enumerate().for_each(|(i, x)| {
            vec_znx.encode_coeff_i64(LOGBASE2K, LOGK, i, reconstruct(x) as i64, 32);
        });

        // Selects according to offset
        // TODO: BOOTSTRAP OFFSET TO RGSW
        module_lwe
            .vec_znx_rotate_inplace(-(((offset as i64) << 2) + mem_w_u5 as i64), &mut vec_znx);

        // Sample extract
        let value: [u8; 8] = decompose(vec_znx.decode_coeff_i64(LOGBASE2K, LOGK, 0) as u32);

        store(
            module_lwe,
            &value,
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
            &mut self.decomposer,
            &self.precomp_decompose_pc,
            reconstruct(&pc_lwe) >> 2, // TODO: HE DIV by 4
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

    fn prepare_memory_address_floor_byte_offset(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm_lwe: &[u8; 8],
        rs1_lwe: &[u8; 8],
        max_size: u32,
    ) -> u8 {
        assert_eq!(
            self.tmp_address_memory_state, false,
            "trying to prepare address rs1 + imm but state indicates it has already been done"
        );
        let offset: u8 = prepare_address_floor_byte_offset(
            module_pbs,
            module_lwe,
            imm_lwe,
            rs1_lwe,
            max_size,
            &self.circuit_btp,
            &mut self.decomposer,
            &self.precomp_decompose_byte_offset,
            &self.precomp_decompose_memory,
            &mut self.tmp_address_memory,
            &mut self.tmp_bytes,
        );
        self.tmp_address_memory_state = true;
        offset
    }

    // Recompose PC ADDRESS to PC LWE and shifts by 2.
    fn get_pc_lwe(&mut self, module_pbs: &Module, module_lwe: &Module) -> [u8; 8] {
        let pc_u32: u32 = self
            .pc_recomposition
            .read(module_lwe, &self.pc, &mut self.tmp_bytes);
        decompose_1xu32_to_8xu4(
            module_pbs,
            &mut self.decomposer,
            &self.precomp_decompose_arithmetic,
            pc_u32 << 2,
        )
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
        let tmp_address = &mut self.tmp_address_register;
        self.circuit_btp.bootstrap_to_address(
            module_pbs,
            module_lwe,
            &mut self.decomposer,
            &self.precomp_decompose_register,
            address as u32,
            tmp_address,
            tmp_bytes_bootstrap_address,
        );
        let value: u32 = self.registers.read(module_lwe, tmp_address, tmp_bytes_read);
        decompose_1xu32_to_8xu4(
            module_pbs,
            &mut self.decomposer,
            &self.precomp_decompose_arithmetic,
            value,
        )
    }

    fn get_instruction_selectors(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
    ) -> (u8, u8, u8, u8, u8, u8) {
        let (tmp_bytes_read, _) = self.tmp_bytes.split_at_mut(read_tmp_bytes(
            module_lwe,
            RLWE_COLS,
            VMPPMAT_ROWS,
            VMPPMAT_COLS,
        ));
        let instructions: u32 = self.instructions.read(module_lwe, &self.pc, tmp_bytes_read);
        let selector: Vec<u8> = self.decomposer.decompose(
            module_pbs,
            &self.precomp_decompose_instructions,
            instructions as u32,
        );
        (
            selector[5], // rs2_u5
            selector[4], // rs1_u5
            selector[3], // rd_u5
            selector[2], // rd_w_u6
            selector[1], // mem_w_u5
            selector[0], // pc_w_u5
        )
    }

    fn get_imm_lwe(&mut self, module_pbs: &Module, module_lwe: &Module) -> [u8; 8] {
        let imm_u32: u32 = self.imm.read(module_lwe, &self.pc, &mut self.tmp_bytes);
        decompose_1xu32_to_8xu4(
            module_pbs,
            &mut self.decomposer,
            &self.precomp_decompose_arithmetic,
            imm_u32,
        )
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
    precomp: &Precomp,
    value: u32,
) -> [u8; 8] {
    let value_u8: Vec<u8> = decomposer.decompose(module_pbs, precomp, value);
    [
        value_u8[0],
        value_u8[1],
        value_u8[2],
        value_u8[3],
        value_u8[4],
        value_u8[5],
        value_u8[6],
        value_u8[7],
    ]
}
