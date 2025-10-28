use fhe_ram::{Address, CryptographicParameters, EvaluationKeys, EvaluationKeysPrepared, Parameters, Ram};
use poulpy_backend::FFT64Ref as BackendImpl;
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow, ScratchTakeBasic}, layouts::{Backend, Module, Scratch, ScratchOwned, ZnxView}, source::Source
};

use poulpy_core::{GLWEDecrypt, layouts::{GLWE, GLWEPlaintext, GLWESecret, GLWESecretPrepared, LWE, LWEInfos, LWELayout, LWEPlaintext, LWEPlaintextLayout, LWESecret}};
use poulpy_schemes::tfhe::bdd_arithmetic::FheUintBlocksPrepared;

// pub struct Interpreter {
//     params: Parameters<BackendImpl>,
//     r1_ram: Ram<BackendImpl>,
//     r2_ram: Ram<BackendImpl>,
//     rd_ram: Ram<BackendImpl>,
//     imm_ram: Ram<BackendImpl>,
// }

// impl Interpreter {
//     pub fn new() -> Self {
//         Self {
//             params: Parameters::new(),
//             r1_ram: Ram::new(),
//             r2_ram: Ram::new(),
//             rd_ram: Ram::new(),
//             imm_ram: Ram::new(),
//         }
//     }
// }

// //use crate::gadget::Gadget;
// use crate::address::Address;
// use crate::circuit_bootstrapping::{bootstrap_address_tmp_bytes, CircuitBootstrapper};
// use crate::decompose::{Base1D, Base2D, Decomposer, Precomp};
// use crate::instructions::memory::{
//     extract_from_byte_offset, load, prepare_address_floor_byte_offset, store,
// };
use crate::{instructions::{
    decompose, reconstruct, InstructionsParser, LOAD_OPS_LIST, PC_OPS_LIST, RD_OPS_LIST,
}, Instruction};
// use crate::memory::{read_tmp_bytes, Memory};
// use crate::parameters::{
//     Parameters, DECOMP_U32, LOGBASE2K, LOGK, RLWE_COLS, VMPPMAT_COLS, VMPPMAT_ROWS,
// };
// use crate::trace::trace_inplace_inv;
// use base2k::{alloc_aligned, Encoding, Module, VecZnx, VecZnxDft, VecZnxDftOps, VecZnxOps};
use itertools::izip;

pub struct Interpreter {
    pub params: CryptographicParameters<BackendImpl>,
    
    pub source_xa: Source,
    pub source_xe: Source,

    pub imm_rom_keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl>,
    pub rs1_rom_keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl>,
    pub rs2_rom_keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl>,
    pub rd_rom_keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl>,

    pub imm_rom: Ram<BackendImpl>,
    pub rs1_rom: Ram<BackendImpl>,
    pub rs2_rom: Ram<BackendImpl>,
    pub rd_rom: Ram<BackendImpl>,

    pub registers: Ram<BackendImpl>,
    pub ram: Ram<BackendImpl>,
    pub ram_offset: u32,

    pub program_counter: FheUintBlocksPrepared<Vec<u8>, u32, BackendImpl>,

    // pub imm: Memory,
    // pub instructions: Memory,
    // pub registers: Memory,
    // pub ram: Memory,
    // pub ret: bool,
    // pub ram_offset: u32,
    // pub pc_recomposition: Memory,
    // pub circuit_btp: CircuitBootstrapper,
    // pub decomposer: Decomposer,
    // pub precomp_decompose_instructions: Precomp,
    // pub precomp_decompose_arithmetic: Precomp,
    // pub tmp_bytes: Vec<u8>,
    // pub addr_pc_precomp: Precomp,
    // pub addr_pc: Address,
    // pub addr_instr: Address,
    // pub addr_ram_precomp: Precomp,
    // pub addr_ram: Address,
    // pub addr_u2_precomp: Precomp,
    // pub addr_u4: Address,
    // pub addr_u4_precomp: Precomp,
    // pub addr_u5: Address,
    // pub addr_u5_precomp: Precomp,
    // pub addr_u6: Address,
    // pub addr_u6_precomp: Precomp,
    // pub addr_ram_state: bool,
}

