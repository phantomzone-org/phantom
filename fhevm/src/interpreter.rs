use std::collections::HashMap;

use crate::{
    address_read::AddressRead,
    address_write::AddressWrite,
    arithmetic::Evaluate,
    base::{get_base_2d, Base2D},
    keys::RAMKeysHelper,
    parameters::CryptographicParameters,
    ram::ram::Ram,
    store::Store,
    update_pc, Load, LOAD_OPS_LIST, RD_RV32I_OP_LIST, STORE_OPS_LIST,
};

use poulpy_hal::{
    api::{ModuleLogN, ModuleN},
    layouts::{Backend, DataRef, Module, Scratch},
    source::Source,
};

use poulpy_core::{
    layouts::{
        GGSWLayout, GGSWPreparedFactory, GLWEInfos, GLWELayout, GLWESecretPreparedFactory,
        GLWESecretPreparedToRef,
    },
    GGSWAutomorphism, GLWEAdd, GLWECopy, GLWEEncryptSk, GLWEExternalProduct, GLWENormalize,
    GLWEPackerOps, GLWEPacking, GLWERotate, GLWESub, GLWETrace, ScratchTakeCore,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{
        Add, BDDKeyHelper, ExecuteBDDCircuit, ExecuteBDDCircuit2WTo1W, FheUint, FheUintPrepare,
        FheUintPrepared, FheUintPreparedFactory, GGSWBlindRotation, GLWEBlinSelection,
    },
    blind_rotation::BlindRotationAlgo,
};

use crate::instructions::InstructionsParser;

pub enum InstructionSet {
    RV32,
    RV32I,
}

pub struct Interpreter<BE: Backend> {
    pub(crate) instruction_set: InstructionSet,
    pub(crate) base_2d_rom: Base2D,
    pub(crate) base_2d_registers: Base2D,
    pub(crate) ggsw_infos: GGSWLayout,

    // ROM
    pub(crate) imm_rom: Ram,
    pub(crate) rs1_rom: Ram,
    pub(crate) rs2_rom: Ram,
    pub(crate) rd_rom: Ram,
    pub(crate) rdu_rom: Ram,
    pub(crate) mu_rom: Ram,
    pub(crate) pcu_rom: Ram,

    // Registers
    pub(crate) registers: Ram,

    // RAM
    pub(crate) ram_bit_size: usize, // log2(#items)
    pub(crate) ram: Ram,
    pub(crate) ram_addr_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) ram_val_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) ram_addr_fhe_uint_prepared: FheUintPrepared<Vec<u8>, u32, BE>,

    // PC
    pub(crate) pc_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) pc_fhe_uint_prepared: FheUintPrepared<Vec<u8>, u32, BE>,

    // RS1
    pub(crate) rs1_addr_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) rs1_val_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) rs1_val_fhe_uint_prepared: FheUintPrepared<Vec<u8>, u32, BE>,

    // RS2
    pub(crate) rs2_addr_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) rs2_val_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) rs2_val_fhe_uint_prepared: FheUintPrepared<Vec<u8>, u32, BE>,

    // IMM
    pub(crate) imm_val_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) imm_val_fhe_uint_prepared: FheUintPrepared<Vec<u8>, u32, BE>,

    // RD
    pub(crate) rd_addr_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) rd_val_fhe_uint: FheUint<Vec<u8>, u32>,

    // OP ID GLWE
    pub(crate) rdu_val_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) rdu_val_fhe_uint_prepared: FheUintPrepared<Vec<u8>, u32, BE>,
    pub(crate) mu_val_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) mu_val_fhe_uint_prepared: FheUintPrepared<Vec<u8>, u32, BE>,
    pub(crate) pcu_val_fhe_uint: FheUint<Vec<u8>, u32>,
    pub(crate) pcu_val_fhe_uint_prepared: FheUintPrepared<Vec<u8>, u32, BE>,
}

