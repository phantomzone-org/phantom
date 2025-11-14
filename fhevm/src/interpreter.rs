use std::{collections::HashMap, time::Instant};

use crate::{
    address_read::AddressRead, address_write::AddressWrite, debug::InterpreterDebug,
    measure_duration, parameters::CryptographicParameters, ram::ram::Ram, ram_offset::ram_offset,
    ram_update::Store, rd_update::Evaluate, update_pc, Measurements, PerCycleMeasurements,
    RAM_UPDATE_OP_LIST, RD_UPDATE, RD_UPDATE_RV32I_OP_LIST,
};

use poulpy_hal::{
    api::{ModuleLogN, ModuleN},
    layouts::{Backend, DataRef, Module, Scratch},
    source::Source,
};

use poulpy_core::{
    layouts::{
        GGLWEInfos, GGLWEPreparedToRef, GGSWLayout, GGSWPreparedFactory, GLWEAutomorphismKeyHelper,
        GLWEInfos, GLWELayout, GLWESecretPrepared, GLWESecretPreparedToRef, GetGaloisElement,
    },
    GGSWEncryptSk, GLWEAdd, GLWECopy, GLWEDecrypt, GLWEEncryptSk, GLWEExternalProduct, GLWENoise,
    GLWENormalize, GLWEPackerOps, GLWEPacking, GLWERotate, GLWESub, GLWETrace, GetDistribution,
    ScratchTakeCore,
};
use poulpy_schemes::tfhe::{
    bdd_arithmetic::{
        BDDKeyHelper, BDDKeyInfos, Cmux, ExecuteBDDCircuit, ExecuteBDDCircuit2WTo1W, FheUint, FheUintPrepare, FheUintPrepared, FheUintPreparedEncryptSk, FheUintPreparedFactory, GGSWBlindRotation, GLWEBlinSelection
    },
    blind_rotation::BlindRotationAlgo,
};

use crate::instructions::InstructionsParser;

pub enum InstructionSet {
    RV32M,
    RV32I,
}

pub struct Interpreter<BE: Backend> {
    pub(crate) cycle: u32,
    pub(crate) vm_debug: Option<InterpreterDebug>,
    
    pub(crate) verbose_timings: bool,
    pub(crate) measurements: Measurements,

    pub(crate) instruction_set: InstructionSet,
    pub(crate) ggsw_infos: GGSWLayout,

    // ROM
    pub(crate) rom_bits_size: usize,
    pub(crate) imm_rom: Ram,
    pub(crate) rs1_rom: Ram,
    pub(crate) rs2_rom: Ram,
    pub(crate) rd_rom: Ram,
    pub(crate) rdu_rom: Ram,
    pub(crate) mu_rom: Ram,
    pub(crate) pcu_rom: Ram,

    // Registers
    pub(crate) reg_bits_size: usize,
    pub(crate) registers: Ram,

    // RAM
    pub(crate) ram_bit_size: usize, // log2(#items)
    pub(crate) ram_size: usize,
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
    pub fn new(params: &CryptographicParameters<BE>, rom_size: usize, ram_size: usize) -> Self
    where
        Module<BE>: FheUintPreparedFactory<u32, BE>,
    {
        Self::new_internal::<false>(params, rom_size, ram_size)
    }

    pub fn new_with_debug(
        params: &CryptographicParameters<BE>,
        rom_size: usize,
        ram_size: usize,
    ) -> Self
    where
        Module<BE>: FheUintPreparedFactory<u32, BE>,
    {
        Self::new_internal::<true>(params, rom_size, ram_size)
    }

    fn new_internal<const DEBUG: bool>(
        params: &CryptographicParameters<BE>,
        rom_size: usize,
        ram_size: usize,
    ) -> Self
    where
        Module<BE>: FheUintPreparedFactory<u32, BE>,
    {

        let verbose_timings = std::env::var("VERBOSE_TIMINGS")
            .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let imm_rom: Ram = Ram::new(params, 32, rom_size);
        let rs1_rom: Ram = Ram::new(params, 32, rom_size);
        let rs2_rom: Ram = Ram::new(params, 32, rom_size);
        let rd_rom: Ram = Ram::new(params, 32, rom_size);

        let rdu_rom: Ram = Ram::new(params, 32, rom_size);
        let mu_rom: Ram = Ram::new(params, 32, rom_size);
        let pcu_rom: Ram = Ram::new(params, 32, rom_size);

        let registers: Ram = Ram::new(params, 32, 32);
        let ram: Ram = Ram::new(params, 32, ram_size);

        let glwe_infos: &GLWELayout = &params.glwe_ct_infos();
        let ggsw_infos: &GGSWLayout = &params.ggsw_infos();

        let module: &Module<BE> = params.module();

        let vm_debug: Option<InterpreterDebug> = if DEBUG {
            Some(InterpreterDebug::new(rom_size, ram_size))
        } else {
            None
        };

        Self {
            vm_debug,
            verbose_timings,
            instruction_set: InstructionSet::RV32I,
            measurements: Measurements::new(),
            ggsw_infos: params.ggsw_infos(),
            imm_rom,
            rs1_rom,
            rs2_rom,
            rd_rom,
            rdu_rom,
            mu_rom,
            pcu_rom,
            registers,
            ram,
            ram_size,
            cycle: 0,
            ram_bit_size: (usize::BITS - (ram_size - 1).leading_zeros()) as usize,
            rom_bits_size: (usize::BITS - (rom_size - 1).leading_zeros()) as usize,
            reg_bits_size: 5,
            ram_addr_fhe_uint_prepared: FheUintPrepared::alloc_from_infos(module, ggsw_infos),
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
        M: ModuleN + GLWEEncryptSk<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        if let Some(vm_debug) = &mut self.vm_debug {
            vm_debug.set_instructions(instructions)
        }

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
            data_ram_imm[i] = instructions.get_raw(i).get_imm() as u32;
            let (rs2, rs1, rd) = instructions.get_raw(i).get_registers();
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
        M: ModuleN + GLWEEncryptSk<BE>,
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
        M: ModuleN + GLWEEncryptSk<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert!(data.len() <= self.ram.max_addr());

        if let Some(vm_debug) = &mut self.vm_debug {
            vm_debug.set_ram(data);
        }

        self.ram
            .encrypt_sk(module, data, sk_prepared, source_xa, source_xe, scratch);
    }

    pub fn ram_decrypt<M, S>(
        &mut self,
        module: &M,
        data_decrypted: &mut [u32],
        sk_prepared: &S,
        scratch: &mut Scratch<BE>,
    ) where
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        M: ModuleN + GLWEDecrypt<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        assert_eq!(data_decrypted.len(), self.ram.max_addr());

        self.ram
            .decrypt(module, data_decrypted, sk_prepared, scratch);
    }

