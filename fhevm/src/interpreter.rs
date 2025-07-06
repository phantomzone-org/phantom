//use std::time::Instant;

//use crate::gadget::Gadget;
use crate::address::Address;
use crate::circuit_bootstrapping::{bootstrap_address_tmp_bytes, CircuitBootstrapper};
use crate::decompose::{Base1D, Base2D, Decomposer, Precomp};
use crate::instructions::memory::{
    extract_from_byte_offset, load, prepare_address_floor_byte_offset, store,
};
use crate::instructions::{
    decompose, reconstruct, InstructionsParser, LOAD_OPS_LIST, PC_OPS_LIST, RD_OPS_LIST,
};
use crate::memory::{read_tmp_bytes, Memory};
use crate::parameters::{
    Parameters, DECOMP_U32, LOGBASE2K, LOGK, RLWE_COLS, VMPPMAT_COLS, VMPPMAT_ROWS,
};
use crate::trace::trace_inplace_inv;
use base2k::{alloc_aligned, Encoding, Module, VecZnx, VecZnxDft, VecZnxDftOps, VecZnxOps};
use itertools::izip;

// retrieve registers
// retrieve immediates
// retrieve (op id)

pub struct Interpreter {
    pub imm: Memory,
    pub instructions: Memory,
    pub registers: Memory,
    pub ram: Memory,
}