impl<BE: Backend> Interpreter<BE> {
    pub fn new(
        params: &CryptographicParameters<BE>,
        rom_size: usize,
        ram_size: usize,
        decomp_n: Vec<u8>,
    ) -> Self
    where
        Module<BE>: FheUintPreparedFactory<u32, BE>,
    {
        let imm_rom: Ram = Ram::new(params, 32, rom_size);
        let rs1_rom: Ram = Ram::new(params, 32, rom_size);
        let rs2_rom: Ram = Ram::new(params, 32, rom_size);
        let rd_rom: Ram = Ram::new(params, 32, rom_size);

        let rdu_rom: Ram = Ram::new(params, 32, rom_size);
        let mu_rom: Ram = Ram::new(params, 32, rom_size);
        let pcu_rom: Ram = Ram::new(params, 32, rom_size);

        let registers: Ram = Ram::new(params, 32, 32);
        let ram: Ram = Ram::new(params, 32, ram_size);

        let base_2d_rom: Base2D = get_base_2d(rom_size as u32, &decomp_n);
        let base_2d_registers: Base2D = get_base_2d(32, &[5].to_vec());
        let glwe_infos: &GLWELayout = &params.glwe_ct_infos();
        let ggsw_infos: &GGSWLayout = &params.ggsw_infos();

        let module: &Module<BE> = params.module();

        Self {
            instruction_set: InstructionSet::RV32I,
            ggsw_infos: params.ggsw_infos(),
            base_2d_registers,
            base_2d_rom,
            imm_rom,
            rs1_rom,
            rs2_rom,
            rd_rom,
            rdu_rom,
            mu_rom,
            pcu_rom,
            registers,
            ram: ram,
            ram_addr_fhe_uint_prepared: FheUintPrepared::alloc_from_infos(module, ggsw_infos),
            ram_bit_size: (usize::BITS - (ram_size - 1).leading_zeros()) as usize,
            rd_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            ram_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            pcu_val_fhe_uint_prepared: FheUintPrepared::alloc_from_infos(module, ggsw_infos),
            mu_val_fhe_uint_prepared: FheUintPrepared::alloc_from_infos(module, ggsw_infos),
            rdu_val_fhe_uint_prepared: FheUintPrepared::alloc_from_infos(module, ggsw_infos),
            pc_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs1_addr_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs2_addr_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rd_addr_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rdu_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            mu_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            pcu_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs1_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs2_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            imm_val_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            ram_addr_fhe_uint: FheUint::alloc_from_infos(glwe_infos),
            rs1_val_fhe_uint_prepared: FheUintPrepared::alloc_from_infos(module, ggsw_infos),
            rs2_val_fhe_uint_prepared: FheUintPrepared::alloc_from_infos(module, ggsw_infos),
            imm_val_fhe_uint_prepared: FheUintPrepared::alloc_from_infos(module, ggsw_infos),
            pc_fhe_uint_prepared: FheUintPrepared::alloc_from_infos(module, ggsw_infos),
        }
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
            data_ram_imm[i] = instructions.get_raw(i).get_immediate() as u32;
            let (rs1, rs2, rd) = instructions.get_raw(i).get_registers();
            data_ram_rs1[i] = rs1 as u32;
            data_ram_rs2[i] = rs2 as u32;
            data_ram_rd[i] = rd as u32;
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

    pub fn ram_encrypt_sk<M, S>(
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

    pub fn cycle<M, DK, H, BRA>(&mut self, module: &M, keys: &H, scratch: &mut Scratch<BE>)
    where
        M: GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + FheUintPreparedFactory<u32, BE>
            + FheUintPrepare<BRA, u32, BE>
            + ExecuteBDDCircuit2WTo1W<u32, BE>
            + GLWEBlinSelection<u32, BE>
            + GGSWAutomorphism<BE>
            + GGSWBlindRotation<u32, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        BRA: BlindRotationAlgo,
        DK: DataRef,
        H: BDDKeyHelper<DK, BRA, BE> + RAMKeysHelper<DK, BE>,
    {
        // Retrive instructions components:
        // - addresses=[rs1, rs2, rd]
        // - imm
        // - opids=[rdu, mu, pcu]
        println!("Reading instruction components");
        self.read_instruction_components(module, keys, scratch);

        // Reads Register[rs1] and Register[rs2]
        println!("Reading registers");
        self.read_registers(module, keys, scratch);

        // Prepares FheUint imm, rs1, rs2 to FheUintPrepared
        println!("Preparing imm, rs1, rs2");
        self.prepare_imm_rs1_rs2(module, keys, scratch);

        // Computes rs2 + imm + offset
        println!("Deriving ram address");
        self.derive_ram_addr(module, keys, scratch);

        // Reads Ram[rs2 + imm + offset]
        println!("Reading ram");
        self.read_ram(module, keys, scratch);

        // Evaluates arithmetic over Register[rs1], Register[rs2], imm and pc
        println!("Deriving rd arithmetic");
        let mut ops: HashMap<usize, FheUint<Vec<u8>, u32>> = HashMap::new();

        match self.instruction_set {
            InstructionSet::RV32 => unimplemented!(),
            InstructionSet::RV32I => {
                self.derive_rd_arithmetic(module, &mut ops, RD_RV32I_OP_LIST, keys, scratch)
            }
        };

        // Finalizeses the loaded value from Ram[rs2 + imm + offset]
        println!("Finalizing rd load");
        self.derive_rd_load(module, &mut ops, LOAD_OPS_LIST, keys, scratch);

        // Selects value from the arithmetic operations and and Ram[rs2 + imm + offset]
        println!("Selecting rd");
        self.select_rd(module, ops, scratch);

        // Store value in Register[rd]
        println!("Storing rd");
        self.store_rd(module, keys, scratch);

        // Derive value to store in the ram
        println!("Deriving ram store");
        self.derive_ram_store(module, STORE_OPS_LIST, keys, scratch);

        // Stores value in Ram[rs2 + imm + offset]
        println!("Storing ram");
        self.store_ram(module, keys, scratch);

        // Updates PC
        println!("Updating pc");
        self.update_pc(module, keys, scratch);
    }

    pub fn update_pc<M, K, BRA: BlindRotationAlgo, D>(
        &mut self,
        module: &M,
        keys: &K,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleLogN
            + GLWEPacking<BE>
            + GLWECopy
            + ExecuteBDDCircuit<u32, BE>
            + FheUintPrepare<BRA, u32, BE>,
        K: RAMKeysHelper<D, BE> + BDDKeyHelper<D, BRA, BE>,
        D: DataRef,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        update_pc(
            module,
            &mut self.pc_fhe_uint,
            &self.rs1_val_fhe_uint_prepared,
            &self.rs2_val_fhe_uint_prepared,
            &self.pc_fhe_uint_prepared,
            &self.imm_val_fhe_uint_prepared,
            &self.pcu_val_fhe_uint_prepared,
            keys,
            scratch,
        );

        self.pc_fhe_uint_prepared
            .prepare(module, &self.pc_fhe_uint, keys, scratch);
    }

    pub fn store_ram<M, K, D, BRA>(&mut self, module: &M, keys: &K, scratch: &mut Scratch<BE>)
    where
        M: ModuleLogN
            + GGSWPreparedFactory<BE>
            + GGSWAutomorphism<BE>
            + GLWENormalize<BE>
            + GLWEAdd
            + GLWESub
            + GLWETrace<BE>
            + GLWERotate<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + FheUintPreparedFactory<u32, BE>
            + FheUintPrepare<BRA, u32, BE>
            + GGSWBlindRotation<u32, BE>,
        BRA: BlindRotationAlgo,
        K: RAMKeysHelper<D, BE> + BDDKeyHelper<D, BRA, BE>,
        D: DataRef,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let mut address =
            AddressWrite::alloc_from_infos(module, &self.ggsw_infos, &self.base_2d_rom);
        address.set_from_fhe_uint(module, &self.ram_addr_fhe_uint, keys, scratch);

        self.ram
            .write(module, &self.ram_val_fhe_uint, &address, keys, scratch);
    }

    pub fn derive_ram_store<M, D, O, K>(
        &mut self,
        module: &M,
        ops: &[O],
        keys: &K,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleLogN
            + GLWEBlinSelection<u32, BE>
            + GLWERotate<BE>
            + GLWETrace<BE>
            + GLWESub
            + GLWEAdd
            + GLWECopy,
        O: Store<u32, BE>,
        D: DataRef,
        K: RAMKeysHelper<D, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let mut res_tmp: HashMap<usize, FheUint<Vec<u8>, u32>> = HashMap::new();

        for op in ops {
            let mut tmp: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&self.imm_val_fhe_uint);
            op.store(
                module,
                &mut tmp,
                &self.rs2_val_fhe_uint,
                &self.ram_val_fhe_uint,
                &self.ram_addr_fhe_uint_prepared,
                keys,
                scratch,
            );
            res_tmp.insert(op.id(), tmp);
        }

        let mut res_tmp_ref: HashMap<usize, &mut FheUint<Vec<u8>, u32>> = HashMap::new();
        for (key, object) in res_tmp.iter_mut() {
            res_tmp_ref.insert(*key, object);
        }

        module.glwe_blind_selection(
            &mut self.ram_val_fhe_uint,
            res_tmp_ref,
            &self.mu_val_fhe_uint_prepared,
            2,
            self.ram_bit_size,
            scratch,
        );
    }

