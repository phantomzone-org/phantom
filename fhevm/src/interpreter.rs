use crate::{
    address_conversion::FHEUintBlocksToAddress, get_base_2d, parameters::CryptographicParameters,
    Address, Base2D, Ram,
};

use poulpy_hal::{
    api::{ModuleN, ScratchOwnedBorrow},
    layouts::{Backend, DataRef, Module, Scratch},
    source::Source,
};

use poulpy_core::{
    layouts::{
        GGLWEInfos, GGLWEPreparedToRef, GGSWPreparedFactory, GLWEAutomorphismKeyHelper, GLWEInfos,
        GLWELayout, GLWESecretPreparedFactory, GLWESecretPreparedToRef, GetGaloisElement,
    },
    GLWEEncryptSk, GLWEExternalProduct, GLWEPackerOps, GLWEPacking, GLWETrace, ScratchTakeCore,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{
        BDDKeyPrepared, FheUint, FheUintBlocksPrepare, FheUintBlocksPreparedEncryptSk,
        FheUintBlocksPreparedFactory, FheUintPrepared,
    },
    blind_rotation::BlindRotationAlgo,
};

use crate::instructions::InstructionsParser;

pub struct Interpreter<BE: Backend> {
    pub imm_rom: Ram,
    pub rs1_rom: Ram,
    pub rs2_rom: Ram,
    pub rd_rom: Ram,
    pub registers: Ram,
    pub ram: Ram,
    pub ram_offset: u32,
    pub program_counter: FheUintPrepared<Vec<u8>, u32, BE>,
    pub rs1_addr_fhe_uint: FheUint<Vec<u8>, u32>,
    pub rs2_addr_fhe_uint: FheUint<Vec<u8>, u32>,
    pub rd_addr_fhe_uint: FheUint<Vec<u8>, u32>,
    pub pc_addr: Address<Vec<u8>>,
    pub rs1_addr: Address<Vec<u8>>,
    pub rs2_addr: Address<Vec<u8>>,
    pub rd_addr: Address<Vec<u8>>,
    pub imm_addr_fhe_uint: FheUint<Vec<u8>, u32>,
    pub rs1_val_fhe_uint: FheUint<Vec<u8>, u32>,
    pub rs2_val_fhe_uint: FheUint<Vec<u8>, u32>,
    pub imm_val_fhe_uint: FheUint<Vec<u8>, u32>,
}

impl<BE: Backend> Interpreter<BE> {
    pub fn new(
        params: &CryptographicParameters<BE>,
        rom_size: usize,
        ram_size: usize,
        decomp_n: Vec<u8>,
    ) -> Self
    where
        Module<BE>: FheUintBlocksPreparedFactory<u32, BE>,
    {
        let imm_rom: Ram = Ram::new(params, 32, &decomp_n, rom_size);
        let rs1_rom: Ram = Ram::new(params, 5, &decomp_n, rom_size);
        let rs2_rom: Ram = Ram::new(params, 5, &decomp_n, rom_size);
        let rd_rom: Ram = Ram::new(params, 5, &decomp_n, rom_size);

        let registers: Ram = Ram::new(params, 32, &decomp_n, 32);
        let ram: Ram = Ram::new(params, 8, &decomp_n, ram_size);

        let base_2d_register: Base2D = get_base_2d(32, &[5].to_vec());
        let glwe_infos: &GLWELayout = &params.glwe_ct_infos();

        Self {
            imm_rom,
            rs1_rom,
            rs2_rom,
            rd_rom,
            registers,
            ram: ram,
            ram_offset: 0,
            program_counter: FheUintPrepared::alloc_from_infos(
                params.module(),
                &params.ggsw_infos(),
            ),
            pc_addr: Address::alloc_from_params(params, &get_base_2d(rom_size as u32, &decomp_n)),
            rs1_addr: Address::alloc_from_params(params, &base_2d_register),
            rs2_addr: Address::alloc_from_params(params, &base_2d_register),
            rd_addr: Address::alloc_from_params(params, &base_2d_register),
            imm_addr_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs1_addr_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs2_addr_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rd_addr_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs1_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs2_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
        }
    }

