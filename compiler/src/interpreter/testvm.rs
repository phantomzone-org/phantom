use super::{BootMemory, InputInfo, OutputInfo};
use std::fmt;
use utils::{extract_bits, sign_extend};

macro_rules! verbose_println {
    ($($arg:tt)*) => {
        #[cfg(feature = "verbose")]
        println!($($arg)*);
    };
}

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
        // println!("write byte {value} at address={}", addr);
        self.data[(addr - self.offset) % self.size] = value;
    }

    fn write_half(&mut self, addr: usize, value: u16) {
        // println!("write half {value} at address={}", addr);
        for i in 0..2 {
            let vbyte = ((value >> (i * 8)) & ((1 << 8) - 1)) as u8;
            self.data[(addr + i - self.offset) % self.size] = vbyte;
        }
    }

    fn write_word(&mut self, addr: usize, value: u32) {
        // println!("write word {value} at address={}", addr);
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
    ADD(RegisterIndex, RegisterIndex, RegisterIndex),
    SUB(RegisterIndex, RegisterIndex, RegisterIndex),
    SLL(RegisterIndex, RegisterIndex, RegisterIndex),
    SLT(RegisterIndex, RegisterIndex, RegisterIndex),
    SLTU(RegisterIndex, RegisterIndex, RegisterIndex),
    XOR(RegisterIndex, RegisterIndex, RegisterIndex),
    SRL(RegisterIndex, RegisterIndex, RegisterIndex),
    SRA(RegisterIndex, RegisterIndex, RegisterIndex),
    OR(RegisterIndex, RegisterIndex, RegisterIndex),
    AND(RegisterIndex, RegisterIndex, RegisterIndex),

    // Integer register-immediate instructions
    ADDI(RegisterIndex, RegisterIndex, u32),
    SLTI(RegisterIndex, RegisterIndex, u32),
    SLTIU(RegisterIndex, RegisterIndex, u32),
    XORI(RegisterIndex, RegisterIndex, u32),
    ORI(RegisterIndex, RegisterIndex, u32),
    ANDI(RegisterIndex, RegisterIndex, u32),
    SLLI(RegisterIndex, RegisterIndex, u32),
    SRLI(RegisterIndex, RegisterIndex, u32),
    SRAI(RegisterIndex, RegisterIndex, u32),

    // M extension
    MUL(RegisterIndex, RegisterIndex, RegisterIndex),
    MULH(RegisterIndex, RegisterIndex, RegisterIndex),
    MULHSU(RegisterIndex, RegisterIndex, RegisterIndex),
    MULHU(RegisterIndex, RegisterIndex, RegisterIndex),
    DIV(RegisterIndex, RegisterIndex, RegisterIndex),
    DIVU(RegisterIndex, RegisterIndex, RegisterIndex),
    REM(RegisterIndex, RegisterIndex, RegisterIndex),
    REMU(RegisterIndex, RegisterIndex, RegisterIndex),

    LUI(RegisterIndex, u32),
    AUIPC(RegisterIndex, u32),

    // Unconditional jumps
    JAL(RegisterIndex, u32),
    JALR(RegisterIndex, RegisterIndex, u32),

    // Branch/conditional jumps
    BEQ(RegisterIndex, RegisterIndex, u32),
    BNE(RegisterIndex, RegisterIndex, u32),
    BLT(RegisterIndex, RegisterIndex, u32),
    BGE(RegisterIndex, RegisterIndex, u32),
    BLTU(RegisterIndex, RegisterIndex, u32),
    BGEU(RegisterIndex, RegisterIndex, u32),

    // Load instructions
    LB(RegisterIndex, RegisterIndex, u32),
    LH(RegisterIndex, RegisterIndex, u32),
    LW(RegisterIndex, RegisterIndex, u32),
    LBU(RegisterIndex, RegisterIndex, u32),
    LHU(RegisterIndex, RegisterIndex, u32),

    // Store instructions
    SB(RegisterIndex, RegisterIndex, u32),
    SH(RegisterIndex, RegisterIndex, u32),
    SW(RegisterIndex, RegisterIndex, u32),
    // ECALL
    // ECALL,

    // UNIMP
    // UNIMP,
}

