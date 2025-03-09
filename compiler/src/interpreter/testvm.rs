use super::{BootMemory, InputInfo, OutputInfo};

struct Memory {
    data: Vec<u8>,
    offset: usize,
    _is_write: bool,
    size: usize,
}

impl Memory {
    fn new(offset: usize, size: usize, is_write: bool) -> Self {
        Memory {
            data: vec![0u8; size],
            offset,
            _is_write: is_write,
            size,
        }
    }

    fn write_byte(&mut self, addr: usize, value: u8) {
        self.data[(addr - self.offset) % self.size] = value;
    }

    fn write_half(&mut self, addr: usize, value: u16) {
        for i in 0..2 {
            let vbyte = ((value >> (i * 8)) & ((1 << 8) - 1)) as u8;
            self.data[(addr + i - self.offset) % self.size] = vbyte;
        }
    }

    fn write_word(&mut self, addr: usize, value: u32) {
        for i in 0..4 {
            let vbyte = ((value >> (i * 8)) & ((1 << 8) - 1)) as u8;
            self.data[((addr + i) - self.offset) % self.size] = vbyte;
        }
    }

    fn load_memory(&mut self, start_at: usize, values: &[u8]) {
        self.data[(start_at - self.offset)..((start_at - self.offset) + values.len())]
            .clone_from_slice(values);
    }

    fn read_byte(&self, addr: usize) -> u8 {
        self.data[(addr - self.offset) % self.size]
    }

    fn read_half(&self, addr: usize) -> u16 {
        let mut out = 0u16;
        for i in 0..2 {
            out += (self.data[((addr - self.offset) + i) % self.size] as u16) << (i * 8);
        }
        out
    }

    fn read_word(&self, addr: usize) -> u32 {
        let mut out = 0u32;
        for i in 0..4 {
            out += (self.data[((addr - self.offset) + i) % self.size] as u32) << (i * 8);
        }

        out
    }
}

#[derive(Debug)]
enum Inst {
    // Integer register-register instructions
    ADD(u32, u32, u32),
    SUB(u32, u32, u32),
    SLL(u32, u32, u32),
    SLT(u32, u32, u32),
    SLTU(u32, u32, u32),
    XOR(u32, u32, u32),
    SRL(u32, u32, u32),
    SRA(u32, u32, u32),
    OR(u32, u32, u32),
    AND(u32, u32, u32),

    // Integer register-immediate instructions
    ADDI(u32, u32, u32),
    SLTI(u32, u32, u32),
    SLTIU(u32, u32, u32),
    XORI(u32, u32, u32),
    ORI(u32, u32, u32),
    ANDI(u32, u32, u32),
    SLLI(u32, u32, u32),
    SRLI(u32, u32, u32),
    SRAI(u32, u32, u32),

    LUI(u32, u32),
    AUIPC(u32, u32),

    // Unconditional jumps
    JAL(u32, u32),
    JALR(u32, u32, u32),

    // Branch/conditional jumps
    BEQ(u32, u32, u32),
    BNE(u32, u32, u32),
    BLT(u32, u32, u32),
    BGE(u32, u32, u32),
    BLTU(u32, u32, u32),
    BGEU(u32, u32, u32),

    // Load instructions
    LB(u32, u32, u32),
    LH(u32, u32, u32),
    LW(u32, u32, u32),
    LBU(u32, u32, u32),
    LHU(u32, u32, u32),

    // Store instructions
    SB(u32, u32, u32),
    SH(u32, u32, u32),
    SW(u32, u32, u32),

    // ECALL
    // ECALL,

    // UNIMP
    UNIMP,
}

#[derive(PartialEq, Debug)]
enum VMState {
    EXEC,
    HALT,
}

pub struct TestVM {
    /// VM registers
    registers: [u32; 32],
    /// program instructions
    rom: Memory,
    /// RAM
    ram: Memory,
    /// program counter
    pc: u32,
    /// state
    state: VMState,
    /// Input info
    input_info: InputInfo,
    /// Output info
    output_info: OutputInfo,
}