    pub fn cycle<M, DK, H, K, BRA>(
        &mut self,
        threads: usize,
        module: &M,
        keys: &H,
        scratch: &mut Scratch<BE>,
    ) where
        M: Sync
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + FheUintPreparedFactory<u32, BE>
            + FheUintPrepare<BRA, BE>
            + ExecuteBDDCircuit2WTo1W<BE>
            + GLWEBlinSelection<u32, BE>
            + GGSWBlindRotation<u32, BE>
            + GLWENoise<BE>
            + GGSWEncryptSk<BE> + FheUintPreparedFactory<u32, BE> + FheUintPreparedEncryptSk<u32, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        BRA: BlindRotationAlgo,
        DK: DataRef,
        H: Sync + BDDKeyHelper<DK, BRA, BE> + BDDKeyInfos + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
    {
        self.cycle_internal(
            threads,
            module,
            keys,
            None::<&GLWESecretPrepared<Vec<u8>, BE>>,
            scratch,
        );
    }

    pub fn cycle_debug<M, DK, H, BRA, K, S>(
        &mut self,
        threads: usize,
        module: &M,
        keys: &H,
        sk: &S,
        scratch: &mut Scratch<BE>,
    ) where
        M: Sync
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + FheUintPreparedFactory<u32, BE>
            + FheUintPrepare<BRA, BE>
            + ExecuteBDDCircuit2WTo1W<BE>
            + GLWEBlinSelection<u32, BE>
            + GGSWBlindRotation<u32, BE>
            + GLWENoise<BE>
            + GGSWEncryptSk<BE> + FheUintPreparedFactory<u32, BE> + FheUintPreparedEncryptSk<u32, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        BRA: BlindRotationAlgo,
        DK: DataRef,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos + GetDistribution,
        H: Sync + BDDKeyHelper<DK, BRA, BE> + BDDKeyInfos + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
    {
        self.cycle_internal(threads, module, keys, Some(sk), scratch);
    }