#[derive(PartialEq, Debug)]
enum VMState {
    EXEC,
    HALT,
}

#[derive(PartialEq, Debug, Clone, Copy)]
struct RegisterIndex(u32);

impl RegisterIndex {
    fn from(v: u32) -> Self {
        assert!(v < 32);
        RegisterIndex(v)
    }
}

impl fmt::Display for RegisterIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "r{}", self.0)
    }
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
        let opcode = extract_bits(inst, 7);

        if opcode == 0b0010011 {
            // Integer register-immediate instructions

            let mut imm = extract_bits(inst >> 20, 12);
            imm = sign_extend(imm, 12);

            let rd = RegisterIndex::from(extract_bits(inst >> 7, 5));
            let rs1 = RegisterIndex::from(extract_bits(inst >> 15, 5));
            let funct3 = extract_bits(inst >> 12, 3);

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
            let shift = extract_bits(inst >> 20, 5);
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
            let imm = extract_bits(inst >> 12, 20) << 12;
            let rd = RegisterIndex::from(extract_bits(inst >> 7, 5));
            return Inst::LUI(rd, imm);
        } else if opcode == 0b0010111 {
            let imm = extract_bits(inst >> 12, 20) << 12;
            let rd = RegisterIndex::from(extract_bits(inst >> 7, 5));
            return Inst::AUIPC(rd, imm);
        } else if opcode == 0b0110011 {
            let rd = RegisterIndex::from(extract_bits(inst >> 7, 5));
            let rs1 = RegisterIndex::from(extract_bits(inst >> 15, 5));
            let rs2 = RegisterIndex::from(extract_bits(inst >> 20, 5));

            let func3 = extract_bits(inst >> 12, 3);

            if inst >> 25 & 1 == 0 {
                // Integer register register
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
            } else {
                // M extension
                if func3 == 0b000 {
                    return Inst::MUL(rs1, rs2, rd);
                } else if func3 == 0b001 {
                    return Inst::MULH(rs1, rs2, rd);
                } else if func3 == 0b010 {
                    return Inst::MULHSU(rs1, rs2, rd);
                } else if func3 == 0b011 {
                    return Inst::MULHU(rs1, rs2, rd);
                } else if func3 == 0b100 {
                    return Inst::DIV(rs1, rs2, rd);
                } else if func3 == 0b101 {
                    return Inst::DIVU(rs1, rs2, rd);
                } else if func3 == 0b110 {
                    return Inst::REM(rs1, rs2, rd);
                } else if func3 == 0b111 {
                    return Inst::REMU(rs1, rs2, rd);
                }
            }
        } else if opcode == 0b1101111 {
            let rd = RegisterIndex::from(extract_bits(inst >> 7, 5));

            let mut imm = (extract_bits(inst >> 21, 10) << 1)
                + (extract_bits(inst >> 20, 1) << 11)
                + (extract_bits(inst >> 12, 8) << 12)
                + (extract_bits(inst >> 31, 1) << 20);
            imm = sign_extend(imm, 21);

            return Inst::JAL(rd, imm);
        } else if opcode == 0b1100111 && (extract_bits(inst >> 12, 3) == 0) {
            let rd = RegisterIndex::from(extract_bits(inst >> 7, 5));
            let rs1 = RegisterIndex::from(extract_bits(inst >> 15, 5));

            let mut imm = extract_bits(inst >> 20, 12);
            imm = sign_extend(imm, 12);

            return Inst::JALR(rs1, rd, imm);
        } else if opcode == 0b1100011 {
            let func3 = extract_bits(inst >> 12, 3);
            let rs1 = RegisterIndex::from(extract_bits(inst >> 15, 5));
            let rs2 = RegisterIndex::from(extract_bits(inst >> 20, 5));

            let mut imm = (extract_bits(inst >> 8, 4) << 1)
                + (extract_bits(inst >> 25, 6) << 5)
                + (extract_bits(inst >> 7, 1) << 11)
                + (extract_bits(inst >> 31, 1) << 12);
            imm = sign_extend(imm, 13);

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
            let func3 = extract_bits(inst >> 12, 3);
            let rs1 = RegisterIndex::from(extract_bits(inst >> 15, 5));
            let rd = RegisterIndex::from(extract_bits(inst >> 7, 5));

            let mut imm = extract_bits(inst >> 20, 12);
            imm = sign_extend(imm, 12);

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
            let func3 = extract_bits(inst >> 12, 3);
            let rs1 = RegisterIndex::from(extract_bits(inst >> 15, 5));
            let rs2 = RegisterIndex::from(extract_bits(inst >> 20, 5));

            let mut imm = extract_bits(inst >> 7, 5) + (extract_bits(inst >> 25, 7) << 5);
            imm = sign_extend(imm, 12);

            if func3 == 0b000 {
                return Inst::SB(rs1, rs2, imm);
            } else if func3 == 0b001 {
                return Inst::SH(rs1, rs2, imm);
            } else if func3 == 0b010 {
                return Inst::SW(rs1, rs2, imm);
            }
        }
        //  else if opcode == 0b1110011 && ((inst >> 20) == 0) {
        //     return Inst::ECALL;
        // }
        // else if inst == 3221229683 {
        //     return Inst::UNIMP;
        // }

        panic!("Instruction={} cannot be decoded", inst);
    }

    fn register(&self, index: RegisterIndex) -> u32 {
        self.registers[index.0 as usize]
    }

    fn register_mut(&mut self, index: RegisterIndex) -> &mut u32 {
        // if index.0 == 15 {
        //     println!("Access 15");
        // }
        &mut self.registers[index.0 as usize]
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
                verbose_println!(
                    "ADDI: rs1={rs1}({}), rd={rd}({}), imm={imm}",
                    self.register(rs1),
                    self.register(rd)
                );

                let rs1v = self.register(rs1);
                *self.register_mut(rd) = rs1v.wrapping_add(imm);

                self.pc += 4;
            }
            Inst::SLTI(rs1, rd, imm) => {
                verbose_println!(
                    "SLTI: rs1={rs1}({}), rd={rd}({}), imm={imm}",
                    self.register(rs1),
                    self.register(rd)
                );

                let rs1v = self.register(rs1) as i32;
                *self.register_mut(rd) = (rs1v < (imm as i32)) as u32;

                self.pc += 4;
            }
            Inst::SLTIU(rs1, rd, imm) => {
                verbose_println!(
                    "SLTIU: rs1={rs1}({}), rd={rd}({}), imm={imm}",
                    self.register(rs1),
                    self.register(rd)
                );

                let rs1v = self.register(rs1);
                *self.register_mut(rd) = (rs1v < imm) as u32;

                self.pc += 4;
            }
            Inst::XORI(rs1, rd, imm) => {
                verbose_println!(
                    "XORI: rs1={rs1}({}), rd={rd}({}), imm={imm}",
                    self.register(rs1),
                    self.register(rd)
                );

                let rs1v = self.register(rs1);
                *self.register_mut(rd) = rs1v ^ imm;

                self.pc += 4;
            }
            Inst::ORI(rs1, rd, imm) => {
                verbose_println!(
                    "ORI: rs1={rs1}({}), rd={rd}({}), imm={imm}",
                    self.register(rs1),
                    self.register(rd)
                );

                let rs1v = self.register(rs1);
                *self.register_mut(rd) = rs1v | imm;

                self.pc += 4;
            }
            Inst::ANDI(rs1, rd, imm) => {
                verbose_println!(
                    "ANDI: rs1={rs1}({}), rd={rd}({}), imm={imm}",
                    self.register(rs1),
                    self.register(rd)
                );

                let rs1v = self.register(rs1);
                *self.register_mut(rd) = rs1v & imm;

                self.pc += 4;
            }
            Inst::SLLI(rs1, rd, shift) => {
                verbose_println!(
                    "SLLI: rs1={rs1}({}), rd={rd}({}), shamt={shift}",
                    self.register(rs1),
                    self.register(rd)
                );

                let rs1v = self.register(rs1);
                *self.register_mut(rd) = rs1v << shift;

                self.pc += 4;
            }
            Inst::SRLI(rs1, rd, shift) => {
                verbose_println!(
                    "SRLI: rs1={rs1}({}), rd={rd}({}), shamt={shift}",
                    self.register(rs1),
                    self.register(rd)
                );

                let rs1v = self.register(rs1);
                *self.register_mut(rd) = rs1v >> shift;

                self.pc += 4;
            }
            Inst::SRAI(rs1, rd, shift) => {
                verbose_println!(
                    "SRAI: rs1={rs1}({}), rd={rd}({}), shamt={shift}",
                    self.register(rs1),
                    self.register(rd)
                );

                let rs1v = self.register(rs1);
                *self.register_mut(rd) = ((rs1v as i32) >> shift) as u32;

                self.pc += 4;
            }
            Inst::LUI(rd, imm) => {
                verbose_println!("LUI: rd={rd}({}), u-imm={imm}", self.register(rd));

                *self.register_mut(rd) = imm;

                self.pc += 4;
            }
            Inst::AUIPC(rd, imm) => {
                verbose_println!("AUIPC: rd={rd}({}), u-imm={imm}", self.register(rd));
                *self.register_mut(rd) = imm.wrapping_add(self.pc);

                self.pc += 4;
            }
            Inst::ADD(rs1, rs2, rd) => {
                verbose_println!(
                    "ADD: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = self.register(rs1).wrapping_add(self.register(rs2));

                self.pc += 4;
            }
            Inst::SUB(rs1, rs2, rd) => {
                verbose_println!(
                    "SUB: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = self.register(rs1).wrapping_sub(self.register(rs2));

                self.pc += 4;
            }
            Inst::SLL(rs1, rs2, rd) => {
                verbose_println!(
                    "SLL: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = self.register(rs1) << extract_bits(self.register(rs2), 5);

                self.pc += 4;
            }
            Inst::SLT(rs1, rs2, rd) => {
                verbose_println!(
                    "SLT: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) =
                    ((self.register(rs1) as i32) < (self.register(rs2) as i32)) as u32;

                self.pc += 4;
            }
            Inst::SLTU(rs1, rs2, rd) => {
                verbose_println!(
                    "SLTU: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = (self.register(rs1) < self.register(rs2)) as u32;

                self.pc += 4;
            }
            Inst::XOR(rs1, rs2, rd) => {
                verbose_println!(
                    "XOR: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = self.register(rs1) ^ self.register(rs2);

                self.pc += 4;
            }
            Inst::OR(rs1, rs2, rd) => {
                verbose_println!(
                    "OR: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = self.register(rs1) | self.register(rs2);

                self.pc += 4;
            }
            Inst::AND(rs1, rs2, rd) => {
                verbose_println!(
                    "AND: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = self.register(rs1) & self.register(rs2);

                self.pc += 4;
            }
            Inst::SRL(rs1, rs2, rd) => {
                verbose_println!(
                    "SRL: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = self.register(rs1) >> extract_bits(self.register(rs2), 5);

                self.pc += 4;
            }
            Inst::SRA(rs1, rs2, rd) => {
                verbose_println!(
                    "SRA: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) =
                    ((self.register(rs1) as i32) >> extract_bits(self.register(rs2), 5)) as u32;

                self.pc += 4;
            }
            Inst::JAL(rd, offset) => {
                verbose_println!("JAL: rd={}({}) offset={}", rd, self.register(rd), offset);

                let jump_target = self.pc.wrapping_add(offset);
                assert!(
                    jump_target % 4 == 0,
                    "Jump target={jump_target} is misaligned"
                );
                *self.register_mut(rd) = self.pc.wrapping_add(4);
                self.pc = jump_target;
            }
            Inst::JALR(rs1, rd, imm) => {
                verbose_println!(
                    "JALR: rs1={}({}) rd={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rd,
                    self.register(rd),
                    imm
                );

                let jump_target = ((self.register(rs1).wrapping_add(imm)) >> 1) << 1;
                assert!(
                    jump_target % 4 == 0,
                    "Jump target={jump_target} is misaligned"
                );
                *self.register_mut(rd) = self.pc.wrapping_add(4);
                self.pc = jump_target;
            }
            Inst::BEQ(rs1, rs2, imm) => {
                verbose_println!(
                    "BEQ: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    imm
                );

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
                verbose_println!(
                    "BNE: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    imm
                );

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
                verbose_println!(
                    "BLT: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    imm
                );

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
                verbose_println!(
                    "BLTU: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    imm
                );

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
                verbose_println!(
                    "BGE: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    imm
                );

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
                verbose_println!(
                    "BGEU: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    imm
                );

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
            Inst::LB(rs1, rd, imm) => {
                verbose_println!(
                    "LB: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rd,
                    self.register(rd),
                    imm
                );

                let addr = self.register(rs1).wrapping_add(imm);

                let mut value = self.ram.read_byte(addr as usize) as u32;
                value = sign_extend(value, 8);
                *self.register_mut(rd) = value;

                self.pc += 4;
            }
            Inst::LH(rs1, rd, imm) => {
                verbose_println!(
                    "LH: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rd,
                    self.register(rd),
                    imm
                );

                let addr = self.register(rs1).wrapping_add(imm);
                assert!(
                    (addr % 4 == 0) || ((addr - 1) % 4 == 0) || ((addr - 2) % 4 == 0),
                    "LH addr={addr} is not 4-byte aligned"
                );

                let mut value = self.ram.read_half(addr as usize) as u32;
                value = sign_extend(value, 16);
                *self.register_mut(rd) = value;

                self.pc += 4;
            }
            Inst::LW(rs1, rd, imm) => {
                verbose_println!(
                    "LW: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rd,
                    self.register(rd),
                    imm
                );

                let addr = self.register(rs1).wrapping_add(imm);
                assert!(addr % 4 == 0, "LW addr={addr} is not 4-byte aligned");

                *self.register_mut(rd) = self.ram.read_word(addr as usize);

                self.pc += 4;
            }
            Inst::LBU(rs1, rd, imm) => {
                verbose_println!(
                    "LBU: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rd,
                    self.register(rd),
                    imm
                );

                let addr = self.register(rs1).wrapping_add(imm);
                *self.register_mut(rd) = self.ram.read_byte(addr as usize) as u32;
                self.pc += 4;
            }
            Inst::LHU(rs1, rd, imm) => {
                verbose_println!(
                    "LHU: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rd,
                    self.register(rd),
                    imm
                );

                let addr = self.register(rs1).wrapping_add(imm);
                assert!(
                    (addr % 4 == 0) || ((addr - 1) % 4 == 0) || ((addr - 2) % 4 == 0),
                    "LHU addr={addr} is not 4-byte aligned"
                );

                *self.register_mut(rd) = self.ram.read_half(addr as usize) as u32;
                self.pc += 4;
            }
            Inst::SB(rs1, rs2, imm) => {
                verbose_println!(
                    "SB: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    imm
                );

                let addr = self.register(rs1).wrapping_add(imm);
                self.ram
                    .write_byte(addr as usize, extract_bits(self.register(rs2), 8) as u8);

                self.pc += 4;
            }
            Inst::SH(rs1, rs2, imm) => {
                verbose_println!(
                    "SH: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    imm
                );

                let addr = self.register(rs1).wrapping_add(imm);
                assert!(
                    (addr % 4 == 0) || ((addr - 1) % 4 == 0) || ((addr - 2) % 4 == 0),
                    "SH addr={addr} is not 4-byte aligned"
                );

                self.ram
                    .write_half(addr as usize, (extract_bits(self.register(rs2), 16)) as u16);

                self.pc += 4;
            }
            Inst::SW(rs1, rs2, imm) => {
                verbose_println!(
                    "SW: rs1={}({}) rs2={}({}) imm={}",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    imm
                );

                let addr = self.register(rs1).wrapping_add(imm);
                assert!(addr % 4 == 0, "SW addr={addr} is not 4-byte aligned");

                self.ram.write_word(addr as usize, self.register(rs2));

                self.pc += 4;
            }
            Inst::MUL(rs1, rs2, rd) => {
                verbose_println!(
                    "MUL: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = self.register(rs1).wrapping_mul(self.register(rs2));

                self.pc += 4;
            }
            Inst::MULH(rs1, rs2, rd) => {
                verbose_println!(
                    "MULH: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                // register values must be treated as signed. Casting u32 to i64 treats source as unsiged.
                // Hence, first case u32 -> i32 and then i32 -> i64
                *self.register_mut(rd) = (((((self.register(rs1) as i32) as i64)
                    .wrapping_mul((self.register(rs2) as i32) as i64))
                    as u64)
                    >> 32) as u32;

                self.pc += 4;
            }
            Inst::MULHU(rs1, rs2, rd) => {
                verbose_println!(
                    "MULHU: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = (((self.register(rs1) as u64)
                    .wrapping_mul(self.register(rs2) as u64))
                    >> 32) as u32;

                self.pc += 4;
            }
            Inst::MULHSU(rs1, rs2, rd) => {
                verbose_println!(
                    "MULHSU: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = (((((self.register(rs1) as i32) as i64)
                    .wrapping_mul(self.register(rs2) as i64))
                    as u64)
                    >> 32) as u32;

                self.pc += 4;
            }
            Inst::DIV(rs1, rs2, rd) => {
                verbose_println!(
                    "DIV: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) =
                    ((self.register(rs1) as i32) / (self.register(rs2) as i32)) as u32;

                self.pc += 4;
            }
            Inst::DIVU(rs1, rs2, rd) => {
                verbose_println!(
                    "DIVU: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = self.register(rs1) / self.register(rs2);

                self.pc += 4;
            }
            Inst::REM(rs1, rs2, rd) => {
                verbose_println!(
                    "REM: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) =
                    (self.register(rs1) as i32 % self.register(rs2) as i32) as u32;

                self.pc += 4;
            }
            Inst::REMU(rs1, rs2, rd) => {
                verbose_println!(
                    "REMU: rs1={}({}) rs2={}({}) rd={}({})",
                    rs1,
                    self.register(rs1),
                    rs2,
                    self.register(rs2),
                    rd,
                    self.register(rd)
                );

                *self.register_mut(rd) = self.register(rs1) % self.register(rs2);

                self.pc += 4;
            } // Inst::ECALL => {
              //     // a0 stores v_addrs, a1 stores v_len
              //     let addr = self.register(RegisterIndex::from(10));
              //     let len = self.register(RegisterIndex::from(11)) as usize;
              //     let mut out_bytes = Vec::with_capacity(len);
              //     for i in 0..len {
              //         out_bytes.push(self.ram.read_byte((addr as usize) + i));
              //     }
              //     let s = std::str::from_utf8(&out_bytes).unwrap();
              //     println!("[TestVM log] {s}");

              //     self.pc += 4;
              // }
              // Inst::UNIMP => {
              //     verbose_println!("UNIMP");
              //     // halt vm
              //     self.state = VMState::HALT
              // } // _ => {}
        }

        *self.register_mut(RegisterIndex(0)) = 0;
    }

    pub fn read_input_tape(&mut self, tape: &[u8]) {
        assert!(
            tape.len() == self.input_info.size as usize,
            "Input tape exceeds .inpdata size"
        );

        self.ram.load_memory(self.input_info.start_addr, tape);
    }

    pub fn output_tape(&self) -> Vec<u8> {
        // println!(
        //     "Output: start_addr={} size={}",
        //     self.output_info.start_addr, self.output_info.size
        // );
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

mod utils {
    pub(super) fn extract_bits(value: u32, bits: usize) -> u32 {
        return value & ((1u32 << bits) - 1);
    }

    pub(super) fn sign_extend(value: u32, bitlen: usize) -> u32 {
        assert!((value >> bitlen) == 0);
        let msb = (value >> (bitlen - 1)) & 1;
        let mut out_v = value;
        for i in bitlen..32 {
            out_v += msb << i;
        }
        return out_v;
    }
}