impl TestVM {
    pub(super) fn init(
        boot_rom: &BootMemory,
        boot_ram: &BootMemory,
        input_info: &InputInfo,
        output_info: &OutputInfo,
    ) -> Self {
        let mut rom = Memory::new(boot_rom.offset, boot_rom.size, false);
        rom.load_memory(boot_rom.offset, &boot_rom.data);

        let mut ram = Memory::new(boot_ram.offset, boot_ram.size, true);
        ram.load_memory(boot_ram.offset, &boot_ram.data);

        Self {
            registers: [0u32; 32],
            ram,
            rom,
            pc: 0,
            state: VMState::EXEC,
            input_info: input_info.clone(),
            output_info: output_info.clone(),
        }
    }

    pub fn is_exec(&self) -> bool {
        self.state == VMState::EXEC
    }

    fn decode_inst(&self, inst: u32) -> Inst {
        let opcode = inst & ((1 << 7) - 1);

        if opcode == 0b0010011 {
            // Integer register-immediate instructions

            let sign = (inst >> 31) & 1;
            let mut imm = (inst >> 20) & ((1 << 12) - 1);
            for i in 0..20 {
                imm += sign << (12 + i);
            }

            let rd = (inst >> 7) & ((1 << 5) - 1);
            let rs1 = (inst >> 15) & ((1 << 5) - 1);
            let funct3 = (inst >> 12) & ((1 << 3) - 1);

            if funct3 == 0b000 {
                return Inst::ADDI(rs1, rd, imm);
            } else if funct3 == 0b010 {
                return Inst::SLTI(rs1, rd, imm);
            } else if funct3 == 0b011 {
                return Inst::SLTIU(rs1, rd, imm);
            } else if funct3 == 0b100 {
                return Inst::XORI(rs1, rd, imm);
            } else if funct3 == 0b0110 {
                return Inst::ORI(rs1, rd, imm);
            } else if funct3 == 0b0111 {
                return Inst::ANDI(rs1, rd, imm);
            }

            // Constant shifts
            let shift = (inst >> 20) & ((1 << 5) - 1);
            if funct3 == 0b001 {
                return Inst::SLLI(rs1, rd, shift);
            } else if funct3 == 0b101 {
                if (inst >> 30) & 1 == 0 {
                    return Inst::SRLI(rs1, rd, shift);
                } else {
                    return Inst::SRAI(rs1, rd, shift);
                }
            }
        } else if opcode == 0b0110111 {
            let imm = (inst >> 12) & ((1 << 20) - 1);
            let rd = (inst >> 7) & ((1 << 5) - 1);
            return Inst::LUI(rd, imm);
        } else if opcode == 0b0010111 {
            let imm = (inst >> 12) & ((1 << 20) - 1);
            let rd = (inst >> 7) & ((1 << 5) - 1);
            return Inst::AUIPC(rd, imm);
        } else if opcode == 0b0110011 {
            let rd = (inst >> 7) & ((1 << 5) - 1);
            let rs1 = (inst >> 15) & ((1 << 5) - 1);
            let rs2 = (inst >> 20) & ((1 << 5) - 1);

            let func3 = (inst >> 12) & ((1 << 3) - 1);

            if func3 == 0b000 {
                if (inst >> 30) & 1 == 0 {
                    return Inst::ADD(rs1, rs2, rd);
                } else {
                    return Inst::SUB(rs1, rs2, rd);
                }
            } else if func3 == 0b001 {
                return Inst::SLL(rs1, rs2, rd);
            } else if func3 == 0b010 {
                return Inst::SLT(rs1, rs2, rd);
            } else if func3 == 0b011 {
                return Inst::SLTU(rs1, rs2, rd);
            } else if func3 == 0b100 {
                return Inst::XOR(rs1, rs2, rd);
            } else if func3 == 0b110 {
                return Inst::OR(rs1, rs2, rd);
            } else if func3 == 0b111 {
                return Inst::AND(rs1, rs2, rd);
            } else if func3 == 0b101 {
                if (inst >> 30) & 1 == 0 {
                    return Inst::SRL(rs1, rs2, rd);
                } else {
                    return Inst::SRA(rs1, rs2, rd);
                }
            }
        } else if opcode == 0b1101111 {
            let rd = (inst >> 7) & ((1 << 5) - 1);
            let sign = (inst >> 31) & 1;
            let imm = ((inst >> 21) & ((1 << 10) - 1))
                + (((inst >> 20) & 1) << 10)
                + (((inst >> 12) & ((1 << 8) - 1)) << 11)
                + (sign << 19);
            let mut imm = imm << 1;
            for i in 0..11 {
                imm += sign << (21 + i);
            }
            return Inst::JAL(rd, imm);
        } else if opcode == 0b1100111 && (((inst >> 12) & ((1 << 3) - 1)) == 0) {
            let rd = (inst >> 7) & ((1 << 5) - 1);
            let rs1 = (inst >> 15) & ((1 << 5) - 1);

            let sign = (inst >> 31) & 1;
            let mut imm = (inst >> 20) & ((1 << 12) - 1);
            for i in 0..20 {
                imm += sign << (12 + i);
            }
            return Inst::JALR(rs1, rd, imm);
        } else if opcode == 0b1100011 {
            let func3 = (inst >> 12) & ((1 << 3) - 1);
            let rs1 = (inst >> 15) & ((1 << 5) - 1);
            let rs2 = (inst >> 20) & ((1 << 5) - 1);

            let sign = (inst >> 31) & 1;
            let imm = ((inst >> 8) & ((1 << 4) - 1))
                + (((inst >> 25) & ((1 << 6) - 1)) << 4)
                + (((inst >> 7) & 1) << 10)
                + (sign << 11);
            let mut imm = imm << 1;
            for i in 0..19 {
                imm += sign << (13 + i);
            }

            if func3 == 0b000 {
                return Inst::BEQ(rs1, rs2, imm);
            } else if func3 == 0b001 {
                return Inst::BNE(rs1, rs2, imm);
            } else if func3 == 0b100 {
                return Inst::BLT(rs1, rs2, imm);
            } else if func3 == 0b101 {
                return Inst::BGE(rs1, rs2, imm);
            } else if func3 == 0b110 {
                return Inst::BLTU(rs1, rs2, imm);
            } else if func3 == 0b111 {
                return Inst::BGEU(rs1, rs2, imm);
            }
        } else if opcode == 0b0000011 {
            let func3 = (inst >> 12) & ((1 << 3) - 1);
            let rs1 = (inst >> 15) & ((1 << 5) - 1);
            let rd = (inst >> 7) & ((1 << 5) - 1);

            let sign = (inst >> 31) & 1;
            let mut imm = (inst >> 20) & ((1 << 12) - 1);
            for i in 0..20 {
                imm += sign << (12 + i);
            }

            if func3 == 0b000 {
                return Inst::LB(rs1, rd, imm);
            } else if func3 == 0b001 {
                return Inst::LH(rs1, rd, imm);
            } else if func3 == 0b010 {
                return Inst::LW(rs1, rd, imm);
            } else if func3 == 0b100 {
                return Inst::LBU(rs1, rd, imm);
            } else if func3 == 0b101 {
                return Inst::LHU(rs1, rd, imm);
            }
        } else if opcode == 0b0100011 {
            let func3 = (inst >> 12) & ((1 << 3) - 1);
            let rs1 = (inst >> 15) & ((1 << 5) - 1);
            let rs2 = (inst >> 20) & ((1 << 5) - 1);

            let sign = (inst >> 31) & 1;
            let mut imm = ((inst >> 7) & ((1 << 5) - 1)) + (((inst >> 25) & ((1 << 7) - 1)) << 5);
            for i in 0..20 {
                imm += sign << (12 + i);
            }

            if func3 == 0b000 {
                return Inst::SB(rs1, rs2, imm);
            } else if func3 == 0b001 {
                return Inst::SH(rs1, rs2, imm);
            } else if func3 == 0b010 {
                return Inst::SW(rs1, rs2, imm);
            }
        } else if inst == 3221229683 {
            return Inst::UNIMP;
        }

        // else if opcode == 0b1110011 && ((inst >> 7) == 0) {
        //     return Inst::ECALL;
        // }

        panic!("Instruction={} cannot be decoded", inst);
    }