    fn cycle_internal<M, DK, H, BRA, K, S>(
        &mut self,
        threads: usize,
        module: &M,
        keys: &H,
        sk: Option<&S>,
        scratch: &mut Scratch<BE>,
    ) where
        M: Sync
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + FheUintPreparedFactory<u32, BE>
            + FheUintPrepare<BRA, BE>
            + ExecuteBDDCircuit2WTo1W<BE>
            + GLWEBlinSelection<u32, BE>
            + GGSWBlindRotation<u32, BE>
            + GLWENoise<BE>
            + GGSWEncryptSk<BE> + FheUintPreparedFactory<u32, BE> + FheUintPreparedEncryptSk<u32, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        BRA: BlindRotationAlgo,
        DK: DataRef,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos + GetDistribution,
        H: Sync + BDDKeyHelper<DK, BRA, BE> + BDDKeyInfos + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
    {
        let mut this_cycle_measurement = PerCycleMeasurements::new();

        let start_cycle_time = Instant::now();
        // Retrive instructions components:
        // - addresses=[rs1, rs2, rd]
        // - imm
        // - opids=[rdu, mu, pcu]
        println!();
        println!(">>>>>>>>> CYCLE[{:03}] <<<<<<<<<<<", self.cycle);
        self.read_instruction_components(
            threads,
            module,
            keys,
            sk,
            scratch,
            &mut this_cycle_measurement,
        );

        // Reads Register[rs1] and Register[rs2]
        self.read_registers(threads, module, keys, sk, scratch, &mut this_cycle_measurement);

        // Prepares FheUint imm, rs1, rs2 to FheUintPrepared
        self.prepare_imm_rs1_rs2_values(threads, module, keys, scratch, &mut this_cycle_measurement);

        self.read_ram(
            threads,
            module,
            keys,
            sk,
            scratch,
            &mut this_cycle_measurement,
        );

        // Evaluates arithmetic over Register[rs1], Register[rs2], imm and pc
        match self.instruction_set {
            InstructionSet::RV32M => unimplemented!(),
            InstructionSet::RV32I => self.update_registers(
                threads,
                module,
                RD_UPDATE_RV32I_OP_LIST,
                keys,
                sk,
                scratch,
                &mut this_cycle_measurement,
            ),
        };

        // Stores value in Ram[rs2 + imm + offset]
        self.update_ram(threads, module, keys, sk, scratch, &mut this_cycle_measurement);

        // Updates PC
        self.update_pc(
            threads,
            module,
            keys,
            sk,
            scratch,
            &mut this_cycle_measurement,
        );
        self.cycle += 1;

        let end_cycle_time = Instant::now();
        let total_cycle_time = end_cycle_time.duration_since(start_cycle_time);
        this_cycle_measurement.total_cycle_time = total_cycle_time;

        if self.verbose_timings {
            self.measurements
            .cycle_measurements
            .push(this_cycle_measurement);
            println!();
            println!("Average cycle measurements:");
            println!(
                "- Total cycle: {:?}",
                self.measurements.average_cycle_time()
            );
            println!(
                "1. Read instruction components: {:?}",
                self.measurements
                    .average_cycle_time_read_instruction_components()
            );
            println!(
                "2. Read registers: {:?}",
                self.measurements.average_cycle_time_read_registers()
            );
            println!(
                "3. Prepare imm rs1 rs2 values: {:?}",
                self.measurements
                    .average_cycle_time_prepare_imm_rs1_rs2_values()
            );
            println!(
                "4. Read ram: {:?}",
                self.measurements.average_cycle_time_read_ram()
            );
            println!(
                "5. Update registers: {:?}",
                self.measurements.average_cycle_time_update_registers()
            );
            println!(
                "   - Evaluate rd ops: {:?}",
                self.measurements.average_cycle_time_evaluate_rd_ops()
            );
            println!(
                "   - Blind selection: {:?}",
                self.measurements.average_cycle_time_blind_selection()
            );
            println!(
                "   - Compute rd address: {:?}",
                self.measurements.average_cycle_time_compute_rd_address()
            );
            println!(
                "   - Write rd: {:?}",
                self.measurements.average_cycle_time_write_rd()
            );
            println!(
                "6. Update ram: {:?}",
                self.measurements.average_cycle_time_update_ram()
            );
            println!(
                "7. Update pc: {:?}",
                self.measurements.average_cycle_time_update_pc()
            );
            println!(
                "   - PCU prepare: {:?}",
                self.measurements.average_cycle_time_pcu_prepare()
            );
            println!(
                "   - PC update BDD: {:?}",
                self.measurements.average_cycle_time_pc_update_bdd()
            );
        }
    }

    pub(crate) fn read_instruction_components<M, D, BRA, H, K, S>(
        &mut self,
        threads: usize,
        module: &M,
        keys: &H,
        sk: Option<&S>,
        scratch: &mut Scratch<BE>,
        this_cycle_measurement: &mut PerCycleMeasurements,
    ) where
        M: Sync
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + FheUintPrepare<BRA, BE>
            + ModuleN
            + GGSWBlindRotation<u32, BE>
            + GGSWPreparedFactory<BE>
            + GLWEDecrypt<BE>
            + GLWENoise<BE> + GGSWEncryptSk<BE> + FheUintPreparedFactory<u32, BE> + FheUintPreparedEncryptSk<u32, BE>,
        H: Sync + BDDKeyHelper<D, BRA, BE> + BDDKeyInfos + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        BRA: BlindRotationAlgo,
        D: DataRef,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos + GetDistribution,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let start_time = if self.verbose_timings { Some(Instant::now()) } else { None };

        self.pc_fhe_uint_prepared.prepare_custom_multi_thread(
            threads,
            module,
            &self.pc_fhe_uint,
            0,
            self.rom_bits_size+2, // PC is 4bytes aligned
            keys,
            scratch,
        );

        let mut address =
            AddressRead::alloc_from_infos(module, &self.ggsw_infos, (1 << self.rom_bits_size) - 1);

        // Skip the first 2 bits because our rom is word alined instead of byte alined.
        address.set_from_fhe_uint_prepared(module, &self.pc_fhe_uint_prepared, 2, scratch);

        self.imm_rom.read(
            threads,
            module,
            &mut self.imm_val_fhe_uint,
            &address,
            keys,
            scratch,
        );

        self.rdu_rom.read(
            threads,
            module,
            &mut self.rdu_val_fhe_uint,
            &address,
            keys,
            scratch,
        );

        self.mu_rom.read(
            threads,
            module,
            &mut self.mu_val_fhe_uint,
            &address,
            keys,
            scratch,
        );

        self.pcu_rom.read(
            threads,
            module,
            &mut self.pcu_val_fhe_uint,
            &address,
            keys,
            scratch,
        );

        self.rs1_rom.read(
            threads,
            module,
            &mut self.rs1_addr_fhe_uint,
            &address,
            keys,
            scratch,
        );
        self.rs2_rom.read(
            threads,
            module,
            &mut self.rs2_addr_fhe_uint,
            &address,
            keys,
            scratch,
        );
        self.rd_rom.read(
            threads,
            module,
            &mut self.rd_addr_fhe_uint,
            &address,
            keys,
            scratch,
        );

        if self.verbose_timings {
            this_cycle_measurement.cycle_time_read_instruction_components = Instant::now().duration_since(start_time.unwrap());
        }

