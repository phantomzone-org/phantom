use crate::{
    address_conversion::FHEUintBlocksToAddress, get_base_2d, parameters::CryptographicParameters,
    Address, Base2D, Ram,
};

use poulpy_hal::{
    api::ModuleN,
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
        BDDKeyHelper, FheUint, FheUintBlocksPrepare, FheUintBlocksPreparedEncryptSk,
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
    pub rdu_rom: Ram,
    pub mu_rom: Ram,
    pub pcu_rom: Ram,

    pub registers: Ram,
    pub ram: Ram,
    pub ram_offset: u32,
    pub pc: FheUintPrepared<Vec<u8>, u32, BE>,
    pub pc_addr: Address<Vec<u8>>,
    pub rs1_addr: Address<Vec<u8>>,
    pub rs2_addr: Address<Vec<u8>>,
    pub rd_addr: Address<Vec<u8>>,
    pub rs1_addr_fhe_uint: FheUint<Vec<u8>, u32>, // Address value of RS1
    pub rs2_addr_fhe_uint: FheUint<Vec<u8>, u32>, // Address value of RS2
    pub rd_addr_fhe_uint: FheUint<Vec<u8>, u32>,  // Address value of RD
    pub rs1_val_fhe_uint: FheUint<Vec<u8>, u32>,  // Registers[RS1]
    pub rs2_val_fhe_uint: FheUint<Vec<u8>, u32>,  // Registers[RS2]
    pub imm_val_fhe_uint: FheUint<Vec<u8>, u32>,  // IMM
    pub rdu_val_fhe_uint: FheUint<Vec<u8>, u32>,  // RDU
    pub mu_val_fhe_uint: FheUint<Vec<u8>, u32>,  // MU
    pub pcu_val_fhe_uint: FheUint<Vec<u8>, u32>,  // PCU
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
        let rs1_rom: Ram = Ram::new(params, 32, &decomp_n, rom_size);
        let rs2_rom: Ram = Ram::new(params, 32, &decomp_n, rom_size);
        let rd_rom: Ram = Ram::new(params, 32, &decomp_n, rom_size);

        let rdu_rom: Ram = Ram::new(params, 32, &decomp_n, rom_size);
        let mu_rom: Ram = Ram::new(params, 32, &decomp_n, rom_size);
        let pcu_rom: Ram = Ram::new(params, 32, &decomp_n, rom_size);

        let registers: Ram = Ram::new(params, 32, &decomp_n, 32);
        let ram: Ram = Ram::new(params, 32, &decomp_n, ram_size);

        let base_2d_register: Base2D = get_base_2d(32, &[5].to_vec());
        let glwe_infos: &GLWELayout = &params.glwe_ct_infos();

        Self {
            imm_rom,
            rs1_rom,
            rs2_rom,
            rd_rom,
            rdu_rom,
            mu_rom,
            pcu_rom,

            registers,
            ram: ram,
            ram_offset: 0,
            pc: FheUintPrepared::alloc_from_infos(params.module(), &params.ggsw_infos()),
            pc_addr: Address::alloc_from_params(params, &get_base_2d(rom_size as u32, &decomp_n)),
            rs1_addr: Address::alloc_from_params(params, &base_2d_register),
            rs2_addr: Address::alloc_from_params(params, &base_2d_register),
            rd_addr: Address::alloc_from_params(params, &base_2d_register),
            rs1_addr_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs2_addr_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rd_addr_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs1_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs2_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            
            imm_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rdu_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            mu_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            pcu_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
        }
    }

    pub fn pc_encrypt_sk<S, M>(
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
        self.pc
            .encrypt_sk(module, pc_value, sk_prepared, source_xa, source_xe, scratch);
    }

    pub fn instructions_encrypt_sk<M, S>(
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
        let max_addr_rdu: usize = self.rdu_rom.max_addr();
        let max_addr_mu: usize = self.mu_rom.max_addr();
        let max_addr_pcu: usize = self.pcu_rom.max_addr();

        let mut data_ram_rs1: Vec<u32> = vec![0u32; max_addr_rs1];
        let mut data_ram_rs2: Vec<u32> = vec![0u32; max_addr_rs2];
        let mut data_ram_rd: Vec<u32> = vec![0u32; max_addr_rd];
        let mut data_ram_imm: Vec<u32> = vec![0u32; max_addr_imm];
        let mut data_ram_rdu: Vec<u32> = vec![0u32; max_addr_rdu];
        let mut data_ram_mu: Vec<u32> = vec![0u32; max_addr_mu];
        let mut data_ram_pcu: Vec<u32> = vec![0u32; max_addr_pcu];


        for i in 0..instructions.instructions.len() {
            data_ram_rs1[i] = instructions.get_raw(i).get_rs1_or_zero() as u32;
            data_ram_rs2[i] = instructions.get_raw(i).get_rs2_or_zero() as u32;
            data_ram_rd[i] = instructions.get_raw(i).get_rd_or_zero() as u32;
            data_ram_imm[i] = instructions.get_raw(i).get_immediate() as u32;
            let (rdu, mu, pcu) = instructions.get_raw(i).get_opid();
            data_ram_rdu[i] = rdu as u32;
            data_ram_mu[i] = mu as u32;
            data_ram_pcu[i] = pcu as u32;
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

        self.rdu_rom.encrypt_sk(
            module,
            &data_ram_rdu,
            sk_prepared,
            source_xa,
            source_xe,
            scratch,
        );
        self.mu_rom.encrypt_sk(
            module,
            &data_ram_mu,
            sk_prepared,
            source_xa,
            source_xe,
            scratch,
        );
        self.pcu_rom.encrypt_sk(
            module,
            &data_ram_pcu,
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
        module: &M,
        data: &[u32],
        sk_prepared: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        M: ModuleN + GLWESecretPreparedFactory<BE> + GLWEEncryptSk<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        self.registers
            .encrypt_sk(module, data, sk_prepared, source_xa, source_xe, scratch);
    }

    pub fn init_ram<M, S>(
        &mut self,
        module: &M,
        data: &[u32],
        sk_prepared: &S,
        source_xa: &mut Source,
        source_xe: &mut Source,
        scratch: &mut Scratch<BE>,
    ) where
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        M: ModuleN + GLWESecretPreparedFactory<BE> + GLWEEncryptSk<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert!(data.len() <= self.ram.max_addr());

        self.ram
            .encrypt_sk(module, data, sk_prepared, source_xa, source_xe, scratch);
    }

    pub fn read_instruction_components<M, K, H>(
        &mut self,
        module: &M,
        key: &H,
        scratch: &mut Scratch<BE>,
    )
    where
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
            .set_from_fheuint_prepared(module, &self.pc, scratch);

        self.imm_rom.read_to_fheuint(
            module,
            &mut self.imm_val_fhe_uint,
            &self.pc_addr,
            key,
            scratch,
        );

        self.rdu_rom.read_to_fheuint(
            module,
            &mut self.rdu_val_fhe_uint,
            &self.pc_addr,
            key,
            scratch,
        );

        self.mu_rom.read_to_fheuint(
            module,
            &mut self.mu_val_fhe_uint,
            &self.pc_addr,
            key,
            scratch,
        );

        self.pcu_rom.read_to_fheuint(
            module,
            &mut self.pcu_val_fhe_uint,
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

    pub fn read_registers<M, DK, H, K, BRA>(
        &mut self,
        module: &M,
        key: &H,
        scratch: &mut Scratch<BE>,
    ) -> (&FheUint<Vec<u8>, u32>, &FheUint<Vec<u8>, u32>)
    where
        BRA: BlindRotationAlgo,
        DK: DataRef,
        M: FheUintBlocksPreparedFactory<u32, BE>
            + FheUintBlocksPrepare<BRA, u32, BE>
            + FHEUintBlocksToAddress<u32, BE>
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>,
        H: BDDKeyHelper<DK, BRA, BE> + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        self.rs1_addr
            .set_from_fheuint(module, &self.rs1_addr_fhe_uint, key, scratch);

        self.rs2_addr
            .set_from_fheuint(module, &self.rs2_addr_fhe_uint, key, scratch);

        self.registers.read_to_fheuint(
            module,
            &mut self.rs1_val_fhe_uint,
            &self.rs1_addr,
            key,
            scratch,
        );
        self.registers.read_to_fheuint(
            module,
            &mut self.rs2_val_fhe_uint,
            &self.rs2_addr,
            key,
            scratch,
        );

        (&self.rs1_val_fhe_uint, &self.rs2_val_fhe_uint)
    }

    pub fn cycle<M, V, DK, H, K, BRA>(&mut self, module: &M, key: &H, scratch: &mut Scratch<BE>)
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
        DK: DataRef,
        V: GLWEAutomorphismKeyHelper<K, BE>,
        H: BDDKeyHelper<DK, BRA, BE> + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
    {
        self.read_instruction_components(module, key, scratch);
        self.read_registers(module, key, scratch);
    }
}
