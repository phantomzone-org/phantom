use std::collections::HashMap;

use crate::{InstructionsParser, PC_UPDATE_OP_LIST, RAM_UPDATE_OP_LIST, RD_UPDATE};

pub(crate) struct InterpreterDebug {
    pub(crate) pc: u32,
    pub(crate) imm_rom: Vec<u32>,
    pub(crate) rs1_rom: Vec<u32>,
    pub(crate) rs2_rom: Vec<u32>,
    pub(crate) rd_rom: Vec<u32>,
    pub(crate) rdu_rom: Vec<u32>,
    pub(crate) mu_rom: Vec<u32>,
    pub(crate) pcu_rom: Vec<u32>,
    pub(crate) registers: [u32; 32],
    pub(crate) ram: Vec<u32>,
    pub(crate) ram_addr: u32,
    pub(crate) imm: u32,
    pub(crate) rs1_val: u32,
    pub(crate) rs2_val: u32,
    pub(crate) ram_val: u32,
    pub(crate) rd_val: u32,
    pub(crate) rs1_addr: u32,
    pub(crate) rs2_addr: u32,
    pub(crate) rd_addr: u32,
    pub(crate) rdu: u32,
    pub(crate) pcu: u32,
    pub(crate) mu: u32,
}

impl InterpreterDebug {
    pub fn new(rom_size: usize, ram_size: usize) -> Self {
        Self {
            pc: 0,
            imm_rom: vec![0u32; rom_size],
            rs1_rom: vec![0u32; rom_size],
            rs2_rom: vec![0u32; rom_size],
            rd_rom: vec![0u32; rom_size],
            rdu_rom: vec![0u32; rom_size],
            mu_rom: vec![0u32; rom_size],
            pcu_rom: vec![0u32; rom_size],
            registers: [0u32; 32],
            ram: vec![0u32; ram_size],
            ram_addr: 0,
            imm: 0,
            rs1_val: 0,
            rs2_val: 0,
            ram_val: 0,
            rd_val: 0,
            rs1_addr: 0,
            rs2_addr: 0,
            rd_addr: 0,
            rdu: 0,
            pcu: 0,
            mu: 0,
        }
    }

    pub fn set_instructions(&mut self, instructions: &InstructionsParser) {
        assert_eq!(self.imm_rom.len(), instructions.instructions.len());
        for i in 0..instructions.instructions.len() {
            self.imm_rom[i] = instructions.get_raw(i).get_immediate() as u32;
            let (rs1, rs2, rd) = instructions.get_raw(i).get_registers();
            self.rs1_rom[i] = rs1 as u32;
            self.rs2_rom[i] = rs2 as u32;
            self.rd_rom[i] = rd as u32;
            let (rdu, mu, pcu) = instructions.get_raw(i).get_opid();
            self.rdu_rom[i] = rdu as u32;
            self.mu_rom[i] = mu as u32;
            self.pcu_rom[i] = pcu as u32;
        }
    }

    pub fn set_ram(&mut self, ram: &[u32]) {
        assert_eq!(self.ram.len(), ram.len());
        self.ram.copy_from_slice(ram);
    }

    pub fn read_instructions(&mut self) {
        let pc: usize = (self.pc & 0xFFFF_FFFE) as usize;
        self.imm = self.imm_rom[pc];
        self.rs1_addr = self.rs1_rom[pc];
        self.rs2_addr = self.rs2_rom[pc];
        self.rd_addr = self.rd_rom[pc];
        self.rdu = self.rdu_rom[pc];
        self.pcu = self.pcu_rom[pc];
        self.mu = self.mu_rom[pc];
    }

    pub fn read_registers(&mut self) {
        self.rs1_val = self.registers[self.rs1_addr as usize];
        self.rs2_val = self.registers[self.rs2_addr as usize];
    }

    pub fn read_ram(&mut self) {
        self.ram_addr = self.imm.wrapping_add(self.rs2_val).wrapping_sub(1 << 18);
        self.ram_val = self.ram[(self.ram_addr & 0xFFFF_FFFE) as usize];
    }

    pub fn update_registers(&mut self, ops: &[RD_UPDATE]) {
        let mut rd_map: HashMap<u32, u32> = HashMap::new();

        let imm: u32 = self.imm;
        let rs1: u32 = self.rs1_val;
        let rs2: u32 = self.rs2_val;
        let ram: u32 = self.ram_val;
        let pc: u32 = self.pc;

        for op in ops {
            rd_map.insert(op.id(), op.eval_plain(imm, rs1, rs2, pc, ram));
        }

        self.rd_val = *rd_map.get(&self.rdu).unwrap();

        self.registers[self.rd_addr as usize] = self.rd_val;
        self.registers[0] = 0;
    }

    pub fn update_ram(&mut self) {
        let rs2: u32 = self.rs2_val;
        let ram: u32 = self.ram_val;
        let offset: u32 = self.ram_addr & 0x3;
        let mut ram_map: HashMap<u32, u32> = HashMap::new();
        for op in RAM_UPDATE_OP_LIST {
            ram_map.insert(op.id(), op.eval_plain(rs2, ram, offset));
        }

        self.ram_val = *ram_map.get(&self.mu).unwrap();

        self.ram[(self.ram_addr & 0xFFFF_FFFE) as usize] = self.ram_val;
    }

    pub fn update_pc(&mut self) {
        let imm: u32 = self.imm;
        let rs1: u32 = self.rs1_val;
        let rs2: u32 = self.rs2_val;
        let pc: u32 = self.pc;

        let mut pc_map: HashMap<u32, u32> = HashMap::new();
        for op in PC_UPDATE_OP_LIST {
            pc_map.insert(op.id(), op.eval_plain(imm, rs1, rs2, pc));
        }

        self.pc = *pc_map.get(&self.pcu).unwrap()
    }
}