        if let (Some(sk), Some(vm_debug)) = (sk, &mut self.vm_debug) {

            vm_debug.read_instructions();

            let pc_have: u32 = self.pc_fhe_uint_prepared.decrypt(module, sk, keys, scratch);
            let imm_have: u32 = self.imm_val_fhe_uint.decrypt(module, sk, scratch);
            let rs1_have: u32 = self.rs1_addr_fhe_uint.decrypt(module, sk, scratch);
            let rs2_have: u32 = self.rs2_addr_fhe_uint.decrypt(module, sk, scratch);
            let rd_have: u32 = self.rd_addr_fhe_uint.decrypt(module, sk, scratch);
            let rdu_have: u32 = self.rdu_val_fhe_uint.decrypt(module, sk, scratch);
            let mu_have: u32 = self.mu_val_fhe_uint.decrypt(module, sk, scratch);
            let pcu_have: u32 = self.pcu_val_fhe_uint.decrypt(module, sk, scratch);

            let pc_want: u32 = vm_debug.pc;
            let imm_want: u32 = vm_debug.imm;
            let rs1_want: u32 = vm_debug.rs1_addr;
            let rs2_want: u32 = vm_debug.rs2_addr;
            let rd_want: u32 = vm_debug.rd_addr;
            let rdu_want: u32 = vm_debug.rdu;
            let mu_want: u32 = vm_debug.mu;
            let pcu_want: u32 = vm_debug.pcu;

            println!("READ ROM");
            let pc_val_fhe_uint_noise = self
                .pc_fhe_uint
                .noise(module, pc_want, sk, scratch)
                .max()
                .log2();
            println!(
                "   pc_val  : {pc_have:08x} - {pc_want:08x} - {:.2}",
                pc_val_fhe_uint_noise
            );
            this_cycle_measurement.pc_val_fhe_uint_noise = pc_val_fhe_uint_noise;
            

            let imm_val_fhe_uint_noise = self
                .imm_val_fhe_uint
                .noise(module, imm_want, sk, scratch)
                .max()
                .log2();
            println!(
                "   imm_val : {imm_have:08x} - {imm_want:08x} - {:.2}",
                imm_val_fhe_uint_noise
            );
            this_cycle_measurement.imm_val_fhe_uint_noise = imm_val_fhe_uint_noise;
            

            let rs1_addr_fhe_uint_noise = self
                .rs1_addr_fhe_uint
                .noise(module, rs1_want, sk, scratch)
                .max()
                .log2();
            println!(
                "   rs1_addr: {rs1_have:08x} - {rs1_want:08x} - {:.2}",
                rs1_addr_fhe_uint_noise
            );
            
            let rs2_addr_fhe_uint_noise = self
                .rs2_addr_fhe_uint
                .noise(module, rs2_want, sk, scratch)
                .max()
                .log2();
            println!(
                "   rs2_addr: {rs2_have:08x} - {rs2_want:08x} - {:.2}",
                rs2_addr_fhe_uint_noise
            );
            
            let rd_addr_fhe_uint_noise = self
                .rd_addr_fhe_uint
                .noise(module, rd_want, sk, scratch)
                .max()
                .log2();
            println!(
                "   rd_addr : {rd_have:08x} - {rd_want:08x} - {:.2}",
                rd_addr_fhe_uint_noise
            );
            
            let rdu_val_fhe_uint_noise = self
                .rdu_val_fhe_uint
                .noise(module, rdu_want, sk, scratch)
                .max()
                .log2();
            println!(
                "   rdu_val : {rdu_have:08x} - {rdu_want:08x} - {:.2}",
                rdu_val_fhe_uint_noise
            );
            
            let mu_val_fhe_uint_noise = self
                .mu_val_fhe_uint
                .noise(module, mu_want, sk, scratch)
                .max()
                .log2();
            println!(
                "   mu_val  : {mu_have:08x} - {mu_want:08x} - {:.2}",
                mu_val_fhe_uint_noise
            );
            
            let pcu_val_fhe_uint_noise = self
                .pcu_val_fhe_uint
                .noise(module, pcu_want, sk, scratch)
                .max()
                .log2();
            println!(
                "   pcu_val : {pcu_have:08x} - {pcu_want:08x} - {:.2}",
                pcu_val_fhe_uint_noise
            );
            assert_eq!(pc_have, pc_want);
            assert_eq!(imm_have, imm_want);
            assert_eq!(rs1_have, rs1_want);
            assert_eq!(rs2_have, rs2_want);
            assert_eq!(rd_have, rd_want);
            assert_eq!(rdu_have, rdu_want);
            assert_eq!(mu_have, mu_want);
            assert_eq!(pcu_have, pcu_want);
        }
    }

    pub(crate) fn read_registers<M, DK, H, BRA, K, S>(
        &mut self,
        threads: usize,
        module: &M,
        keys: &H,
        sk: Option<&S>,
        scratch: &mut Scratch<BE>,
        this_cycle_measurement: &mut PerCycleMeasurements,
    ) where
        BRA: BlindRotationAlgo,
        DK: DataRef,
        M: Sync
            + FheUintPreparedFactory<u32, BE>
            + FheUintPrepare<BRA, BE>
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + ModuleN
            + FheUintPreparedFactory<u32, BE>
            + FheUintPrepare<BRA, BE>
            + GGSWBlindRotation<u32, BE>
            + GLWEDecrypt<BE>
            + GLWENoise<BE>,
        H: Sync + BDDKeyHelper<DK, BRA, BE> + BDDKeyInfos + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let start_time = if self.verbose_timings { Some(Instant::now()) } else { None };

        let mut address: AddressRead<Vec<u8>, BE> =
            AddressRead::alloc_from_infos(module, &self.ggsw_infos, 31);

        address.set_from_fhe_uint(
            threads,
            module,
            &self.rs1_addr_fhe_uint,
            0,
            self.reg_bits_size,
            keys,
            scratch,
        );

        self.registers.read(
            threads,
            module,
            &mut self.rs1_val_fhe_uint,
            &address,
            keys,
            scratch,
        );

        address.set_from_fhe_uint(
            threads,
            module,
            &self.rs2_addr_fhe_uint,
            0,
            self.reg_bits_size,
            keys,
            scratch,
        );

        self.registers.read(
            threads,
            module,
            &mut self.rs2_val_fhe_uint,
            &address,
            keys,
            scratch,
        );

        if self.verbose_timings {
            this_cycle_measurement.cycle_time_read_registers = Instant::now().duration_since(start_time.unwrap());
        }

        if let (Some(sk), Some(vm_debug)) = (sk, &mut self.vm_debug) {
            vm_debug.read_registers();
            let rs1_have: u32 = self.rs1_val_fhe_uint.decrypt(module, sk, scratch);
            let rs2_have: u32 = self.rs2_val_fhe_uint.decrypt(module, sk, scratch);
            let rs1_want: u32 = vm_debug.rs1_val;
            let rs2_want: u32 = vm_debug.rs2_val;
            println!("READ RD");
            println!(
                "   rs1_val : {rs1_have:08x} - {rs1_want:08x} - {:.2}",
                self.rs1_val_fhe_uint
                    .noise(module, rs1_want, sk, scratch)
                    .max()
                    .log2()
            );
            assert_eq!(rs1_have, rs1_want);
            println!(
                "   rs2_val : {rs2_have:08x} - {rs2_want:08x} - {:.2}",
                self.rs2_val_fhe_uint
                    .noise(module, rs2_want, sk, scratch)
                    .max()
                    .log2()
            );
            assert_eq!(rs2_have, rs2_want);
        }
    }

    pub fn prepare_imm_rs1_rs2_values<D, M, BRA, K>(
        &mut self,
        threads: usize,
        module: &M,
        keys: &K,
        scratch: &mut Scratch<BE>,
        this_cycle_measurement: &mut PerCycleMeasurements,
    ) where
        K: BDDKeyHelper<D, BRA, BE> + BDDKeyInfos,
        D: DataRef,
        BRA: BlindRotationAlgo,
        M: FheUintPrepare<BRA, BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let start_time = if self.verbose_timings { Some(Instant::now()) } else { None };
        self.imm_val_fhe_uint_prepared.prepare_custom_multi_thread(
            threads,
            module,
            &self.imm_val_fhe_uint,
            0,
            32,
            keys,
            scratch,
        ); // TODO switch to 20 bits immediate & update circuits
        self.rs1_val_fhe_uint_prepared.prepare_custom_multi_thread(
            threads,
            module,
            &self.rs1_val_fhe_uint,
            0,
            32,
            keys,
            scratch,
        );
        self.rs2_val_fhe_uint_prepared.prepare_custom_multi_thread(
            threads,
            module,
            &self.rs2_val_fhe_uint,
            0,
            32,
            keys,
            scratch,
        );
        if self.verbose_timings {
            this_cycle_measurement.cycle_time_prepare_imm_rs1_rs2_values = Instant::now().duration_since(start_time.unwrap());
        }
    }

    pub(crate) fn read_ram<D, M, H, BRA, K, S>(
        &mut self,
        threads: usize,
        module: &M,
        keys: &H,
        sk: Option<&S>,
        scratch: &mut Scratch<BE>,
        this_cycle_measurement: &mut PerCycleMeasurements,
    ) where
        M: Sync
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + ModuleN
            + GGSWBlindRotation<u32, BE>
            + GGSWPreparedFactory<BE>
            + ExecuteBDDCircuit2WTo1W<BE>
            + FheUintPrepare<BRA, BE>
            + GLWEBlinSelection<u32, BE>
            + GLWENoise<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        H: Sync + BDDKeyHelper<D, BRA, BE> + BDDKeyInfos + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        BRA: BlindRotationAlgo,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        D: DataRef,
    {
        let start_time = if self.verbose_timings { Some(Instant::now()) } else { None };
        // Derives ram address = [rs2 + imm + 2^18]
        ram_offset(
            threads,
            module,
            &mut self.ram_addr_fhe_uint,
            &self.imm_val_fhe_uint_prepared,
            &self.rs1_val_fhe_uint_prepared,
            keys,
            scratch,
        );

        self.ram_addr_fhe_uint_prepared.prepare_custom_multi_thread(
            threads,
            module,
            &self.ram_addr_fhe_uint,
            0,
            self.ram_bit_size + 2, // ram_bit_size is 4bytes alined
            keys,
            scratch,
        );

        // Derives address for read
        let mut address: AddressRead<Vec<u8>, BE> =
            AddressRead::alloc_from_infos(module, &self.ggsw_infos, (1 << self.ram_bit_size) - 1);
        address.set_from_fhe_uint_prepared(module, &self.ram_addr_fhe_uint_prepared, 2, scratch);

        // Read ram_val_fhe_uint from Ram[rs2 + imm]
        self.ram.read_prepare_write(
            threads,
            module,
            &mut self.ram_val_fhe_uint,
            &address,
            keys,
            scratch,
        );
        if self.verbose_timings {
            this_cycle_measurement.cycle_time_read_ram = Instant::now().duration_since(start_time.unwrap());
        }

        if let (Some(sk), Some(vm_debug)) = (sk, &mut self.vm_debug) {
            vm_debug.read_ram();
            let ram_addr_have: u32 = self.ram_addr_fhe_uint.decrypt(module, sk, scratch);
            let ram_val_have: u32 = self.ram_val_fhe_uint.decrypt(module, sk, scratch);
            let ram_addr_want: u32 = vm_debug.ram_addr;
            let ram_val_want: u32 = vm_debug.ram_val;
            println!("READ RAM");
            let ram_addr_read_noise = self
                .ram_addr_fhe_uint
                .noise(module, ram_addr_want, sk, scratch)
                .max()
                .log2();
            println!(
                "   ram_addr: {ram_addr_have:08x} - {ram_addr_want:08x} - {:.2}",
                ram_addr_read_noise
            );
            this_cycle_measurement.ram_addr_read_noise = ram_addr_read_noise;
            assert_eq!(
                ram_addr_have, ram_addr_want,
                "ram addr noise {ram_addr_read_noise:.2}"
            );
            let ram_val_read_noise = self
                .ram_val_fhe_uint
                .noise(module, ram_val_want, sk, scratch)
                .max()
                .log2();
            println!(
                "   ram_val : {ram_val_have:08x} - {ram_val_want:08x} - {:.2}",
                ram_val_read_noise
            );
            this_cycle_measurement.ram_val_read_noise = ram_val_read_noise;
            assert_eq!(ram_val_have, ram_val_want);
        }
    }

    pub(crate) fn update_registers<M, H, D, BRA, K, S>(
        &mut self,
        threads: usize,
        module: &M,
        ops: &[RD_UPDATE],
        keys: &H,
        sk: Option<&S>,
        scratch: &mut Scratch<BE>,
        this_cycle_measurement: &mut PerCycleMeasurements,
    ) where
        M: Sync
            + ExecuteBDDCircuit2WTo1W<BE>
            + GLWEBlinSelection<u32, BE>
            + ModuleLogN
            + GLWERotate<BE>
            + GLWETrace<BE>
            + GLWESub
            + GLWEAdd
            + GLWECopy
            + GGSWPreparedFactory<BE>
            + ModuleN
            + FheUintPreparedFactory<u32, BE>
            + FheUintPrepare<BRA, BE>
            + GGSWBlindRotation<u32, BE>
            + GLWENormalize<BE>
            + GLWEExternalProduct<BE>
            + GLWENoise<BE>
            + GLWEPackerOps<BE>,
        BRA: BlindRotationAlgo,
        H: Sync + BDDKeyHelper<D, BRA, BE> + BDDKeyInfos + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        D: DataRef,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let start_time = if self.verbose_timings { Some(Instant::now()) } else { None };
        let rs1: &FheUintPrepared<Vec<u8>, u32, BE> = &self.rs1_val_fhe_uint_prepared;
        let rs2: &FheUintPrepared<Vec<u8>, u32, BE> = &self.rs2_val_fhe_uint_prepared;
        let imm: &FheUintPrepared<Vec<u8>, u32, BE> = &self.imm_val_fhe_uint_prepared;
        let pc: &FheUintPrepared<Vec<u8>, u32, BE> = &self.pc_fhe_uint_prepared;
        let ram_val: &FheUint<Vec<u8>, u32> = &self.ram_val_fhe_uint;

        let mut rd_map: HashMap<u32, FheUint<Vec<u8>, u32>> = HashMap::new();

        // Evaluates arithmetic operations & store in map with respective op ID
        this_cycle_measurement.cycle_time_evaluate_rd_ops = measure_duration(|| {
            for op in ops {
                let mut tmp: FheUint<Vec<u8>, u32> =
                    FheUint::alloc_from_infos(&self.imm_val_fhe_uint);
                op.eval_enc(
                    threads, module, &mut tmp, rs1, rs2, imm, pc, ram_val, keys, scratch,
                );
                rd_map.insert(op.id(), tmp);
            }
        });

        // Blind selection of the correct rd value using rdu_val_fhe_uint_prepared
        let start_time_blind_selection = if self.verbose_timings { Some(Instant::now()) } else { None };
        
        let mut ops_ref: HashMap<usize, &mut FheUint<Vec<u8>, u32>> = HashMap::new();
        for (key, object) in rd_map.iter_mut() {
            ops_ref.insert(*key as usize, object);
        }

        let ops_bit_size: usize = (usize::BITS - (ops.len() - 1).leading_zeros()) as usize;

        self.rdu_val_fhe_uint_prepared.prepare_custom_multi_thread(
            threads,
            module,
            &self.rdu_val_fhe_uint,
            0,
            ops_bit_size,
            keys,
            scratch,
        );
        module.glwe_blind_selection(
            &mut self.rd_val_fhe_uint,
            ops_ref,
            &self.rdu_val_fhe_uint_prepared,
            0,
            ops_bit_size,
            scratch,
        );
        
        if self.verbose_timings {
            this_cycle_measurement.cycle_time_blind_selection = Instant::now().duration_since(start_time_blind_selection.unwrap());
        }

        let mut address_read: AddressRead<Vec<u8>, BE> =
            AddressRead::alloc_from_infos(module, &self.ggsw_infos, 31);
        let mut address_write: AddressWrite<Vec<u8>, BE> =
            AddressWrite::alloc_from_infos(module, &self.ggsw_infos, 31);

        // Computes rd address
        let start_time_compute_rd_address = if self.verbose_timings { Some(Instant::now()) } else { None };
        address_read.set_from_fhe_uint(
            threads,
            module,
            &self.rd_addr_fhe_uint,
            0,
            self.reg_bits_size,
            keys,
            scratch,
        );

        address_write.set_from_fhe_uint(
            threads,
            module,
            &self.rd_addr_fhe_uint,
            0,
            self.reg_bits_size,
            keys,
            scratch,
        );
        if self.verbose_timings {
            this_cycle_measurement.cycle_time_compute_rd_address = Instant::now().duration_since(start_time_compute_rd_address.unwrap());
        }

        let mut tmp: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&self.rd_addr_fhe_uint);

        // Stores rd value in register
        let start_time_write_rd = if self.verbose_timings { Some(Instant::now()) } else { None };
        self.registers.read_prepare_write(
            threads,
            module,
            &mut tmp,
            &address_read,
            keys,
            scratch,
        );
        self.registers.write(
            threads,
            module,
            &self.rd_val_fhe_uint,
            &address_write,
            keys,
            scratch,
        );

        self.registers.zero(threads, module, 0, keys, scratch);

        if self.verbose_timings {
            this_cycle_measurement.cycle_time_write_rd = Instant::now().duration_since(start_time_write_rd.unwrap());
        }

        if self.verbose_timings {
            this_cycle_measurement.cycle_time_update_registers = Instant::now().duration_since(start_time.unwrap());
        }

        if let (Some(sk), Some(vm_debug)) = (sk, &mut self.vm_debug) {
            vm_debug.update_registers(ops);

            let rd_have: u32 = self.rd_val_fhe_uint.decrypt(module, sk, scratch);
            let rd_want: u32 = vm_debug.rd_val;
            println!("WRITE RD");
            let rd_val_fhe_uint_noise = self
                .rd_val_fhe_uint
                .noise(module, rd_want, sk, scratch)
                .max()
                .log2();
            println!(
                "   rd_val  : {rd_have:08x} - {rd_want:08x} - {:.2}",
                rd_val_fhe_uint_noise
            );
            assert_eq!(rd_have, rd_want);
            this_cycle_measurement.rd_val_fhe_uint_noise = rd_val_fhe_uint_noise;

            let mut registers_have: Vec<u32> = vec![0u32; 32];
            self.registers
                .decrypt(module, &mut registers_have, sk, scratch);
            let registers_want: &[u32; 32] = &vm_debug.registers;
            //for i in 0..self.ram_size{
            //   println!("RAM[{:02}]: {:08x} - {:08x}", i, ram_have[i], ram_want[i]);
            //}
            println!("reg: {:?}", registers_have);
            assert_eq!(registers_have, registers_want);
        }
    }

    pub(crate) fn update_ram<D, M, H, BRA, K, S>(
        &mut self,
        threads: usize,
        module: &M,
        keys: &H,
        sk: Option<&S>,
        scratch: &mut Scratch<BE>,
        this_cycle_measurement: &mut PerCycleMeasurements,
    ) where
        M: Sync
            + GGSWPreparedFactory<BE>
            + GLWEExternalProduct<BE>
            + GLWEPackerOps<BE>
            + GLWETrace<BE>
            + GLWEPacking<BE>
            + ModuleN
            + GGSWBlindRotation<u32, BE>
            + GGSWPreparedFactory<BE>
            + ExecuteBDDCircuit2WTo1W<BE>
            + FheUintPrepare<BRA, BE>
            + GLWEBlinSelection<u32, BE>
            + GLWENoise<BE>
            + GGSWEncryptSk<BE>,
        Scratch<BE>: ScratchTakeCore<BE>,
        H: Sync + BDDKeyHelper<D, BRA, BE> + BDDKeyInfos + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        BRA: BlindRotationAlgo,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos + GetDistribution,
        D: DataRef,
    {
        let start_time = if self.verbose_timings { Some(Instant::now()) } else { None };
        // Constructs diffferent possible values that are stored back
        let mut res_tmp: HashMap<u32, FheUint<Vec<u8>, u32>> = HashMap::new();
        if let Some(sk) = sk{
            for op in RAM_UPDATE_OP_LIST {
                let mut tmp: FheUint<Vec<u8>, u32> = FheUint::alloc_from_infos(&self.imm_val_fhe_uint);
                op.eval_enc(
                    threads,
                    module,
                    &mut tmp,
                    &self.rs2_val_fhe_uint,
                    &self.ram_val_fhe_uint,
                    &self.ram_addr_fhe_uint_prepared, // offset is the 2 LSB of [rs2 + imm]
                    keys,
                    sk,
                    scratch,
                );
                res_tmp.insert(op.id(), tmp);
            }
        }

        // Blind selection of the value to store
        let mut res_tmp_ref: HashMap<usize, &mut FheUint<Vec<u8>, u32>> = HashMap::new();
        for (key, object) in res_tmp.iter_mut() {
            res_tmp_ref.insert(*key as usize, object);
        }
        let ops_bit_size: usize =
            (usize::BITS - (RAM_UPDATE_OP_LIST.len() - 1).leading_zeros()) as usize;
        self.mu_val_fhe_uint_prepared.prepare_custom_multi_thread(
            threads,
            module,
            &self.mu_val_fhe_uint,
            0,
            ops_bit_size,
            keys,
            scratch,
        );
        module.glwe_blind_selection(
            &mut self.ram_val_fhe_uint,
            res_tmp_ref,
            &self.mu_val_fhe_uint_prepared,
            0,
            ops_bit_size,
            scratch,
        );

        // Derives address for write
        let mut address: AddressWrite<Vec<u8>, BE> =
            AddressWrite::alloc_from_infos(module, &self.ggsw_infos, (1 << self.ram_bit_size) - 1);
        address.set_from_fhe_uint_prepared(module, &self.ram_addr_fhe_uint_prepared, 2, scratch);

        self.ram.write(
            threads,
            module,
            &self.ram_val_fhe_uint,
            &address,
            keys,
            scratch,
        );

        if self.verbose_timings {
            this_cycle_measurement.cycle_time_update_ram = Instant::now().duration_since(start_time.unwrap());
        }

        if let (Some(sk), Some(vm_debug)) = (sk, &mut self.vm_debug) {
            vm_debug.update_ram();
            let ram_have: u32 = self.ram_val_fhe_uint.decrypt(module, sk, scratch);
            let ram_want: u32 = vm_debug.ram_val;
            println!("WRITE RAM");
            println!(
                "   ram_val : {ram_have:08x} - {ram_want:08x} - {:.2}",
                self.ram_val_fhe_uint
                    .noise(module, ram_want, sk, scratch)
                    .max()
                    .log2()
            );
            assert_eq!(ram_have, ram_want);
            let mut ram_have: Vec<u32> = vec![0u32; self.ram_size];
            self.ram.decrypt(module, &mut ram_have, sk, scratch);
            let ram_want: &Vec<u32> = &vm_debug.ram;
            //for i in 0..self.ram_size {
            //    println!(
            //        "RAM[{:02}]: {:08x} - {:08x} : {}",
            //        i,
            //        ram_have[i],
            //        ram_want[i],
            //        ram_have[i] - ram_want[i]
            //    );
            //}
            println!(
                "ram:addrss: {}",
                self.ram_addr_fhe_uint_prepared
                    .decrypt(module, sk, keys, scratch)
            );
            assert_eq!(&ram_have, ram_want);
        }
    }

    pub(crate) fn update_pc<M, K, S, H, BRA: BlindRotationAlgo, D>(
        &mut self,
        threads: usize,
        module: &M,
        keys: &H,
        sk: Option<&S>,
        scratch: &mut Scratch<BE>,
        this_cycle_measurement: &mut PerCycleMeasurements,
    ) where
        M: ModuleLogN
            + GLWEPacking<BE>
            + GLWECopy
            + ExecuteBDDCircuit<BE>
            + FheUintPrepare<BRA, BE>
            + Cmux<BE>
            + GLWEDecrypt<BE>
            + GLWENoise<BE>,
        S: GLWESecretPreparedToRef<BE> + GLWEInfos,
        H: BDDKeyHelper<D, BRA, BE> + BDDKeyInfos + GLWEAutomorphismKeyHelper<K, BE>,
        K: GGLWEPreparedToRef<BE> + GGLWEInfos + GetGaloisElement,
        D: DataRef,
        Scratch<BE>: ScratchTakeCore<BE>,
    {
        let start_time = if self.verbose_timings { Some(Instant::now()) } else { None };

        let start_time_pcu_prepare = if self.verbose_timings { Some(Instant::now()) } else { None };
        self.pcu_val_fhe_uint_prepared.prepare_custom_multi_thread(
            threads,
            module,
            &self.pcu_val_fhe_uint,
            0,
            4,
            keys,
            scratch,
        );
        if self.verbose_timings {
            this_cycle_measurement.cycle_time_pcu_prepare = Instant::now().duration_since(start_time_pcu_prepare.unwrap());
        }

        let start_time_pc_update_bdd = if self.verbose_timings { Some(Instant::now()) } else { None };
        update_pc(
            threads,
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
        if self.verbose_timings {
            this_cycle_measurement.cycle_time_pc_update_bdd = Instant::now().duration_since(start_time_pc_update_bdd.unwrap());
        }

        if self.verbose_timings {
            this_cycle_measurement.cycle_time_update_pc = Instant::now().duration_since(start_time.unwrap());
        }

        if let (Some(sk), Some(vm_debug)) = (sk, &mut self.vm_debug) {
            vm_debug.update_pc();
            let pc_have: u32 = self.pc_fhe_uint.decrypt(module, sk, scratch);
            let pc_want: u32 = vm_debug.pc;
            println!("UPDATE PC");
            println!(
                "   pc_val  : {pc_have:08x} - {pc_want:08x} - {:.2}",
                self.pc_fhe_uint
                    .noise(module, pc_want, sk, scratch)
                    .max()
                    .log2()
            );
            assert_eq!(pc_have, pc_want);
        }
    }
}