impl Interpreter {
    pub fn new(sk_glwe: &GLWESecret<Vec<u8>>, sk_glwe_prepared: &GLWESecretPrepared<Vec<u8>, BackendImpl>) -> Self {
        pub const DECOMP_N: [u8; 6] = [2, 2, 2, 2, 2, 2];
        pub const ROM_MAX_ADDR: usize = 1 << 14;
        pub const RAM_MAX_ADDR: usize = 1 << 14;

        let imm_rom = Ram::new_from_ram_params(32, DECOMP_N.to_vec(), ROM_MAX_ADDR);
        let rs1_rom = Ram::new_from_ram_params(5, DECOMP_N.to_vec(), ROM_MAX_ADDR);
        let rs2_rom = Ram::new_from_ram_params(5, DECOMP_N.to_vec(), ROM_MAX_ADDR);
        let rd_rom = Ram::new_from_ram_params(5, DECOMP_N.to_vec(), ROM_MAX_ADDR);
        
        let registers = Ram::new_from_ram_params(32, DECOMP_N.to_vec(), 32);
        let ram = Ram::new_from_ram_params(8, DECOMP_N.to_vec(), RAM_MAX_ADDR);

        // Generate random seeds for encryption
        let seed_xa = [5u8; 32];
        let seed_xe = [6u8; 32];

        let mut source_xa = Source::new(seed_xa);
        let mut source_xe = Source::new(seed_xe);

        let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 22);
        
        let params = Parameters::<BackendImpl>::new();
        let keys: EvaluationKeys<Vec<u8>> =
            EvaluationKeys::encrypt_sk(&params, sk_glwe, &mut source_xa, &mut source_xe);
        
        let mut imm_rom_keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl> =
            EvaluationKeysPrepared::alloc(&params);
        imm_rom_keys_prepared.prepare(params.module(), &keys, scratch.borrow());

        let mut rs1_rom_keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl> =
            EvaluationKeysPrepared::alloc(&params);
        rs1_rom_keys_prepared.prepare(params.module(), &keys, scratch.borrow());

        let mut rs2_rom_keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl> =
            EvaluationKeysPrepared::alloc(&params);
        rs2_rom_keys_prepared.prepare(params.module(), &keys, scratch.borrow());

        let mut rd_rom_keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl> =
            EvaluationKeysPrepared::alloc(&params);
        rd_rom_keys_prepared.prepare(params.module(), &keys, scratch.borrow());
    