    pub fn set_pc_to<S, M>(
        &mut self,
        module: &M,
        pc_value: u32,
        sk_prepared: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        Scratch<BE>: ScratchTakeCore<BE>,
        M: FheUintBlocksPreparedEncryptSk<u32, BE>,
    {
        self.program_counter.encrypt_sk(
            module,
            pc_value,
            sk_prepared,
            source_xa,
            source_xe,
            scratch,
        );
    }

    pub fn init_instructions<M, S>(
        &mut self,
        module: &M,
        instructions: &InstructionsParser,
        sk_prepared: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        M: ModuleN + GLWESecretPreparedFactory<BE> + GLWEEncryptSk<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let max_addr_imm: usize = self.imm_rom.max_addr();
        let max_addr_rs1: usize = self.rs1_rom.max_addr();
        let max_addr_rs2: usize = self.rs2_rom.max_addr();
        let max_addr_rd: usize = self.rd_rom.max_addr();

        let rs1_word_size: usize = self.rs1_rom.word_size();
        let rs2_word_size: usize = self.rs2_rom.word_size();
        let rd_word_size: usize = self.rd_rom.word_size();
        let imm_word_size: usize = self.imm_rom.word_size();

        let mut data_ram_rs1: Vec<u8> = vec![0u8; max_addr_rs1 * rs1_word_size];
        let mut data_ram_rs2: Vec<u8> = vec![0u8; max_addr_rs2 * rs2_word_size];
        let mut data_ram_rd: Vec<u8> = vec![0u8; max_addr_rd * rd_word_size];
        let mut data_ram_imm: Vec<u8> = vec![0u8; max_addr_imm * imm_word_size];

        for i in 0..instructions.instructions.len() {
            let rs1 = instructions.get_raw(i).get_rs1_or_zero();
            let rs2 = instructions.get_raw(i).get_rs2_or_zero();
            let rd = instructions.get_raw(i).get_rd_or_zero();
            let imm = instructions.get_raw(i).get_immediate();

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

        self.rs1_rom.encrypt_sk(
            module,
            &data_ram_rs1,
            sk_prepared,
            source_xa,
            source_xe,
            scratch,
        );
        self.rs2_rom.encrypt_sk(
            module,
            &data_ram_rs2,
            sk_prepared,
            source_xa,
            source_xe,
            scratch,
        );
        self.rd_rom.encrypt_sk(
            module,
            &data_ram_rd,
            sk_prepared,
            source_xa,
            source_xe,
            scratch,
        );
        self.imm_rom.encrypt_sk(
            module,
            &data_ram_imm,
            sk_prepared,
            source_xa,
            source_xe,
            scratch,
        );
    }

    pub fn init_ram_offset(&mut self, ram_offset: u32) {
        self.ram_offset = ram_offset;
    }

    pub fn init_registers<M, S>(
        &mut self,
        registers: &Vec<u32>,
        module: &M,
        sk_prepared: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        M: ModuleN + GLWESecretPreparedFactory<BE> + GLWEEncryptSk<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let max_addr = self.registers.max_addr();
        let default_word_size = self.registers.word_size(); // TODO: hardcoded based on 8-bit plaintext precision

        let mut registers_data = vec![0u8; max_addr * default_word_size];
        for i in 0..registers.len() {
            for j in 0..default_word_size {
                registers_data[j + i * default_word_size] = ((registers[i] >> j) & 1) as u8;
            }
        }

        self.registers.encrypt_sk(
            module,
            &registers_data,
            sk_prepared,
            source_xa,
            source_xe,
            scratch,
        );
    }

    pub fn init_ram<M, S>(
        &mut self,
        ram: &Vec<u8>,
        module: &M,
        sk_prepared: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        M: ModuleN + GLWESecretPreparedFactory<BE> + GLWEEncryptSk<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert!(ram.len() <= self.ram.max_addr());
        let max_addr = self.ram.max_addr();
        let default_word_size = self.ram.word_size(); // TODO: hardcoded based on 8-bit plaintext precision
        let mut ram_data = vec![0u8; max_addr * default_word_size];
        for i in 0..ram.len() {
            for j in 0..default_word_size {
                ram_data[j + i * default_word_size] = ((ram[i] >> j) & 1) as u8;
            }
        }

        self.ram.encrypt_sk(
            module,
            &ram_data,
            sk_prepared,
            source_xa,
            source_xe,
            scratch,
        );
    }

    // TODO: add missing components
    pub fn read_instruction_components<M, K, H>(
        &mut self,
        module: &M,
        key: &H,
        scratch: &mut Scratch<BE>,
    ) where
        M: FHEUintBlocksToAddress<u32, BE>
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>,
        H: GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        self.pc_addr
            .set_from_fheuint_prepared(module, &self.program_counter, scratch);

        self.imm_rom.read_to_fheuint(
            module,
            &mut self.imm_addr_fhe_uint,
            &self.pc_addr,
            key,
            scratch,
        );

        self.rs1_rom.read_to_fheuint(
            module,
            &mut self.rs1_addr_fhe_uint,
            &self.pc_addr,
            key,
            scratch,
        );
        self.rs2_rom.read_to_fheuint(
            module,
            &mut self.rs2_addr_fhe_uint,
            &self.pc_addr,
            key,
            scratch,
        );
        self.rd_rom.read_to_fheuint(
            module,
            &mut self.rd_addr_fhe_uint,
            &self.pc_addr,
            key,
            scratch,
        );
    }

    pub fn read_registers<M, F, K, BRA>(
        &mut self,
        module: &M,
        vm_key: &K,
        scratch: &mut Scratch<BE>,
    ) where
        BRA: BlindRotationAlgo,
        F: DataRef,
        M: FheUintBlocksPreparedFactory<u32, BE>
            + FheUintBlocksPrepare<BRA, u32, BE>
            + FHEUintBlocksToAddress<u32, BE>
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>,
        K:,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        self.rs1_addr
            .set_from_fheuint(module, &self.rs1_addr_fhe_uint, bdd_key_prepared, scratch);

        self.rs2_addr
            .set_from_fheuint(module, &self.rs2_addr_fhe_uint, bdd_key_prepared, scratch);

        self.registers.read_to_fheuint(
            module,
            &mut self.rs1_val_fhe_uint,
            &self.rs1_addr,
            &self.bdd_key_prepared,
            scratch,
        );
        self.registers.read_to_fheuint(
            module,
            &mut self.rs2_val_fhe_uint,
            &self.rs2_addr,
            &self.bdd_key_prepared,
            scratch,
        );
    }

    pub fn cycle<M, V, K, BRA>(&mut self, module: &M, vm_keys: &V, scratch: &mut Scratch<BE>)
    where
        M: FHEUintBlocksToAddress<u32, BE>
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + FheUintBlocksPreparedFactory<u32, BE>
            + FheUintBlocksPrepare<BRA, u32, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        BRA: BlindRotationAlgo,
        V: GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
    {
        self.read_instruction_components(module, vm_keys, scratch);
        self.read_registers(module, vm_keys, scratch);
    }
}