impl Interpreter {
    pub fn new(params: &Parameters) -> Self {
        let module_lwe: &Module = params.module_lwe();
        let module_pbs: &Module = params.module_pbs();
        let log_k: usize = LOGBASE2K * (VMPPMAT_COLS - 1) - 5;
        let cols: usize = (log_k + LOGBASE2K - 1) / LOGBASE2K;
        let mut pc_recomposition: Memory =
            Memory::new(module_lwe, LOGBASE2K, cols, params.rom_size);
        let mut data: Vec<i64> = vec![i64::default(); params.rom_size];
        data.iter_mut().enumerate().for_each(|(i, x)| *x = i as i64);
        pc_recomposition.set(&data, log_k);
        Self {
            imm: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, params.rom_size),
            instructions: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, params.rom_size),
            registers: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, params.u5_max()),
            ram: Memory::new(module_lwe, LOGBASE2K, RLWE_COLS, params.ram_size),
        }
    }

    pub fn init_pc(&mut self, params: &Parameters) {
        self.addr_pc.set(params.module_lwe(), 0);
    }

    pub fn init_instructions(&mut self, instructions: InstructionsParser) {
        assert_eq!(instructions.instructions.len(), self.instructions.max_size);
        self.imm.set(&instructions.imm, LOGK);
        self.instructions.set(&instructions.instructions, LOGK);
    }

    pub fn init_ram_offset(&mut self, ram_offset: u32) {
        self.ram_offset = ram_offset;
    }

    pub fn init_registers(&mut self, registers: &Vec<u32>) {
        let mut registers_i64: Vec<i64> = vec![0i64; registers.len()];
        izip!(registers_i64.iter_mut(), registers.iter()).for_each(|(a, b)| *a = *b as i64);
        self.registers.set(&registers_i64, LOGK);
    }

    pub fn init_ram(&mut self, ram: &Vec<u32>) {
        assert_eq!(ram.len(), self.ram.max_size);
        let mut ram_i64: Vec<i64> = vec![i64::default(); ram.len()];
        izip!(ram_i64.iter_mut(), ram.iter()).for_each(|(a, b)| *a = *b as i64);
        self.ram.set(&ram_i64[..], LOGK);
    }

    pub fn cycle(&mut self, params: &Parameters) {
        println!(
            "pc: {}",
            self.addr_pc.debug_as_u32(params.module_lwe()) << 2
        );
        println!("REGS: {:?}", &self.registers.debug_as_u32());
        println!("MEM: {:?}", &self.ram.debug_as_u32()[..32]);

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

        // 2) Prepares ram address read/write (x_rs1 + sext(imm) - offset) where offset = (x_rs1 + sext(imm))%4
        //let now: Instant = Instant::now();
        let offset: u8 = self.prepare_ram_address_floor_byte_offset(
            module_pbs,
            module_lwe,
            &imm_lwe,
            &rs1_lwe,
            self.ram_offset,
            self.ram.max_size as u32,
        );
        //println!(
        //    "prepare_ram_address   : {} ms",
        //    now.elapsed().as_millis()
        //);

        // 3)  loads value from ram
        //let now: Instant = Instant::now();
        let loaded: [u8; 8] = self.read_ram(module_lwe);
        //println!(
        //    "read_ram              : {} ms",
        //    now.elapsed().as_millis()
        //);

        //println!("offset: {}", offset);
        //println!("ram_address: {}", self.addr_ram.debug_as_u32(module_lwe));
        //println!("loaded: {:08x}", reconstruct(&loaded));

        // Selects [4, 2, 1] bytes from loaded value
        // according to offset.
        let loaded_offset: [u8; 8] = extract_from_byte_offset(&loaded, offset);

        // 4) Retrieves RD value from OPS(imm, rs1, rs2, pc, loaded)[rd_w_u6]
        //let now: Instant = Instant::now();
        let rd_lwe: [u8; 8] = self.evaluate_ops(
            module_pbs,
            module_lwe,
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

        // 5) Updates ram from {RD|LOADED}[mem_w_u5]
        //let now: Instant = Instant::now();
        self.store_ram(module_pbs, module_lwe, &rs2_lwe, &loaded, offset, mem_w_u5);
        //println!(
        //    "store_ram             : {} ms",
        //    now.elapsed().as_millis()
        //);

        // 6) Updates registers from RD
        //let now: Instant = Instant::now();
        self.store_registers(module_pbs, module_lwe, &rd_lwe, rd_u5);
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
        self.addr_ram_state = false;

        //println!("pc_out: {}", self.addr_pc.debug_as_u32(params.module_lwe()));
        //println!();
    }

    fn evaluate_ops(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm_lwe: &[u8; 8],
        rs1_lwe: &[u8; 8],
        rs2_lwe: &[u8; 8],
        pc_lwe: &[u8; 8],
        loaded: &[u8; 8],
        rd_w_u6: u8,
    ) -> [u8; 8] {
        let mut vec_znx: base2k::VecZnx = module_lwe.new_vec_znx(RLWE_COLS);

        // Evaluates all arithmetic operations
        RD_OPS_LIST.iter().for_each(|op| {
            let (idx, out) = op.apply(imm_lwe, rs1_lwe, rs2_lwe, pc_lwe);
            vec_znx.encode_coeff_i64(LOGBASE2K, LOGK, idx, reconstruct(&out) as i64, 32);
        });

        // Selects correct loading mode
        LOAD_OPS_LIST.iter().for_each(|op| {
            let (idx, out) = op.apply(loaded);
            vec_znx.encode_coeff_i64(LOGBASE2K, LOGK, idx, reconstruct(&out) as i64, 32);
        });

        decompose(self.select_op::<6>(module_pbs, module_lwe, rd_w_u6 as u32, &mut vec_znx))
    }

    fn read_ram(&mut self, module_lwe: &Module) -> [u8; 8] {
        assert_eq!(
            self.addr_ram_state, true,
            "trying to read ram but ram address hasn't been prepared"
        );
        load(
            module_lwe,
            &mut self.ram,
            &mut self.addr_ram,
            &mut self.tmp_bytes,
        )
    }

    fn store_ram(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
        rs2_lwe: &[u8; 8],
        loaded: &[u8; 8],
        offset: u8,
        mem_w_u5: u8,
    ) {
        assert_eq!(
            self.addr_ram_state, true,
            "trying to store in ram but addr_ram_state is false"
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

        // Sample extract
        let value: [u8; 8] = decompose(self.select_op::<4>(
            module_pbs,
            module_lwe,
            ((offset as u32) << 2) + mem_w_u5 as u32,
            &mut vec_znx,
        ));

        store(
            module_lwe,
            &value,
            &mut self.ram,
            &mut self.addr_ram,
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
        let mut vec_znx: base2k::VecZnx = module_lwe.new_vec_znx(RLWE_COLS);

        PC_OPS_LIST.iter().for_each(|op| {
            let (idx, out) = op.apply(imm_lwe, rs1_lwe, rs2_lwe, pc_lwe);
            vec_znx.encode_coeff_i64(LOGBASE2K, LOGK, idx, reconstruct(&out) as i64, 32);
        });

        let pc_u32: u32 = self.select_op::<5>(module_pbs, module_lwe, pc_w_u5 as u32, &mut vec_znx);

        self.circuit_btp.bootstrap_to_address(
            module_pbs,
            module_lwe,
            &mut self.decomposer,
            &self.addr_pc_precomp,
            pc_u32 >> 2, // TODO: HE DIV by 4
            &mut self.addr_pc,
            &mut self.tmp_bytes,
        );
    }

    fn store_registers(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
        rd_lwe: &[u8; 8],
        rd_u5: u8,
    ) {
        self.circuit_btp.bootstrap_to_address(
            module_pbs,
            module_lwe,
            &mut self.decomposer,
            &self.addr_u5_precomp,
            rd_u5 as u32,
            &mut self.addr_u5,
            &mut self.tmp_bytes,
        );

        self.registers
            .read_prepare_write(module_lwe, &self.addr_u5, &mut self.tmp_bytes);
        store(
            module_lwe,
            rd_lwe,
            &mut self.registers,
            &mut self.addr_u5,
            &mut self.tmp_bytes,
        );

        trace_inplace_inv(
            module_lwe,
            LOGBASE2K,
            0,
            module_lwe.log_n(),
            &mut self.registers.data[0],
            &mut self.tmp_bytes,
        );
    }

    fn prepare_ram_address_floor_byte_offset(
        &mut self,
        module_pbs: &Module,
        module_lwe: &Module,
        imm_lwe: &[u8; 8],
        rs1_lwe: &[u8; 8],
        ram_offset: u32,
        max_size: u32,
    ) -> u8 {
        assert_eq!(
            self.addr_ram_state, false,
            "trying to prepare address rs1 + imm - ram_offset but state indicates it has already been done"
        );
        let offset: u8 = prepare_address_floor_byte_offset(
            module_pbs,
            module_lwe,
            imm_lwe,
            rs1_lwe,
            ram_offset,
            max_size,
            &self.circuit_btp,
            &mut self.decomposer,
            &self.addr_u2_precomp,
            &self.addr_ram_precomp,
            &mut self.addr_ram,
            &mut self.tmp_bytes,
        );
        self.addr_ram_state = true;
        offset
    }

    // Recompose PC ADDRESS to PC LWE and shifts by 2.
    fn get_pc_lwe(&mut self, module_pbs: &Module, module_lwe: &Module) -> [u8; 8] {
        let pc_u32: u32 =
            self.pc_recomposition
                .read(module_lwe, &self.addr_pc, &mut self.tmp_bytes);
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
        let tmp_address = &mut self.addr_u5;
        self.circuit_btp.bootstrap_to_address(
            module_pbs,
            module_lwe,
            &mut self.decomposer,
            &self.addr_u5_precomp,
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
        let instructions: u32 = self
            .instructions
            .read(module_lwe, &self.addr_pc, tmp_bytes_read);
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
        let imm_u32: u32 = self
            .imm
            .read(module_lwe, &self.addr_pc, &mut self.tmp_bytes);
        decompose_1xu32_to_8xu4(
            module_pbs,
            &mut self.decomposer,
            &self.precomp_decompose_arithmetic,
            imm_u32,
        )
    }

    //     fn select_op<const SIZE: u8>(
    //         &mut self,
    //         module_pbs: &Module,
    //         module_lwe: &Module,
    //         value: u32,
    //         vec_znx: &mut VecZnx,
    //     ) -> u32 {
    //         let precomp: &Precomp;
    //         let addr: &mut Address;

    //         match SIZE {
    //             4 => {
    //                 precomp = &self.addr_u4_precomp;
    //                 addr = &mut self.addr_u4
    //             }
    //             5 => {
    //                 precomp = &self.addr_u5_precomp;
    //                 addr = &mut self.addr_u5
    //             }
    //             6 => {
    //                 precomp = &self.addr_u6_precomp;
    //                 addr = &mut self.addr_u6
    //             }
    //             _ => panic!("invalid operation selector size"),
    //         }

    //         #[cfg(debug_assertions)]
    //         {
    //             match SIZE {
    //                 4 => assert!(value < 1 << 4, "4 bits selector out of range"),
    //                 5 => assert!(value < 1 << 5, "5 bits selector out of range"),
    //                 6 => assert!(value < 1 << 6, "6 bits selector out of range"),
    //                 _ => panic!("invalid operation selector size"),
    //             }
    //         }

    //         // Bootstraps u4 address to X^{i}
    //         self.circuit_btp.bootstrap_to_address(
    //             module_pbs,
    //             module_lwe,
    //             &mut self.decomposer,
    //             precomp,
    //             value,
    //             addr,
    //             &mut self.tmp_bytes,
    //         );

    //         let (vec_znx_dft_tmp_bytes, tmp_bytes) = self
    //             .tmp_bytes
    //             .split_at_mut(module_lwe.bytes_of_vec_znx_dft(VMPPMAT_COLS));

    //         let mut tmp_b_dft: VecZnxDft =
    //             VecZnxDft::from_bytes_borrow(module_lwe, VMPPMAT_COLS, vec_znx_dft_tmp_bytes);

    //         // Selects according to offset
    //         addr.at_lsh(0)
    //             .product_inplace(module_lwe, LOGBASE2K, vec_znx, &mut tmp_b_dft, tmp_bytes);

    //         vec_znx.decode_coeff_i64(LOGBASE2K, LOGK, 0) as u32
    //     }
}

// pub fn next_tmp_bytes(module_pbs: &Module, module_lwe: &Module) -> usize {
//     read_tmp_bytes(module_lwe, RLWE_COLS, VMPPMAT_ROWS, VMPPMAT_COLS)
//         + bootstrap_address_tmp_bytes(module_pbs, module_lwe, VMPPMAT_COLS)
// }

// pub fn get_lwe_tmp_bytes(module_pbs: &Module, module_lwe: &Module) -> usize {
//     next_tmp_bytes(module_pbs, module_lwe)
// }
