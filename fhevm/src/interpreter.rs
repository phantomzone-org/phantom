use crate::{Address, CryptographicParameters, EvaluationKeys, EvaluationKeysPrepared, Parameters, Ram, TEST_BDD_KEY_LAYOUT};

use poulpy_backend::FFT64Ref as BackendImpl;
use poulpy_hal::{
    api::{ScratchOwnedAlloc, ScratchOwnedBorrow},
    layouts::{ScratchOwned},
    source::Source
};

use poulpy_core::{layouts::{GLWESecret, GLWESecretPrepared, LWESecret}};
use poulpy_schemes::tfhe::{bdd_arithmetic::{BDDKey, BDDKeyPrepared, FheUint, FheUintPrepared}, blind_rotation::CGGI};

use crate::{instructions::{
    InstructionsParser
}};

pub struct Interpreter {
    pub params: CryptographicParameters<BackendImpl>,
    pub source_xa: Source,
    pub source_xe: Source,
    pub fhe_ram_keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl>,
    pub bdd_key_prepared: BDDKeyPrepared<Vec<u8>, CGGI, BackendImpl>,
    pub imm_rom: Ram<BackendImpl>,
    pub rs1_rom: Ram<BackendImpl>,
    pub rs2_rom: Ram<BackendImpl>,
    pub rd_rom: Ram<BackendImpl>,
    pub registers: Ram<BackendImpl>,
    pub ram: Ram<BackendImpl>,
    pub ram_offset: u32,
    pub program_counter: FheUintPrepared<Vec<u8>, u32, BackendImpl>,
}

impl Interpreter {
    pub fn new(sk_lwe: &LWESecret<Vec<u8>>, sk_glwe: &GLWESecret<Vec<u8>>) -> Self {
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
        
        let params: Parameters<BackendImpl> = crate::Parameters::<BackendImpl>::new();

        let keys: EvaluationKeys<Vec<u8>> =
            EvaluationKeys::encrypt_sk(&params, sk_glwe, &mut source_xa, &mut source_xe);

        let mut fhe_ram_keys_prepared: EvaluationKeysPrepared<Vec<u8>, BackendImpl> =
            EvaluationKeysPrepared::alloc(&params);
        fhe_ram_keys_prepared.prepare(params.module(), &keys, scratch.borrow());

        let mut bdd_key: BDDKey<Vec<u8>, CGGI> = BDDKey::alloc_from_infos(&TEST_BDD_KEY_LAYOUT);
        bdd_key.encrypt_sk(params.module(), sk_lwe, sk_glwe, &mut source_xa, &mut source_xe, scratch.borrow());

        let mut bdd_key_prepared: BDDKeyPrepared<Vec<u8>, CGGI, BackendImpl> = BDDKeyPrepared::alloc_from_infos(params.module(), &TEST_BDD_KEY_LAYOUT);
        bdd_key_prepared.prepare(params.module(), &bdd_key, scratch.borrow());

        Self {
            params: CryptographicParameters::<BackendImpl>::new(),
            source_xa,
            source_xe,
            fhe_ram_keys_prepared,
            bdd_key_prepared,
            imm_rom,
            rs1_rom,
            rs2_rom,
            rd_rom,
            registers,
            ram: ram,
            ram_offset: 0,
            program_counter: FheUintPrepared::alloc_from_infos(params.module(), &params.ggsw_infos()),
        }
    }

    pub fn init_pc(&mut self, sk_glwe_prepared: &GLWESecretPrepared<Vec<u8>, BackendImpl>) {
        let pc_value = 0;

        // TODO: set scratch correctly.
        let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 22);