    pub fn store_rd<M, D, BRA, K>(&mut self, module: &M, keys: &K, scratch: &mut Scratch<BE>)
    where
        M: ModuleLogN
            + GGSWPreparedFactory<BE>
            + GGSWAutomorphism<BE>
            + GLWENormalize<BE>
            + GLWEAdd
            + GLWESub
            + GLWETrace<BE>
            + GLWERotate<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + FheUintPreparedFactory<u32, BE>
            + FheUintPrepare<BRA, u32, BE>
            + GGSWBlindRotation<u32, BE>,
        K: RAMKeysHelper<D, BE> + BDDKeyHelper<D, BRA, BE>,
        BRA: BlindRotationAlgo,
        D: DataRef,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let mut address =
            AddressWrite::alloc_from_infos(module, &self.ggsw_infos, &self.base_2d_registers);
        address.set_from_fhe_uint(module, &self.rd_addr_fhe_uint, keys, scratch);

        self.registers
            .write(module, &self.rd_val_fhe_uint, &address, keys, scratch);
    }

    pub fn derive_rd_load<M, K, L, D>(
        &self,
        module: &M,
        res: &mut HashMap<usize, FheUint<Vec<u8>, u32>>,
        ops: &[L],
        keys: &K,
        scratch: &mut Scratch<BE>,
    ) where
        M: ModuleLogN + GLWERotate<BE> + GLWETrace<BE> + GLWESub + GLWEAdd + GLWECopy,
        D: DataRef,
        K: RAMKeysHelper<D, BE>,
        L: Load<u32, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let ram_val: &FheUint<Vec<u8>, u32> = &self.ram_val_fhe_uint;
        for op in ops {
            let mut tmp: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&self.imm_val_fhe_uint);
            op.load(module, &mut tmp, ram_val, keys, scratch);
            res.insert(op.id(), tmp);
        }
    }

    pub fn select_rd<M>(
        &mut self,
        module: &M,
        mut ops: HashMap<usize, FheUint<Vec<u8>, u32>>,
        scratch: &mut Scratch<BE>,
    ) where
        M: GLWEBlinSelection<u32, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let mut ops_ref: HashMap<usize, &mut FheUint<Vec<u8>, u32>> = HashMap::new();
        for (key, object) in ops.iter_mut() {
            ops_ref.insert(*key, object);
        }

        module.glwe_blind_selection(
            &mut self.rd_val_fhe_uint,
            ops_ref,
            &self.rdu_val_fhe_uint_prepared,
            0,
            5,
            scratch,
        );
    }

    pub fn read_instruction_components<M, D, BRA, H>(
        &mut self,
        module: &M,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        M: GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + FheUintPrepare<BRA, u32, BE>
            + ModuleN
            + GGSWBlindRotation<u32, BE>
            + GGSWPreparedFactory<BE>,
        H: RAMKeysHelper<D, BE> + BDDKeyHelper<D, BRA, BE>,
        BRA: BlindRotationAlgo,
        D: DataRef,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        self.pc_fhe_uint_prepared
            .prepare(module, &self.pc_fhe_uint, keys, scratch);

        let mut address =
            AddressRead::alloc_from_infos(module, &self.ggsw_infos, &self.base_2d_rom);
        address.set_from_fhe_uint_prepared(module, &self.pc_fhe_uint_prepared, scratch);

        self.imm_rom
            .read(module, &mut self.imm_val_fhe_uint, &address, keys, scratch);

        self.rdu_rom
            .read(module, &mut self.rdu_val_fhe_uint, &address, keys, scratch);

        self.mu_rom
            .read(module, &mut self.mu_val_fhe_uint, &address, keys, scratch);

        self.pcu_rom
            .read(module, &mut self.pcu_val_fhe_uint, &address, keys, scratch);

        self.rs1_rom
            .read(module, &mut self.rs1_addr_fhe_uint, &address, keys, scratch);
        self.rs2_rom
            .read(module, &mut self.rs2_addr_fhe_uint, &address, keys, scratch);
        self.rd_rom
            .read(module, &mut self.rd_addr_fhe_uint, &address, keys, scratch);
    }

    pub fn prepare_imm_rs1_rs2<D, M, BRA, K>(
        &mut self,
        module: &M,
        keys: &K,
        scratch: &mut Scratch<BE>,
    ) -> (
        &FheUintPrepared<Vec<u8>, u32, BE>,
        &FheUintPrepared<Vec<u8>, u32, BE>,
        &FheUintPrepared<Vec<u8>, u32, BE>,
    )
    where
        K: BDDKeyHelper<D, BRA, BE>,
        D: DataRef,
        BRA: BlindRotationAlgo,
        M: FheUintPrepare<BRA, u32, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        self.imm_val_fhe_uint_prepared
            .prepare(module, &self.imm_val_fhe_uint, keys, scratch);
        self.rs1_val_fhe_uint_prepared
            .prepare(module, &self.rs1_addr_fhe_uint, keys, scratch);
        self.rs2_val_fhe_uint_prepared
            .prepare(module, &self.rs2_addr_fhe_uint, keys, scratch);
        (
            &self.imm_val_fhe_uint_prepared,
            &self.rs1_val_fhe_uint_prepared,
            &self.rs2_val_fhe_uint_prepared,
        )
    }

    pub fn derive_ram_addr<D, M, BRA, K>(&mut self, module: &M, keys: &K, scratch: &mut Scratch<BE>)
    where
        K: BDDKeyHelper<D, BRA, BE> + RAMKeysHelper<D, BE>,
        D: DataRef,
        BRA: BlindRotationAlgo,
        M: FheUintPrepare<BRA, u32, BE> + ExecuteBDDCircuit2WTo1W<u32, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        self.ram_addr_fhe_uint.add(
            module,
            &self.imm_val_fhe_uint_prepared,
            &self.rs2_val_fhe_uint_prepared,
            keys,
            scratch,
        );
        self.ram_addr_fhe_uint_prepared
            .prepare(module, &self.ram_addr_fhe_uint, keys, scratch);
    }

    pub fn read_ram<D, M, H>(&mut self, module: &M, keys: &H, scratch: &mut Scratch<BE>)
    where
        M: GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + ModuleN
            + GGSWBlindRotation<u32, BE>
            + GGSWPreparedFactory<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        H: RAMKeysHelper<D, BE>,
        D: DataRef,
    {
        let mut address: AddressRead<Vec<u8>, BE> =
            AddressRead::alloc_from_infos(module, &self.ggsw_infos, &self.base_2d_rom);
        address.set_from_fhe_uint_prepared(module, &self.ram_addr_fhe_uint_prepared, scratch);

        self.ram
            .read_prepare_write(module, &mut self.ram_val_fhe_uint, &address, keys, scratch)
    }

    pub fn read_registers<M, DK, H, BRA>(
        &mut self,
        module: &M,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) -> (&FheUint<Vec<u8>, u32>, &FheUint<Vec<u8>, u32>)
    where
        BRA: BlindRotationAlgo,
        DK: DataRef,
        M: FheUintPreparedFactory<u32, BE>
            + FheUintPrepare<BRA, u32, BE>
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + ModuleN
            + FheUintPreparedFactory<u32, BE>
            + FheUintPrepare<BRA, u32, BE>
            + GGSWBlindRotation<u32, BE>,
        H: BDDKeyHelper<DK, BRA, BE> + RAMKeysHelper<DK, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let mut address =
            AddressRead::alloc_from_infos(module, &self.ggsw_infos, &self.base_2d_registers);

        address.set_from_fheuint(module, &self.rs1_addr_fhe_uint, keys, scratch);

        self.registers
            .read(module, &mut self.rs1_val_fhe_uint, &address, keys, scratch);

        address.set_from_fheuint(module, &self.rs2_addr_fhe_uint, keys, scratch);

        self.registers.read_prepare_write(
            module,
            &mut self.rs2_val_fhe_uint,
            &address,
            keys,
            scratch,
        );

        (&self.rs1_val_fhe_uint, &self.rs2_val_fhe_uint)
    }

    pub fn derive_rd_arithmetic<M, H, O, D>(
        &mut self,
        module: &M,
        res: &mut HashMap<usize, FheUint<Vec<u8>, u32>>,
        ops: &[O],
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        M: ExecuteBDDCircuit2WTo1W<u32, BE>,
        H: RAMKeysHelper<D, BE>,
        D: DataRef,
        O: Evaluate<u32, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let rs1: &FheUintPrepared<Vec<u8>, u32, BE> = &self.rs1_val_fhe_uint_prepared;
        let rs2: &FheUintPrepared<Vec<u8>, u32, BE> = &self.rs2_val_fhe_uint_prepared;
        let imm: &FheUintPrepared<Vec<u8>, u32, BE> = &self.imm_val_fhe_uint_prepared;
        let pc: &FheUintPrepared<Vec<u8>, u32, BE> = &self.pc_fhe_uint_prepared;

        for op in ops {
            let mut tmp: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&self.imm_val_fhe_uint);
            op.eval(module, &mut tmp, rs1, rs2, imm, pc, keys, scratch);
            res.insert(op.id(), tmp);
        }
    }
}