        Self {
            params: CryptographicParameters::<BackendImpl>::new(),

            source_xa: Source::new(seed_xa),
            source_xe: Source::new(seed_xe),

            imm_rom_keys_prepared: imm_rom_keys_prepared,
            rs1_rom_keys_prepared: rs1_rom_keys_prepared,
            rs2_rom_keys_prepared: rs2_rom_keys_prepared,
            rd_rom_keys_prepared: rd_rom_keys_prepared,

            imm_rom: imm_rom,
            rs1_rom: rs1_rom,
            rs2_rom: rs2_rom,
            rd_rom: rd_rom,
            registers: registers,
            ram: ram,
            
            ram_offset: 0,
            program_counter: FheUintBlocksPrepared::alloc(params.module(), &params.ggsw_infos()),
        }
    }

    pub fn init_pc(&mut self, initial_pc: u32, sk_glwe_prep: &GLWESecretPrepared<Vec<u8>, BackendImpl>) {
        let pc_value = initial_pc;
        // Generate random seeds for encryption
        let seed_xa = [5u8; 32];
        let seed_xe = [6u8; 32];

        let mut source_xa = Source::new(seed_xa);
        let mut source_xe = Source::new(seed_xe);

        // TODO: set scratch correctly.
        let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 22);

        self.program_counter.encrypt_sk(
            self.params.module(),
            pc_value,
            sk_glwe_prep,
            &mut source_xa,
            &mut source_xe,
            scratch.borrow(),
        );    
    }

    pub fn init_instructions(&mut self, sk_glwe: &GLWESecret<Vec<u8>>, instructions: InstructionsParser) {
        
        let max_addr_imm = self.imm_rom.params.max_addr();
        let max_addr_rs1 = self.rs1_rom.params.max_addr();
        let max_addr_rs2 = self.rs2_rom.params.max_addr();
        let max_addr_rd = self.rd_rom.params.max_addr();

        let rs1_word_size = self.rs1_rom.params.word_size();
        let rs2_word_size = self.rs2_rom.params.word_size();
        let rd_word_size = self.rd_rom.params.word_size();
        let imm_word_size = self.imm_rom.params.word_size();
        
        let mut data_ram_rs1 = vec![0u8; max_addr_rs1 * rs1_word_size];
        let mut data_ram_rs2 = vec![0u8; max_addr_rs2 * rs2_word_size];
        let mut data_ram_rd = vec![0u8; max_addr_rd * rd_word_size];
        let mut data_ram_imm = vec![0u8; max_addr_imm * imm_word_size];
        
        for i in 0..instructions.instructions.len() {
            
            let rs1 = instructions.get_raw(i).get_rs1_or_zero();
            let rs2 = instructions.get_raw(i).get_rs2_or_zero();
            let rd = instructions.get_raw(i).get_rd_or_zero();
            let imm = instructions.get_raw(i).get_immediate();
            println!("rs1: {:?}, rs2: {:?}, rd: {:?}, imm: {:?}", rs1, rs2, rd, imm);

            data_ram_rs1[i * rs1_word_size..(i + 1) * rs1_word_size]
                .iter_mut()
                .enumerate()
                .for_each(|(idx, v)| *v = ((rs1 >> idx) & 1) as u8);

            data_ram_rs2[i * rs2_word_size..(i + 1) * rs2_word_size]
                .iter_mut()
                .enumerate()
                .for_each(|(idx, v)| *v = ((rs2 >> idx) & 1) as u8);
            
            data_ram_rd[i * rd_word_size..(i + 1) * rd_word_size]
                .iter_mut()
                .enumerate()
                .for_each(|(idx, v)| *v = ((rd >> idx) & 1) as u8);

            data_ram_imm[i * imm_word_size..(i + 1) * imm_word_size]
                .iter_mut()
                .enumerate()
                .for_each(|(idx, v)| *v = ((imm >> idx) & 1) as u8);
        }

        self.rs1_rom.encrypt_sk(&data_ram_rs1, &sk_glwe, &mut self.source_xa, &mut self.source_xe);
        self.rs2_rom.encrypt_sk(&data_ram_rs2, &sk_glwe, &mut self.source_xa, &mut self.source_xe);
        self.rd_rom.encrypt_sk(&data_ram_rd, &sk_glwe, &mut self.source_xa, &mut self.source_xe);
        self.imm_rom.encrypt_sk(&data_ram_imm, &sk_glwe, &mut self.source_xa, &mut self.source_xe);

    }

    pub fn init_ram_offset(&mut self, ram_offset: u32) {
        self.ram_offset = ram_offset;
    }

    pub fn init_registers(&mut self, sk_glwe: &GLWESecret<Vec<u8>>, registers: &Vec<u32>) {

        let max_addr = self.registers.params.max_addr();
        let default_word_size = self.registers.params.word_size(); // TODO: hardcoded based on 8-bit plaintext precision

        let mut registers_data = vec![0u8; max_addr * default_word_size];
        for i in 0..registers.len() {
            for j in 0..default_word_size {
                registers_data[j + i * default_word_size] = ((registers[i] >> j) & 1) as u8;
            }
        }

        self.registers.encrypt_sk(&registers_data, &sk_glwe, &mut self.source_xa, &mut self.source_xe);
    }

    pub fn init_ram(&mut self, sk_glwe: &GLWESecret<Vec<u8>>, ram: &Vec<u8>) {
        assert_eq!(ram.len(), self.ram.params.max_addr());
        let max_addr = self.ram.params.max_addr();
        let default_word_size = self.ram.params.word_size(); // TODO: hardcoded based on 8-bit plaintext precision

        let mut ram_data = vec![0u8; max_addr * default_word_size];
        for i in 0..ram.len() {
            for j in 0..default_word_size {
                ram_data[j + i * default_word_size] = ((ram[i] >> j) & 1) as u8;
            }
        }

        self.ram.encrypt_sk(&ram_data, &sk_glwe, &mut self.source_xa, &mut self.source_xe);
    }

    pub fn cycle(&mut self, sk_glwe: &GLWESecret<Vec<u8>>, sk_glwe_prep: &GLWESecretPrepared<Vec<u8>, BackendImpl>) {

        let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 22);

        let params = Parameters::<BackendImpl>::new();
        let mut address_rom: Address<Vec<u8>> = Address::alloc_from_params(&params);
        address_rom.set_from_fheuint(self.params.module(), &self.program_counter, scratch.borrow());

        let imm_glwe = self.imm_rom.read(&address_rom, &self.imm_rom_keys_prepared);
        let rs1_glwe = self.rs1_rom.read(&address_rom, &self.rs1_rom_keys_prepared);
        let rs2_glwe = self.rs2_rom.read(&address_rom, &self.rs2_rom_keys_prepared);
        let rd_glwe = self.rd_rom.read(&address_rom, &self.rd_rom_keys_prepared);

        // println!(
        //     "pc: {}",
        //     self.addr_pc.debug_as_u32(params.module_lwe()) << 2
        // );
        // println!("REGS: {:?}", &self.registers.debug_as_u32());
        // println!("MEM: {:?}", &self.ram.debug_as_u32()[..32]);

        // let module_lwe: &Module = params.module_lwe();
        // let module_pbs: &Module = params.module_pbs();

        // // 0) Fetches instructions selectors
        // //let now: Instant = Instant::now();
        // let (rs2_u5, rs1_u5, rd_u5, rd_w_u6, mem_w_u5, pc_w_u5) =
        //     self.get_instruction_selectors();
        // println!(
        //    "get_instruction_selectors: {} ms",
        //    now.elapsed().as_millis()
        // );

        // //println!("rd_w_u6: {}", rd_w_u6);
        // //println!("mem_w_u5: {}", mem_w_u5);
        // //println!("pc_w_u5: {}", pc_w_u5);
        // //println!("rs1_u5: {}", rs1_u5);
        // //println!("rs2_u5: {}", rs2_u5);
        // //println!("rd_u5: {}", rd_u5);

        // // 1) Retrieve 8xLWE(u4) inputs (imm, rs2, rs1, pc)
        // //let now: Instant = Instant::now();
        // let (imm_lwe, rs2_lwe, rs1_lwe, pc_lwe) =
        //     self.get_lwe_inputs(module_pbs, module_lwe, rs2_u5, rs1_u5);
        // //println!(
        // //    "get_lwe_inputs           : {} ms",
        // //    now.elapsed().as_millis()
        // //);

        // //println!("rs2_lwe: {}", reconstruct(&rs2_lwe));
        // //println!("rs1_lwe: {}", reconstruct(&rs1_lwe));
        // //println!("imm_lwe: {}", reconstruct(&imm_lwe));

        // // 2) Prepares ram address read/write (x_rs1 + sext(imm) - offset) where offset = (x_rs1 + sext(imm))%4
        // //let now: Instant = Instant::now();
        // let offset: u8 = self.prepare_ram_address_floor_byte_offset(
        //     module_pbs,
        //     module_lwe,
        //     &imm_lwe,
        //     &rs1_lwe,
        //     self.ram_offset,
        //     self.ram.max_size as u32,
        // );
        // //println!(
        // //    "prepare_ram_address   : {} ms",
        // //    now.elapsed().as_millis()
        // //);

        // // 3)  loads value from ram
        // //let now: Instant = Instant::now();
        // let loaded: [u8; 8] = self.read_ram(module_lwe);
        // //println!(
        // //    "read_ram              : {} ms",
        // //    now.elapsed().as_millis()
        // //);

        // //println!("offset: {}", offset);
        // //println!("ram_address: {}", self.addr_ram.debug_as_u32(module_lwe));
        // //println!("loaded: {:08x}", reconstruct(&loaded));

        // // Selects [4, 2, 1] bytes from loaded value
        // // according to offset.
        // let loaded_offset: [u8; 8] = extract_from_byte_offset(&loaded, offset);

        // // 4) Retrieves RD value from OPS(imm, rs1, rs2, pc, loaded)[rd_w_u6]
        // //let now: Instant = Instant::now();
        // let rd_lwe: [u8; 8] = self.evaluate_ops(
        //     module_pbs,
        //     module_lwe,
        //     &imm_lwe,
        //     &rs1_lwe,
        //     &rs2_lwe,
        //     &pc_lwe,
        //     &loaded_offset,
        //     rd_w_u6,
        // );
        // //println!(
        // //    "evaluate_ops             : {} ms",
        // //    now.elapsed().as_millis()
        // //);

        // // 5) Updates ram from {RD|LOADED}[mem_w_u5]
        // //let now: Instant = Instant::now();
        // self.store_ram(module_pbs, module_lwe, &rs2_lwe, &loaded, offset, mem_w_u5);
        // //println!(
        // //    "store_ram             : {} ms",
        // //    now.elapsed().as_millis()
        // //);

        // // 6) Updates registers from RD
        // //let now: Instant = Instant::now();
        // self.store_registers(module_pbs, module_lwe, &rd_lwe, rd_u5);
        // //println!(
        // //    "store_registers          : {} ms",
        // //    now.elapsed().as_millis()
        // //);

        // // 7) Update PC from OPS(imm, rs1, rs2, pc)[pc_w_u5]
        // //let now: Instant = Instant::now();
        // self.update_pc(
        //     module_pbs, module_lwe, &imm_lwe, &rs1_lwe, &rs2_lwe, &pc_lwe, pc_w_u5,
        // );
        // //println!(
        // //    "update_pc                : {} ms",
        // //    now.elapsed().as_millis()
        // //);

        // // Reinitialize checks
        // self.addr_ram_state = false;

        // //println!("pc_out: {}", self.addr_pc.debug_as_u32(params.module_lwe()));
        // //println!();
    }

    // fn evaluate_ops(
    //     &mut self,
    //     module_pbs: &Module,
    //     module_lwe: &Module,
    //     imm_lwe: &[u8; 8],
    //     rs1_lwe: &[u8; 8],
    //     rs2_lwe: &[u8; 8],
    //     pc_lwe: &[u8; 8],
    //     loaded: &[u8; 8],
    //     rd_w_u6: u8,
    // ) -> [u8; 8] {
    //     let mut vec_znx: base2k::VecZnx = module_lwe.new_vec_znx(RLWE_COLS);

    //     // Evaluates all arithmetic operations
    //     RD_OPS_LIST.iter().for_each(|op| {
    //         let (idx, out) = op.apply(imm_lwe, rs1_lwe, rs2_lwe, pc_lwe);
    //         vec_znx.encode_coeff_i64(LOGBASE2K, LOGK, idx, reconstruct(&out) as i64, 32);
    //     });

    //     // Selects correct loading mode
    //     LOAD_OPS_LIST.iter().for_each(|op| {
    //         let (idx, out) = op.apply(loaded);
    //         vec_znx.encode_coeff_i64(LOGBASE2K, LOGK, idx, reconstruct(&out) as i64, 32);
    //     });

    //     decompose(self.select_op::<6>(module_pbs, module_lwe, rd_w_u6 as u32, &mut vec_znx))
    // }

    // fn read_ram(&mut self, module_lwe: &Module) -> [u8; 8] {
    //     assert_eq!(
    //         self.addr_ram_state, true,
    //         "trying to read ram but ram address hasn't been prepared"
    //     );
    //     load(
    //         module_lwe,
    //         &mut self.ram,
    //         &mut self.addr_ram,
    //         &mut self.tmp_bytes,
    //     )
    // }

    // fn store_ram(
    //     &mut self,
    //     module_pbs: &Module,
    //     module_lwe: &Module,
    //     rs2_lwe: &[u8; 8],
    //     loaded: &[u8; 8],
    //     offset: u8,
    //     mem_w_u5: u8,
    // ) {
    //     assert_eq!(
    //         self.addr_ram_state, true,
    //         "trying to store in ram but addr_ram_state is false"
    //     );

    //     // Creates a list with all possible values to store.
    //     // rs2 = [a, b, c, d]
    //     // loaded = [e, f, g, h]

    //     // offset = 0
    //     // NONE: [e, f, g, h]
    //     // SB:   [a, f, g, h]
    //     // SH:   [a, b, g, h]
    //     // SW:   [a, b, c, d]
    //     //
    //     // offset = 1
    //     // NONE: [e, f, g, h]
    //     // SB:   [e, a, g, h]
    //     // SH:   [e, a, b, h]
    //     // SW:   [ INVALID  ]
    //     //
    //     // offset = 2
    //     // NONE: [e, f, g, h]
    //     // SB:   [e, f, g, a]
    //     // SH:   [e, f, a, b]
    //     // SW:   [ INVALID  ]
    //     //
    //     // offset = 3
    //     // NONE: [e, f, g, h]
    //     // SB:   [e, f, g, a]
    //     // SH:   [ INVALID  ]
    //     // SW:   [ INVALID  ]

    //     let list: [[u8; 8]; 16] = [
    //         *loaded,
    //         [
    //             rs2_lwe[0], rs2_lwe[1], loaded[2], loaded[3], loaded[4], loaded[5], loaded[6],
    //             loaded[7],
    //         ],
    //         [
    //             rs2_lwe[0], rs2_lwe[1], rs2_lwe[2], rs2_lwe[3], loaded[4], loaded[5], loaded[6],
    //             loaded[7],
    //         ],
    //         *rs2_lwe,
    //         *loaded,
    //         [
    //             loaded[0], loaded[1], rs2_lwe[0], rs2_lwe[1], loaded[4], loaded[5], loaded[6],
    //             loaded[7],
    //         ],
    //         [
    //             loaded[0], loaded[1], rs2_lwe[0], rs2_lwe[1], rs2_lwe[2], rs2_lwe[3], loaded[6],
    //             loaded[7],
    //         ],
    //         [0, 0, 0, 0, 0, 0, 0, 0],
    //         *loaded,
    //         [
    //             loaded[0], loaded[1], loaded[2], loaded[3], rs2_lwe[0], rs2_lwe[1], loaded[6],
    //             loaded[7],
    //         ],
    //         [
    //             loaded[0], loaded[1], loaded[2], loaded[3], rs2_lwe[0], rs2_lwe[1], rs2_lwe[2],
    //             rs2_lwe[3],
    //         ],
    //         [0, 0, 0, 0, 0, 0, 0, 0],
    //         *loaded,
    //         [
    //             loaded[0], loaded[1], loaded[2], loaded[3], loaded[4], loaded[5], rs2_lwe[0],
    //             rs2_lwe[1],
    //         ],
    //         [0, 0, 0, 0, 0, 0, 0, 0],
    //         [0, 0, 0, 0, 0, 0, 0, 0],
    //     ];

    //     // Creates a vector of VecZnx storing list.
    //     let mut vec_znx: base2k::VecZnx = module_lwe.new_vec_znx(RLWE_COLS);
    //     list.iter().enumerate().for_each(|(i, x)| {
    //         vec_znx.encode_coeff_i64(LOGBASE2K, LOGK, i, reconstruct(x) as i64, 32);
    //     });

    //     // Sample extract
    //     let value: [u8; 8] = decompose(self.select_op::<4>(
    //         module_pbs,
    //         module_lwe,
    //         ((offset as u32) << 2) + mem_w_u5 as u32,
    //         &mut vec_znx,
    //     ));

    //     store(
    //         module_lwe,
    //         &value,
    //         &mut self.ram,
    //         &mut self.addr_ram,
    //         &mut self.tmp_bytes,
    //     );
    // }

    // fn get_lwe_inputs(
    //     &mut self,
    //     module_pbs: &Module,
    //     module_lwe: &Module,
    //     rs2_u5: u8,
    //     rs1_u5: u8,
    // ) -> ([u8; 8], [u8; 8], [u8; 8], [u8; 8]) {
    //     let imm_lwe: [u8; 8] = self.get_imm_lwe(module_pbs, module_lwe);
    //     let rs2_lwe: [u8; 8] = self.get_input_from_register_lwe(module_pbs, module_lwe, rs2_u5);
    //     let rs1_lwe: [u8; 8] = self.get_input_from_register_lwe(module_pbs, module_lwe, rs1_u5);
    //     let pc_lwe: [u8; 8] = self.get_pc_lwe(module_pbs, module_lwe);
    //     (imm_lwe, rs2_lwe, rs1_lwe, pc_lwe)
    // }

    // fn update_pc(
    //     &mut self,
    //     module_pbs: &Module,
    //     module_lwe: &Module,
    //     imm_lwe: &[u8; 8],
    //     rs1_lwe: &[u8; 8],
    //     rs2_lwe: &[u8; 8],
    //     pc_lwe: &[u8; 8],
    //     pc_w_u5: u8,
    // ) {
    //     let mut vec_znx: base2k::VecZnx = module_lwe.new_vec_znx(RLWE_COLS);

    //     PC_OPS_LIST.iter().for_each(|op| {
    //         let (idx, out) = op.apply(imm_lwe, rs1_lwe, rs2_lwe, pc_lwe);
    //         vec_znx.encode_coeff_i64(LOGBASE2K, LOGK, idx, reconstruct(&out) as i64, 32);
    //     });

    //     let pc_u32: u32 = self.select_op::<5>(module_pbs, module_lwe, pc_w_u5 as u32, &mut vec_znx);

    //     self.circuit_btp.bootstrap_to_address(
    //         module_pbs,
    //         module_lwe,
    //         &mut self.decomposer,
    //         &self.addr_pc_precomp,
    //         pc_u32 >> 2, // TODO: HE DIV by 4
    //         &mut self.addr_pc,
    //         &mut self.tmp_bytes,
    //     );
    // }

    // fn store_registers(
    //     &mut self,
    //     module_pbs: &Module,
    //     module_lwe: &Module,
    //     rd_lwe: &[u8; 8],
    //     rd_u5: u8,
    // ) {
    //     self.circuit_btp.bootstrap_to_address(
    //         module_pbs,
    //         module_lwe,
    //         &mut self.decomposer,
    //         &self.addr_u5_precomp,
    //         rd_u5 as u32,
    //         &mut self.addr_u5,
    //         &mut self.tmp_bytes,
    //     );

    //     self.registers
    //         .read_prepare_write(module_lwe, &self.addr_u5, &mut self.tmp_bytes);
    //     store(
    //         module_lwe,
    //         rd_lwe,
    //         &mut self.registers,
    //         &mut self.addr_u5,
    //         &mut self.tmp_bytes,
    //     );

    //     trace_inplace_inv(
    //         module_lwe,
    //         LOGBASE2K,
    //         0,
    //         module_lwe.log_n(),
    //         &mut self.registers.data[0],
    //         &mut self.tmp_bytes,
    //     );
    // }

    // fn prepare_ram_address_floor_byte_offset(
    //     &mut self,
    //     module_pbs: &Module,
    //     module_lwe: &Module,
    //     imm_lwe: &[u8; 8],
    //     rs1_lwe: &[u8; 8],
    //     ram_offset: u32,
    //     max_size: u32,
    // ) -> u8 {
    //     assert_eq!(
    //         self.addr_ram_state, false,
    //         "trying to prepare address rs1 + imm - ram_offset but state indicates it has already been done"
    //     );
    //     let offset: u8 = prepare_address_floor_byte_offset(
    //         module_pbs,
    //         module_lwe,
    //         imm_lwe,
    //         rs1_lwe,
    //         ram_offset,
    //         max_size,
    //         &self.circuit_btp,
    //         &mut self.decomposer,
    //         &self.addr_u2_precomp,
    //         &self.addr_ram_precomp,
    //         &mut self.addr_ram,
    //         &mut self.tmp_bytes,
    //     );
    //     self.addr_ram_state = true;
    //     offset
    // }

    // // Recompose PC ADDRESS to PC LWE and shifts by 2.
    // fn get_pc_lwe(&mut self, module_pbs: &Module, module_lwe: &Module) -> [u8; 8] {
    //     let pc_u32: u32 =
    //         self.pc_recomposition
    //             .read(module_lwe, &self.addr_pc, &mut self.tmp_bytes);
    //     decompose_1xu32_to_8xu4(
    //         module_pbs,
    //         &mut self.decomposer,
    //         &self.precomp_decompose_arithmetic,
    //         pc_u32 << 2,
    //     )
    // }

    // fn get_input_from_register_lwe(
    //     &mut self,
    //     module_pbs: &Module,
    //     module_lwe: &Module,
    //     address: u8,
    // ) -> [u8; 8] {
    //     let (tmp_bytes_read, tmp_bytes_bootstrap_address) = self.tmp_bytes.split_at_mut(
    //         read_tmp_bytes(module_lwe, RLWE_COLS, VMPPMAT_ROWS, VMPPMAT_COLS),
    //     );
    //     let tmp_address = &mut self.addr_u5;
    //     self.circuit_btp.bootstrap_to_address(
    //         module_pbs,
    //         module_lwe,
    //         &mut self.decomposer,
    //         &self.addr_u5_precomp,
    //         address as u32,
    //         tmp_address,
    //         tmp_bytes_bootstrap_address,
    //     );
    //     let value: u32 = self.registers.read(module_lwe, tmp_address, tmp_bytes_read);
    //     decompose_1xu32_to_8xu4(
    //         module_pbs,
    //         &mut self.decomposer,
    //         &self.precomp_decompose_arithmetic,
    //         value,
    //     )
    // }

    fn get_instruction_selectors(
        &mut self,
        // module_pbs: &Module,
        // module_lwe: &Module,
    ) -> (u8, u8, u8, u8, u8, u8) {
        // let (tmp_bytes_read, _) = self.tmp_bytes.split_at_mut(read_tmp_bytes(
        //     module_lwe,
        //     RLWE_COLS,
        //     VMPPMAT_ROWS,
        //     VMPPMAT_COLS,
        // ));
        // let instructions: u32 = self
        //     .instructions
        //     .read(module_lwe, &self.addr_pc, tmp_bytes_read);
        // let selector: Vec<u8> = self.decomposer.decompose(
        //     module_pbs,
        //     &self.precomp_decompose_instructions,
        //     instructions as u32,
        // );
        // (
        //     selector[5], // rs2_u5
        //     selector[4], // rs1_u5
        //     selector[3], // rd_u5
        //     selector[2], // rd_w_u6
        //     selector[1], // mem_w_u5
        //     selector[0], // pc_w_u5
        // )
        (0, 0, 0, 0, 0, 0)
    }

    // fn get_imm_lwe(&mut self, module_pbs: &Module, module_lwe: &Module) -> [u8; 8] {
    //     let imm_u32: u32 = self
    //         .imm
    //         .read(module_lwe, &self.addr_pc, &mut self.tmp_bytes);
    //     decompose_1xu32_to_8xu4(
    //         module_pbs,
    //         &mut self.decomposer,
    //         &self.precomp_decompose_arithmetic,
    //         imm_u32,
    //     )
    // }

    // fn select_op<const SIZE: u8>(
    //     &mut self,
    //     module_pbs: &Module,
    //     module_lwe: &Module,
    //     value: u32,
    //     vec_znx: &mut VecZnx,
    // ) -> u32 {
    //     let precomp: &Precomp;
    //     let addr: &mut Address;

    //     match SIZE {
    //         4 => {
    //             precomp = &self.addr_u4_precomp;
    //             addr = &mut self.addr_u4
    //         }
    //         5 => {
    //             precomp = &self.addr_u5_precomp;
    //             addr = &mut self.addr_u5
    //         }
    //         6 => {
    //             precomp = &self.addr_u6_precomp;
    //             addr = &mut self.addr_u6
    //         }
    //         _ => panic!("invalid operation selector size"),
    //     }

    //     #[cfg(debug_assertions)]
    //     {
    //         match SIZE {
    //             4 => assert!(value < 1 << 4, "4 bits selector out of range"),
    //             5 => assert!(value < 1 << 5, "5 bits selector out of range"),
    //             6 => assert!(value < 1 << 6, "6 bits selector out of range"),
    //             _ => panic!("invalid operation selector size"),
    //         }
    //     }

    //     // Bootstraps u4 address to X^{i}
    //     self.circuit_btp.bootstrap_to_address(
    //         module_pbs,
    //         module_lwe,
    //         &mut self.decomposer,
    //         precomp,
    //         value,
    //         addr,
    //         &mut self.tmp_bytes,
    //     );

    //     let (vec_znx_dft_tmp_bytes, tmp_bytes) = self
    //         .tmp_bytes
    //         .split_at_mut(module_lwe.bytes_of_vec_znx_dft(VMPPMAT_COLS));

    //     let mut tmp_b_dft: VecZnxDft =
    //         VecZnxDft::from_bytes_borrow(module_lwe, VMPPMAT_COLS, vec_znx_dft_tmp_bytes);

    //     // Selects according to offset
    //     addr.at_lsh(0)
    //         .product_inplace(module_lwe, LOGBASE2K, vec_znx, &mut tmp_b_dft, tmp_bytes);

    //     vec_znx.decode_coeff_i64(LOGBASE2K, LOGK, 0) as u32
    // }
}

// pub fn next_tmp_bytes(module_pbs: &Module, module_lwe: &Module) -> usize {
//     read_tmp_bytes(module_lwe, RLWE_COLS, VMPPMAT_ROWS, VMPPMAT_COLS)
//         + bootstrap_address_tmp_bytes(module_pbs, module_lwe, VMPPMAT_COLS)
// }

// pub fn get_lwe_tmp_bytes(module_pbs: &Module, module_lwe: &Module) -> usize {
//     next_tmp_bytes(module_pbs, module_lwe)
// }

// pub fn decompose_1xu32_to_8xu4(
//     module_pbs: &Module,
//     decomposer: &mut Decomposer,
//     precomp: &Precomp,
//     value: u32,
// ) -> [u8; 8] {
//     let value_u8: Vec<u8> = decomposer.decompose(module_pbs, precomp, value);
//     [
//         value_u8[0],
//         value_u8[1],
//         value_u8[2],
//         value_u8[3],
//         value_u8[4],
//         value_u8[5],
//         value_u8[6],
//         value_u8[7],
//     ]
// }