        self.program_counter.encrypt_sk(
            self.params.module(),
            pc_value,
            sk_glwe_prepared,
            &mut self.source_xa,
            &mut self.source_xe,
            scratch.borrow(),
        );
    }

    pub fn set_pc_to(&mut self, pc_value: u32, sk_glwe_prepared: &GLWESecretPrepared<Vec<u8>, BackendImpl>) {
        // TODO: set scratch correctly.
        let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 22);

        self.program_counter.encrypt_sk(
            self.params.module(),
            pc_value,
            sk_glwe_prepared,
            &mut self.source_xa,
            &mut self.source_xe,
            scratch.borrow(),
        );
    }    

    pub fn init_instructions(&mut self, sk_glwe: &GLWESecret<Vec<u8>>, instructions: &InstructionsParser) {
        
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

    // TODO: add missing components
    pub fn read_instruction_components(&mut self) -> (FheUint<Vec<u8>, u32>, FheUint<Vec<u8>, u32>, FheUint<Vec<u8>, u32>, FheUint<Vec<u8>, u32>) {
        let params = Parameters::<BackendImpl>::new(); // TODO: pass params, instead of hardcoding it
        let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 22); // TODO: Set correct scratch

        let mut instruction_address: Address<Vec<u8>> = Address::alloc_from_params(&params);
        instruction_address.set_from_fheuint_prepared(self.params.module(), &self.program_counter, scratch.borrow());

        let imm_fheuint: FheUint<Vec<u8>, u32> = self.imm_rom.read_to_fheuint(&instruction_address, &self.fhe_ram_keys_prepared, &self.bdd_key_prepared);
        
        let rs1_fheuint: FheUint<Vec<u8>, u32> = self.rs1_rom.read_to_fheuint(&instruction_address, &self.fhe_ram_keys_prepared, &self.bdd_key_prepared);
        let rs2_fheuint: FheUint<Vec<u8>, u32> = self.rs2_rom.read_to_fheuint(&instruction_address, &self.fhe_ram_keys_prepared, &self.bdd_key_prepared);
        let rd_fheuint: FheUint<Vec<u8>, u32> = self.rd_rom.read_to_fheuint(&instruction_address, &self.fhe_ram_keys_prepared, &self.bdd_key_prepared);

        (imm_fheuint, rs1_fheuint, rs2_fheuint, rd_fheuint)
    }

    pub fn read_registers(&mut self, rs1_fheuint:FheUint<Vec<u8>, u32>, rs2_fheuint:FheUint<Vec<u8>, u32>, rd_fheuint:FheUint<Vec<u8>, u32>) -> (FheUint<Vec<u8>, u32>, FheUint<Vec<u8>, u32>, FheUint<Vec<u8>, u32>) {
        let params = Parameters::<BackendImpl>::new(); // TODO: pass params, instead of hardcoding it
        let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 22); // TODO: Set correct scratch

        let mut rs1_address: Address<Vec<u8>> = Address::alloc_from_params(&params);
        rs1_address.set_from_fheuint(self.params.module(), &rs1_fheuint, &self.bdd_key_prepared, &params.ggsw_infos(), scratch.borrow());

        let mut rs2_address: Address<Vec<u8>> = Address::alloc_from_params(&params);
        rs2_address.set_from_fheuint(self.params.module(), &rs2_fheuint, &self.bdd_key_prepared, &params.ggsw_infos(), scratch.borrow());

        let mut rd_address: Address<Vec<u8>> = Address::alloc_from_params(&params);
        rd_address.set_from_fheuint(self.params.module(), &rd_fheuint, &self.bdd_key_prepared, &params.ggsw_infos(), scratch.borrow());

        let reg_rs1: FheUint<Vec<u8>, u32> = self.registers.read_to_fheuint(&rs1_address, &self.fhe_ram_keys_prepared, &self.bdd_key_prepared);
        let reg_rs2: FheUint<Vec<u8>, u32> = self.registers.read_to_fheuint(&rs2_address, &self.fhe_ram_keys_prepared, &self.bdd_key_prepared);
        let reg_rd: FheUint<Vec<u8>, u32> = self.registers.read_to_fheuint(&rd_address, &self.fhe_ram_keys_prepared, &self.bdd_key_prepared);

        (reg_rs1, reg_rs2, reg_rd)
    }

    pub fn cycle(&mut self) {

        let (imm_fheuint, rs1_fheuint, rs2_fheuint, rd_fheuint) = self.read_instruction_components();
        let (reg_rs1, reg_rs2, reg_rd) = self.read_registers(rs1_fheuint, rs2_fheuint, rd_fheuint);
      
    }
}

#[test]
pub fn test_interpreter_init() {
    // Initializing cryptographic parameters
    let params = CryptographicParameters::<BackendImpl>::new();

    let seed_xs: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);

    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.module().n().into());
    sk_lwe.fill_binary_block(8, &mut source_xs);

    let interpreter = Interpreter::new(&sk_lwe, &sk_glwe);

    assert_eq!(interpreter.params.basek(), params.basek());
    assert_eq!(interpreter.params.rank(), params.rank());
    assert_eq!(interpreter.params.k_evk_trace(), params.k_evk_trace());
    assert_eq!(interpreter.params.k_evk_ggsw_inv(), params.k_evk_ggsw_inv());
    assert_eq!(interpreter.params.dnum_ct(), params.dnum_ct());
    assert_eq!(interpreter.params.dnum_ggsw(), params.dnum_ggsw());
    assert_eq!(interpreter.params.k_evk_ggsw_inv(), params.k_evk_ggsw_inv());
    
}

#[test]
pub fn test_interpreter_init_pc() {
    // use poulpy_schemes::tfhe::bdd_arithmetic::{FheUintPreparedDebug, FheUintBlockDebugPrepare};

    let params = CryptographicParameters::<BackendImpl>::new();

    let seed_xs: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);

    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.module().n().into());
    sk_lwe.fill_binary_block(8, &mut source_xs);

    let mut interpreter = Interpreter::new(&sk_lwe, &sk_glwe);

    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BackendImpl> = GLWESecretPrepared::alloc_from_infos(params.module(), &params.glwe_ct_infos());
    sk_glwe_prepared.prepare(params.module(), &sk_glwe);
    interpreter.init_pc(&sk_glwe_prepared);

    // TODO: decrypt the prepared program counter
    // fheuint_debug can only be initialized from 

}