    fn register(&self, index: u32) -> u32 {
        self.registers[index as usize]
    }

    fn register_mut(&mut self, index: u32) -> &mut u32 {
        &mut self.registers[index as usize]
    }

    pub fn run(&mut self) {
        // Note: Any operation that writes to register x0 can be discarded. For ex, ADDI x0 x0 0
        // is set as NOP instruction, most HINTs use integer computation with rd=x0.

        if self.state != VMState::EXEC {
            panic!("VM State = {:?}", self.state);
        }

        let inst_u32 = self.rom.read_word(self.pc as usize);
        // println!("Inst raw = {:?} at pc={}", inst_u32, self.pc);
        let inst = self.decode_inst(inst_u32);
        // println!("Inst = {:?} at pc={}", inst, self.pc);
        match inst {
            Inst::ADDI(rs1, rd, imm) => {
                let rs1v = self.register(rs1);
                let rdv = self.register_mut(rd);
                *rdv = rs1v.wrapping_add(imm);

                self.pc += 4;
            }
            Inst::SLTI(rs1, rd, imm) => {
                let rs1v = self.register(rs1);
                let rdv = self.register_mut(rd);
                *rdv = ((rs1v as i32) < (imm as i32)) as u32;

                self.pc += 4;
            }
            Inst::SLTIU(rs1, rd, imm) => {
                let rs1v = self.register(rs1);
                let rdv = self.register_mut(rd);
                *rdv = (rs1v < imm) as u32;

                self.pc += 4;
            }
            Inst::XORI(rs1, rd, imm) => {
                let rs1v = self.register(rs1);
                let rdv = self.register_mut(rd);
                *rdv = rs1v ^ imm;

                self.pc += 4;
            }
            Inst::ORI(rs1, rd, imm) => {
                let rs1v = self.register(rs1);
                let rdv = self.register_mut(rd);
                *rdv = rs1v | imm;

                self.pc += 4;
            }
            Inst::ANDI(rs1, rd, imm) => {
                let rs1v = self.register(rs1);
                let rdv = self.register_mut(rd);
                *rdv = rs1v & imm;

                self.pc += 4;
            }
            Inst::SLLI(rs1, rd, shift) => {
                let rs1v = self.register(rs1);
                let rdv = self.register_mut(rd);
                *rdv = rs1v << shift;

                self.pc += 4;
            }
            Inst::SRLI(rs1, rd, shift) => {
                let rs1v = self.register(rs1);
                let rdv = self.register_mut(rd);
                *rdv = rs1v >> shift;

                self.pc += 4;
            }
            Inst::SRAI(rs1, rd, shift) => {
                let rs1v = self.register(rs1);
                let rdv = self.register_mut(rd);
                *rdv = ((rs1v as i32) >> shift) as u32;

                self.pc += 4;
            }
            Inst::LUI(rd, imm) => {
                *self.register_mut(rd) = imm << 12;

                self.pc += 4;
            }
            Inst::AUIPC(rd, imm) => {
                let offset = imm << 12;
                *self.register_mut(rd) = offset.wrapping_add(self.pc);

                self.pc += 4;
            }
            Inst::ADD(rs1, rs2, rd) => {
                *self.register_mut(rd) = self.register(rs1).wrapping_add(self.register(rs2));

                self.pc += 4;
            }
            Inst::SUB(rs1, rs2, rd) => {
                *self.register_mut(rd) = self.register(rs1).wrapping_sub(self.register(rs2));

                self.pc += 4;
            }
            Inst::SLL(rs1, rs2, rd) => {
                *self.register_mut(rd) =
                    self.register(rs1) << (self.register(rs2) & ((1 << 6) - 1));

                self.pc += 4;
            }
            Inst::SLT(rs1, rs2, rd) => {
                *self.register_mut(rd) =
                    ((self.register(rs1) as i32) < (self.register(rs2) as i32)) as u32;

                self.pc += 4;
            }
            Inst::SLTU(rs1, rs2, rd) => {
                *self.register_mut(rd) = (self.register(rs1) < self.register(rs2)) as u32;

                self.pc += 4;
            }
            Inst::XOR(rs1, rs2, rd) => {
                *self.register_mut(rd) = self.register(rs1) ^ self.register(rs2);

                self.pc += 4;
            }
            Inst::OR(rs1, rs2, rd) => {
                *self.register_mut(rd) = self.register(rs1) | self.register(rs2);

                self.pc += 4;
            }
            Inst::AND(rs1, rs2, rd) => {
                *self.register_mut(rd) = self.register(rs1) & self.register(rs2);

                self.pc += 4;
            }
            Inst::SRL(rs1, rs2, rd) => {
                *self.register_mut(rd) =
                    self.register(rs1) >> (self.register(rs2) & ((1 << 6) - 1));

                self.pc += 4;
            }
            Inst::SRA(rs1, rs2, rd) => {
                *self.register_mut(rd) =
                    ((self.register(rs1) as i32) >> (self.register(rs2) & ((1 << 6) - 1))) as u32;

                self.pc += 4;
            }
            Inst::JAL(rd, offset) => {
                let jump_target = self.pc.wrapping_add(offset);
                assert!(
                    jump_target % 4 == 0,
                    "Jump target={jump_target} is misaligned"
                );
                *self.register_mut(rd) = self.pc.wrapping_add(4);
                self.pc = jump_target;
            }
            Inst::JALR(rs1, rd, imm) => {
                let jump_target = ((self.register(rs1).wrapping_add(imm)) >> 1) << 1;
                // println!("JALR: jump_target = {jump_target}");
                assert!(
                    jump_target % 4 == 0,
                    "Jump target={jump_target} is misaligned"
                );
                *self.register_mut(rd) = self.pc.wrapping_add(4);
                self.pc = jump_target;
            }
            Inst::BEQ(rs1, rs2, imm) => {
                let jump_target = imm.wrapping_add(self.pc);
                if self.register(rs1) == self.register(rs2) {
                    // jump target is expected to be 4-byte aligned iff branch condition evaluates to true
                    assert!(
                        jump_target % 4 == 0,
                        "Jump target={jump_target} is misaligned"
                    );
                    self.pc = jump_target;
                } else {
                    self.pc += 4;
                }
            }
            Inst::BNE(rs1, rs2, imm) => {
                let jump_target = imm.wrapping_add(self.pc);
                if self.register(rs1) != self.register(rs2) {
                    assert!(
                        jump_target % 4 == 0,
                        "Jump target={jump_target} is misaligned"
                    );
                    self.pc = jump_target;
                } else {
                    self.pc += 4;
                }
            }
            Inst::BLT(rs1, rs2, imm) => {
                let jump_target = imm.wrapping_add(self.pc);
                if (self.register(rs1) as i32) < (self.register(rs2) as i32) {
                    assert!(
                        jump_target % 4 == 0,
                        "Jump target={jump_target} is misaligned"
                    );
                    self.pc = jump_target;
                } else {
                    self.pc += 4;
                }
            }
            Inst::BLTU(rs1, rs2, imm) => {
                let jump_target = imm.wrapping_add(self.pc);
                if self.register(rs1) < self.register(rs2) {
                    assert!(
                        jump_target % 4 == 0,
                        "Jump target={jump_target} is misaligned"
                    );
                    self.pc = jump_target;
                } else {
                    self.pc += 4;
                }
            }
            Inst::BGE(rs1, rs2, imm) => {
                let jump_target = imm.wrapping_add(self.pc);
                if (self.register(rs1) as i32) >= (self.register(rs2) as i32) {
                    assert!(
                        jump_target % 4 == 0,
                        "Jump target={jump_target} is misaligned"
                    );
                    self.pc = jump_target;
                } else {
                    self.pc += 4;
                }
            }
            Inst::BGEU(rs1, rs2, imm) => {
                let jump_target = imm.wrapping_add(self.pc);
                if self.register(rs1) >= self.register(rs2) {
                    assert!(
                        jump_target % 4 == 0,
                        "Jump target={jump_target} is misaligned"
                    );
                    self.pc = jump_target;
                } else {
                    self.pc += 4;
                }
            }
            Inst::LB(rs1, rd, offset) => {
                let addr = self.register(rs1).wrapping_add(offset);

                let mut v = self.ram.read_byte(addr as usize) as u32;
                let v_sign = (v >> 7) & 1;
                for i in 0..24 {
                    v += v_sign << (i + 8);
                }

                *self.register_mut(rd) = v;

                self.pc += 4;
            }
            Inst::LH(rs1, rd, offset) => {
                let addr = self.register(rs1).wrapping_add(offset);

                let mut v = self.ram.read_half(addr as usize) as u32;
                let v_sign = (v >> 15) & 1;
                for i in 0..16 {
                    v += v_sign << (i + 16);
                }

                *self.register_mut(rd) = v;

                self.pc += 4;
            }
            Inst::LW(rs1, rd, offset) => {
                let addr = self.register(rs1).wrapping_add(offset);

                *self.register_mut(rd) = self.ram.read_word(addr as usize);

                self.pc += 4;
            }
            Inst::LBU(rs1, rd, offset) => {
                let addr = self.register(rs1).wrapping_add(offset);

                *self.register_mut(rd) = self.ram.read_byte(addr as usize) as u32;

                self.pc += 4;
            }
            Inst::LHU(rs1, rd, offset) => {
                let addr = self.register(rs1).wrapping_add(offset);

                *self.register_mut(rd) = self.ram.read_half(addr as usize) as u32;

                self.pc += 4;
            }
            Inst::SB(rs1, rs2, offset) => {
                let addr = self.register(rs1).wrapping_add(offset);
                self.ram
                    .write_byte(addr as usize, (self.register(rs2) & ((1 << 9) - 1)) as u8);

                self.pc += 4;
            }
            Inst::SH(rs1, rs2, offset) => {
                let addr = self.register(rs1).wrapping_add(offset);
                self.ram
                    .write_half(addr as usize, (self.register(rs2) & ((1 << 17) - 1)) as u16);

                self.pc += 4;
            }
            Inst::SW(rs1, rs2, offset) => {
                let addr = self.register(rs1).wrapping_add(offset);
                self.ram.write_word(addr as usize, self.register(rs2));

                self.pc += 4;
            }
            // Inst::ECALL => {
            //     // a0 stores v_addrs, a1 stores v_len
            //     self.output_info = Some(VMOutputInfo {
            //         addr: self.register(10),
            //         len: self.register(11),
            //     });

            //     self.pc += 4;
            // }
            Inst::UNIMP => {
                // halt vm
                self.state = VMState::HALT
            }
        }

        *self.register_mut(0) = 0;
    }

    pub fn read_input_tape(&mut self, tape: &[u8]) {
        assert!(
            tape.len() == self.input_info.size as usize,
            "Input tape exceeds .inpdata size"
        );

        self.ram.load_memory(self.input_info.start_addr, tape);
    }

    pub fn output_tape(&self) -> Vec<u8> {
        let mut output = Vec::with_capacity((self.output_info.size) as usize);
        for i in 0..self.output_info.size {
            output.push(
                self.ram
                    .read_byte((self.output_info.start_addr + i) as usize),
            );
        }
        output
    }
}