#[test]
pub fn test_interpreter_init_one_instruction() {
    use crate::Instruction;

    let params = CryptographicParameters::<BackendImpl>::new();

    let seed_xs: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    // sk_glwe.fill_ternary_prob(0.5, &mut source_xs);
    sk_glwe.fill_zero();

    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.module().n().into());
    // sk_lwe.fill_binary_block(8, &mut source_xs);
    sk_lwe.fill_zero();

    let mut interpreter = Interpreter::new(&sk_lwe, &sk_glwe);

    let instruction_u32 = 258455;
    let mut parser = InstructionsParser::new();
    parser.add(Instruction::new(instruction_u32));

    interpreter.init_instructions(&sk_glwe, &parser);
    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BackendImpl> = GLWESecretPrepared::alloc_from_infos(params.module(), &params.glwe_ct_infos());
    sk_glwe_prepared.prepare(params.module(), &sk_glwe);

    let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 22);

    let instruction = parser.get_raw(0);
    let correct_imm = instruction.get_immediate();
    let correct_rs1 = instruction.get_rs1() as u32;
    let correct_rs2 = instruction.get_rs2() as u32;
    let correct_rd = instruction.get_rd() as u32;

    interpreter.init_pc(&sk_glwe_prepared);
    let (imm_fheuint, rs1_fheuint, rs2_fheuint, rd_fheuint) = interpreter.read_instruction_components();

    let dec_rs1 = rs1_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    let dec_rs2 = rs2_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    let dec_rd = rd_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
    let dec_imm = imm_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());

    println!("{} {} {} {}", correct_imm, correct_rs1, correct_rs2, correct_rd);
    println!("{} {} {} {}", dec_imm, dec_rs1, dec_rs2, dec_rd);

    assert_eq!(correct_imm, dec_imm);
    assert_eq!(correct_rs1, dec_rs1);
    assert_eq!(correct_rs2, dec_rs2);
    assert_eq!(correct_rd, dec_rd);

}

#[test]
pub fn test_interpreter_init_many_instructions() {
    use crate::Instruction;

    let params = CryptographicParameters::<BackendImpl>::new();

    let seed_xs: [u8; 32] = [0u8; 32];

    let mut source_xs: Source = Source::new(seed_xs);

    // Generates a new secret-key along with the public evaluation keys.
    let mut sk_glwe: GLWESecret<Vec<u8>> = GLWESecret::alloc_from_infos(&params.glwe_ct_infos());
    sk_glwe.fill_ternary_prob(0.5, &mut source_xs);

    let mut sk_lwe: LWESecret<Vec<u8>> = LWESecret::alloc(params.module().n().into());
    sk_lwe.fill_binary_block(8, &mut source_xs);

    let mut interpreter = Interpreter::new(&sk_lwe, &sk_glwe);

    let instructions_u32 = vec![
        258455,
        33653139,
        512279,
        4286644499,
        66579,
        10507363,
        3221229683,
        8388847,
        3221229683,
        791,
        8585319,
        259383,
    ];

    let mut parser = InstructionsParser::new();
    for inst in instructions_u32.clone() {
        parser.add(Instruction::new(inst));
    }

    interpreter.init_instructions(&sk_glwe, &parser);
    let mut sk_glwe_prepared: GLWESecretPrepared<Vec<u8>, BackendImpl> = GLWESecretPrepared::alloc_from_infos(params.module(), &params.glwe_ct_infos());
    sk_glwe_prepared.prepare(params.module(), &sk_glwe);

    let mut scratch: ScratchOwned<BackendImpl> = ScratchOwned::alloc(1 << 22);

    for idx in 0..instructions_u32.len() {

        let instruction = parser.get_raw(idx);
        let correct_imm = instruction.get_immediate();
        let correct_rs1 = instruction.get_rs1() as u32;
        let correct_rs2 = instruction.get_rs2() as u32;
        let correct_rd = instruction.get_rd() as u32;

        interpreter.set_pc_to(idx as u32, &sk_glwe_prepared);
        let (imm_fheuint, rs1_fheuint, rs2_fheuint, rd_fheuint) = interpreter.read_instruction_components();

        let dec_rs1 = rs1_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
        let dec_rs2 = rs2_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
        let dec_rd = rd_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());
        let dec_imm = imm_fheuint.decrypt(params.module(), &sk_glwe_prepared, scratch.borrow());

        assert_eq!(correct_imm, dec_imm);
        assert_eq!(correct_rs1, dec_rs1);
        assert_eq!(correct_rs2, dec_rs2);
        assert_eq!(correct_rd, dec_rd);
    }

}