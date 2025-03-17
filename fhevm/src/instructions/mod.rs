//! # RD UPDATE         
//! ID |  OP  |     4      |      4     |     4     |     4    |     4    |  5  |  5  |  5 |
//!    |------|------------|------------|-----------|----------|----------|-----|-----|----|
//!  0 |none  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rd]
//!  1 |lui   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(imm[19:0] << 12
//!  2 |auipc | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + sext(imm[19:0] << 12)
//!  3 |addi  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] + sext(imm[11:0])
//!  4 |slti  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = (x[rs1] < sext(imm[11:0])) ? 1 : 0
//!  5 |sltiu | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = (x[rs1] <u sext(imm[11:0])) ? 1 : 0
//!  6 |xori  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] ^ sext(imm[11:0])
//!  7 |ori   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] | sext(imm[11:0])
//!  8 |andi  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] & sext(imm[11:0])
//!  9 |slli  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] << rs2
//! 10 |srli  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] >> rs2 (logical)
//! 11 |srai  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] >> rs2 (arithmetic)
//!
//! 12 |add   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] + x[rs2]
//! 13 |sub   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] - x[rs2]
//! 14 |sll   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] << x[rs2]
//! 15 |slt   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = (x[rs1] < x[rs2]) ? 1 : 0
//! 16 |sltu  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = (x[rs1] <u x[rs2]) ? 1 : 0   
//! 17 |xor   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] ^ x[rs2]
//! 18 |srl   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] >> x[rs2] (logical)
//! 19 |sra   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] >> x[rs2] (arithmetic)
//! 20 |or    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] | x[rs2]  
//! 21 |and   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = x[rs1] & x[rs2]
//!
//! 22 |lb    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(lbu)
//! 23 |lh    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = sext(lhu)
//! 24 |lw    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = M[x[rs1] + sext(imm[11:0])][31:0]
//! 25 |lbu   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = M[x[rs1] + sext(imm[11:0])][7:0]
//! 26 |lhu   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = M[x[rs1] + sext(imm[11:0])][15:0]
//!
//! 27 |jal   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + 4
//! 28 |jalr  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | x[rd] = pc + 4
//!
//! # MEMORY UPDATE
//! ID |  OP  |     4      |      4     |     4     |     4    |     4    |  5  |  5  |  5 |
//!    |------|------------|------------|-----------|----------|----------|-----|-----|----|
//!  0 |none  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | M[x[rs1] + sext(imm[11:0])] = M[x[rs1] + sext(imm[11:0])]
//!  1 |sb    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | M[x[rs1] + sext(imm[11:0])] = x[rs2][7:0]
//!  2 |sh    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | M[x[rs1] + sext(imm[11:0])] = x[rs2][15:0]
//!  3 |sw    | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | M[x[rs1] + sext(imm[11:0])] = x[rs2][31:0]
//!
//! # PC UPDATE
//! ID |  OP  |     4      |      4     |     4     |     4    |     4    |  5  |  5  |  5 |
//!    |------|------------|------------|-----------|----------|----------|-----|-----|----|
//!  0 |one   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | pc += 4
//!  1 |jal   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | pc += sext(imm[19:0])
//!  2 |jalr  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | t = pc + 4; pc = (x[rs1] + sext(imm[11:0])) & ~1
//!  3 |beq   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] ==  x[rs2]), pc += sext(imm[19:0])
//!  4 |bne   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] !=  x[rs2]), pc += sext(imm[19:0])
//!  5 |blt   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] <   x[rs2]), pc += sext(imm[19:0])
//!  6 |bge   | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] >=s x[rs2]), pc += sext(imm[19:0])
//!  7 |bltu  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] <u  x[rs2]), pc += sext(imm[19:0])
//!  8 |bgeu  | imm[19:16] | imm[15:12] | imm[11:8] | imm[7:4] | imm[3:0] | rs2 | rs1 | rd | if (x[rs1] >=u x[rs2]), pc += sext(imm[19:0])

pub mod b_type;
pub mod i_type;
pub mod j_type;
pub mod memory;
pub mod r_type;
pub mod s_type;
pub mod u_type;

pub fn reconstruct(x: &[u8; 8]) -> u32 {
    let mut y: u32 = 0;
    y |= (x[7] as u32) << 28;
    y |= (x[6] as u32) << 24;
    y |= (x[5] as u32) << 20;
    y |= (x[4] as u32) << 16;
    y |= (x[3] as u32) << 12;
    y |= (x[2] as u32) << 8;
    y |= (x[1] as u32) << 4;
    y |= x[0] as u32;
    y
}

pub fn decompose(x: u32) -> [u8; 8] {
    let mut y: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
    y[0] = ((x >> 0) & 0xF) as u8;
    y[1] = ((x >> 4) & 0xF) as u8;
    y[2] = ((x >> 8) & 0xF) as u8;
    y[3] = ((x >> 12) & 0xF) as u8;
    y[4] = ((x >> 16) & 0xF) as u8;
    y[5] = ((x >> 20) & 0xF) as u8;
    y[6] = ((x >> 24) & 0xF) as u8;
    y[7] = ((x >> 28) & 0xF) as u8;
    y
}

pub fn sext(x: u32, bits: usize) -> u32 {
    x | ((x >> bits) & 1) * (0xFFFF_FFFF & (0xFFFF_FFFF << bits))
}

pub trait Arithmetic {
    fn apply(imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8]) -> [u8; 8];
}

pub trait PcUpdates {
    fn apply(imm: &[u8; 8], x_rs1: &[u8; 8], x_rs2: &[u8; 8], pc: &[u8; 8]) -> ([u8; 8], [u8; 8]);
}

pub enum StoreOps {
    Sb,
    Sh,
    Sw,
}

pub enum LoadOps {
    Lb,
    Lbu,
    Lh,
    Lhu,
    Lw,
}

pub enum PcUpdatesOps {
    Auipc(u_type::auipc::Auipc),
    Beq(b_type::beq::Beq),
    Bge(b_type::bge::Bge),
    Bgeu(b_type::bgeu::Bgeu),
    Blt(b_type::blt::Blt),
    Bltu(b_type::bltu::Bltu),
    Bne(b_type::bne::Bne),
    Jal(j_type::jal::Jal),
    Jalr(i_type::jalr::Jalr),
}

pub enum ArithmeticOps {
    Add(r_type::add::Add),
    Addi(i_type::addi::Addi),
    And(r_type::and::And),
    Andi(i_type::andi::Andi),
    Lui(u_type::lui::Lui),
    Or(r_type::or::Or),
    Ori(i_type::ori::Ori),
    //Sll(sll::Sll),
    //Slli(slli::Slli),
    //Slt(slt::Slt),
    //Slti(slti::Slti),
    //Sltiu(sltiu::Sltiu),
    //Sltu(sltu::Sltu),
    //Sra(sra::Sra),
    //Srai(srai::Srai),
    //Srl(slr::Srl),
    //Sub(sub::Sub),
    //Xor(xor::Xor),
    //Xori(xori::Xori),
}

#[non_exhaustive]
pub struct OpID;

impl OpID {
    pub const LUI: (u8, u8, u8) = (1, 0, 0);
    pub const AUIPC: (u8, u8, u8) = (2, 0, 0);
    pub const ADDI: (u8, u8, u8) = (3, 0, 0);
    pub const SLTI: (u8, u8, u8) = (4, 0, 0);
    pub const SLTIU: (u8, u8, u8) = (5, 0, 0);
    pub const XORI: (u8, u8, u8) = (6, 0, 0);
    pub const ORI: (u8, u8, u8) = (7, 0, 0);
    pub const ANDI: (u8, u8, u8) = (8, 0, 0);
    pub const SLLI: (u8, u8, u8) = (9, 0, 0);
    pub const SRLI: (u8, u8, u8) = (10, 0, 0);
    pub const SRAI: (u8, u8, u8) = (11, 0, 0);
    pub const ADD: (u8, u8, u8) = (12, 0, 0);
    pub const SUB: (u8, u8, u8) = (13, 0, 0);
    pub const SLL: (u8, u8, u8) = (14, 0, 0);
    pub const SLT: (u8, u8, u8) = (15, 0, 0);
    pub const SLTU: (u8, u8, u8) = (16, 0, 0);
    pub const XOR: (u8, u8, u8) = (17, 0, 0);
    pub const SRL: (u8, u8, u8) = (18, 0, 0);
    pub const SRA: (u8, u8, u8) = (19, 0, 0);
    pub const OR: (u8, u8, u8) = (20, 0, 0);
    pub const AND: (u8, u8, u8) = (21, 0, 0);
    pub const LB: (u8, u8, u8) = (22, 0, 0);
    pub const LH: (u8, u8, u8) = (23, 0, 0);
    pub const LW: (u8, u8, u8) = (24, 0, 0);
    pub const LBU: (u8, u8, u8) = (25, 0, 0);
    pub const LHU: (u8, u8, u8) = (26, 0, 0);
    pub const SB: (u8, u8, u8) = (0, 1, 0);
    pub const SH: (u8, u8, u8) = (0, 2, 0);
    pub const SW: (u8, u8, u8) = (0, 3, 0);
    pub const JAL: (u8, u8, u8) = (27, 0, 1);
    pub const JALR: (u8, u8, u8) = (28, 0, 2);
    pub const BEQ: (u8, u8, u8) = (0, 0, 3);
    pub const BNE: (u8, u8, u8) = (0, 0, 4);
    pub const BLT: (u8, u8, u8) = (0, 0, 5);
    pub const BGE: (u8, u8, u8) = (0, 0, 6);
    pub const BLTU: (u8, u8, u8) = (0, 0, 7);
    pub const BGEU: (u8, u8, u8) = (0, 0, 8);
}

pub struct InstructionsParser {
    pub imm: Vec<i64>,
    pub instructions: Vec<i64>,
}

impl InstructionsParser {
    pub fn new() -> Self {
        InstructionsParser {
            imm: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub fn add(&mut self, instruction: Instruction) {
        let (rs2, rs1, rd) = instruction.get_registers();
        let (rd_w, mem_w, pc_w) = instruction.get_opid();
        self.imm.push(instruction.get_immediate() as i64);
        self.instructions.push(
            (rs2 as i64) << 25
                | (rs1 as i64) << 20
                | (rd as i64) << 15
                | (rd_w as i64) << 10
                | (mem_w as i64) << 5
                | (pc_w as i64),
        );
    }

    pub fn assert_size(&self, size: usize) {
        assert_eq!(self.imm.len(), size);
        assert_eq!(self.instructions.len(), size);
    }

    pub fn get(&self, idx: usize) -> (i64, i64, i64, i64, i64, i64, i64) {
        assert!(self.imm.len() > idx);
        let data = self.instructions[idx];
        (
            self.imm[idx] as i64,
            ((data >> 25) & 0x1F) as i64,
            ((data >> 20) & 0x1F) as i64,
            ((data >> 15) & 0x1F) as i64,
            ((data >> 10) & 0x1F) as i64,
            ((data >> 5) & 0x1F) as i64,
            (data & 0x1F) as i64,
        )
    }

    pub fn assert_instruction(
        &self,
        idx: usize,
        imm: i64,
        rs2: i64,
        rs1: i64,
        rd: i64,
        rd_w: i64,
        mem_w: i64,
        pc_w: i64,
    ) {
        let (imm_have, rs2_have, rs1_have, rd_have, rd_w_have, mem_w_have, pc_w_have) =
            self.get(idx);

        let number_of_instructions: usize = self.imm.len();
        assert!(number_of_instructions > idx);
        assert_eq!(
            imm_have, imm,
            "invalid imm: have {:032b} want {:032b}",
            imm_have, imm
        );
        assert_eq!(
            rs2_have, rs2,
            "invalid rs2: have {:05b} want {:05b}",
            rs2_have, rs2
        );
        assert_eq!(
            rs1_have, rs1,
            "invalid rs1: have {:05b} want {:05b}",
            rs1_have, rs1
        );
        assert_eq!(
            rd_have, rd,
            "invalid rd: have {:05b} want {:05b}",
            rd_have, rd
        );
        assert_eq!(
            rd_w_have, rd_w,
            "invalid rd_w: have {} want {}",
            rd_w_have, rd_w
        );
        assert_eq!(
            mem_w_have, mem_w,
            "invalid mem_w: have {} want {}",
            mem_w_have, mem_w
        );
        assert_eq!(
            pc_w_have, pc_w,
            "invalid pc_w: have {} want {}",
            pc_w_have, pc_w
        );
    }
}

pub struct Instruction(u32);

pub const RS1MASK: u32 = 0x000F_8000;
pub const RS2MASK: u32 = 0x01F0_0000;
pub const FUNCT3MASK: u32 = 0x0000_7000;
pub const FUNCT7MASK: u32 = 0xFE00_0000;
pub const SHAMTMASK: u32 = 0x01F0_0000;
pub const RDMASK: u32 = 0x0000_0F80;
pub const OPCODEMASK: u32 = 0x0000_007F;

pub const RS1SHIFT: u32 = 15;
pub const RS2SHIFT: u32 = 20;
pub const FUNCT3SHIFT: u32 = 12;
pub const FUNCT7SHIFT: u32 = 25;
pub const SHAMTSHIFT: u32 = 20;
pub const RDSHIFT: u32 = 7;
pub const OPCODESHIFT: u32 = 0;

pub enum Type {
    R,
    I,
    S,
    B,
    U,
    J,
}

impl Instruction {
    #[inline(always)]
    pub fn new(instruction: u32) -> Self {
        Self(instruction)
    }

    pub fn print(&self) {
        println!("{:032b}", self.0);
    }

    #[inline(always)]
    pub fn get_type(&self) -> Type {
        let opcode: u8 = self.get_opcode();
        match opcode {
            0b0110111 | 0b0010111 => Type::U,
            0b0010011 | 0b0000011 | 0b1100111 => Type::I,
            0b0110011 => Type::R,
            0b0100011 => Type::S,
            0b1101111 => Type::J,
            0b1100011 => Type::B,
            _ => panic!("unrecognized opcode: {:07b}", opcode),
        }
    }

    #[inline(always)]
    pub fn get_funct3(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::U | Type::J => panic!(
                    "cannot get funct3 on Type::(U, J) instructions: {:032b}",
                    self.0
                ),
                _ => {}
            }
        }
        ((self.0 & FUNCT3MASK) >> FUNCT3SHIFT) as u8
    }

    #[inline(always)]
    pub fn set_funct3(&mut self, funct3: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::U | Type::J => panic!(
                    "cannot set funct3 on Type::(U, J) instructions: {:032b}",
                    self.0
                ),
                _ => {}
            }
        }
        self.0 =
            (self.0 & (0xFFFF_FFFF ^ FUNCT3MASK)) | ((funct3 as u32) << FUNCT3SHIFT) & FUNCT3MASK
    }

    #[inline(always)]
    pub fn get_funct7(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::I => {}
                _ => panic!(
                    "cannot get funct7 on Type::(S, B, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        ((self.0 & FUNCT7MASK) >> FUNCT7SHIFT) as u8
    }

    #[inline(always)]
    pub fn set_funct7(&mut self, funct7: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R => {}
                _ => panic!(
                    "cannot set funct7 on Type::(I, S, B, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        self.0 =
            (self.0 & (0xFFFF_FFFF ^ FUNCT7MASK)) | ((funct7 as u32) << FUNCT7SHIFT) & FUNCT7MASK
    }

    #[inline(always)]
    pub fn get_rs1(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::I | Type::S | Type::B => {}
                _ => panic!(
                    "cannot get rs1 on Type::(U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        ((self.0 & RS1MASK) >> RS1SHIFT) as u8
    }

    #[inline(always)]
    pub fn set_rs1(&mut self, rs1: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::I | Type::S | Type::B => {}
                _ => panic!(
                    "cannot set rs1 on Type::(U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        self.0 = (self.0 & (0xFFFF_FFFF ^ RS1MASK)) | ((rs1 as u32) << RS1SHIFT) & RS1MASK
    }

    #[inline(always)]
    pub fn get_rs2(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::S | Type::B => {}
                _ => panic!(
                    "cannot get rs2 on Type::(I, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        ((self.0 & RS2MASK) >> RS2SHIFT) as u8
    }

    #[inline(always)]
    pub fn set_rs2(&mut self, rs2: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::S | Type::B => {}
                _ => panic!(
                    "cannot set rs2 on Type::(I, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        self.0 = (self.0 & (0xFFFF_FFFF ^ RS2MASK)) | ((rs2 as u32) << RS2SHIFT) & RS2MASK
    }

    #[inline(always)]
    pub fn get_rd(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::I | Type::U | Type::J => {}
                _ => panic!(
                    "cannot get rd on Type::(S, B) instructions: {:032b}",
                    self.0
                ),
            }
        }
        ((self.0 & RDMASK) >> RDSHIFT) as u8
    }

    #[inline(always)]
    pub fn set_rd(&mut self, rd: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::R | Type::I | Type::U | Type::J => {}
                _ => panic!(
                    "cannot set rd on Type::(S, B) instructions: {:032b}",
                    self.0
                ),
            }
        }
        self.0 = (self.0 & (0xFFFF_FFFF ^ RDMASK)) | ((rd as u32) << RDSHIFT) & RDMASK
    }

    #[inline(always)]
    pub fn get_shamt(&self) -> u8 {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::I => {}
                _ => panic!(
                    "cannot get shamt on Type::(R, S, B, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        ((self.0 & SHAMTMASK) >> SHAMTSHIFT) as u8
    }

    #[inline(always)]
    pub fn set_shamt(&mut self, shamt: u8) {
        #[cfg(debug_assertions)]
        {
            match self.get_type() {
                Type::I => {}
                _ => panic!(
                    "cannot set shamt on Type::(R, S, B, U, J) instructions: {:032b}",
                    self.0
                ),
            }
        }
        self.0 = (self.0 & (0xFFFF_FFFF ^ SHAMTMASK)) | ((shamt as u32) << SHAMTSHIFT) & SHAMTMASK
    }

    #[inline(always)]
    pub fn get_opcode(&self) -> u8 {
        ((self.0 & OPCODEMASK) >> OPCODESHIFT) as u8
    }

    #[inline(always)]
    pub fn set_opcode(&mut self, opcode: u8) {
        self.0 =
            (self.0 & (0xFFFF_FFFF ^ OPCODEMASK)) | ((opcode as u32) << OPCODESHIFT) & OPCODEMASK
    }

    #[inline(always)]
    pub fn set_immediate(&mut self, immediate: u32) {
        match self.get_type() {
            Type::R => panic!("cannot encode immediate on type R instruction"),
            Type::I => match (self.get_funct3(), self.get_opcode()) {
                (0b001, 0b0010011) | (0b101, 0b0010011) => self.set_shamt(immediate as u8),
                _ => i_type::set_immediate(&mut self.0, immediate),
            },
            Type::S => s_type::set_immediate(&mut self.0, immediate),
            Type::B => b_type::set_immediate(&mut self.0, immediate),
            Type::U => u_type::set_immediate(&mut self.0, immediate),
            Type::J => j_type::set_immediate(&mut self.0, immediate),
        }
    }

    #[inline(always)]
    pub fn get_immediate(&self) -> u32 {
        match self.get_type() {
            Type::R => 0,
            Type::I => match (self.get_funct3(), self.get_opcode()) {
                (0b001, 0b0010011) | (0b101, 0b0010011) => self.get_shamt() as u32,
                _ => i_type::get_immediate(&self.0),
            },
            Type::S => s_type::get_immediate(&self.0),
            Type::B => b_type::get_immediate(&self.0),
            Type::U => u_type::get_immediate(&self.0),
            Type::J => j_type::get_immediate(&self.0),
        }
    }

    #[inline(always)]
    pub fn get_registers(&self) -> (u8, u8, u8) {
        match self.get_type() {
            Type::R => (self.get_rs2(), self.get_rs1(), self.get_rd()),
            Type::I => (0, self.get_rs1(), self.get_rd()),
            Type::S | Type::B => (self.get_rs2(), self.get_rs1(), 0),
            Type::U | Type::J => (0, 0, self.get_rd()),
        }
    }

    #[inline(always)]
    pub fn get_opid(&self) -> (u8, u8, u8) {
        let opcode: u8 = self.get_opcode();
        match self.get_type() {
            Type::R => match (self.get_funct7(), self.get_funct3()) {
                (0b0000000, 0b000) => OpID::ADD,
                (0b0100000, 0b000) => OpID::SUB,
                (0b0000000, 0b001) => OpID::SLL,
                (0b0000000, 0b010) => OpID::SLT,
                (0b0000000, 0b011) => OpID::SLTU,
                (0b0000000, 0b100) => OpID::XOR,
                (0b0000000, 0b101) => OpID::SRL,
                (0b0100000, 0b101) => OpID::SRA,
                (0b0000000, 0b110) => OpID::OR,
                (0b0000000, 0b111) => OpID::AND,
                _ => panic!(
                    "invalid funct3 {:03b} or funct7 {:07b}: {:032b}",
                    self.get_funct3(),
                    self.get_funct7(),
                    self.0
                ),
            },
            Type::I => {
                let funct3: u8 = self.get_funct3();
                match opcode {
                    0b0010011 => match funct3 {
                        0b000 => OpID::ADDI,
                        0b010 => OpID::SLTI,
                        0b011 => OpID::SLTIU,
                        0b100 => OpID::XORI,
                        0b110 => OpID::ORI,
                        0b111 => OpID::ANDI,
                        0b001 => OpID::SLLI,
                        0b101 => match self.get_funct7() {
                            0b0000000 => OpID::SRLI,
                            0b0100000 => OpID::SRAI,
                            _ => panic!("invalid funct7: {:032b}", self.0),
                        },
                        _ => panic!("invalid funct3: {:032b}", self.0),
                    },
                    0b0000011 => match funct3 {
                        0b000 => OpID::LB,
                        0b001 => OpID::LH,
                        0b010 => OpID::LW,
                        0b100 => OpID::LBU,
                        0b101 => OpID::LHU,
                        _ => panic!("invalid funct3: {:032b}", self.0),
                    },
                    0b1100111 => OpID::JALR,
                    _ => panic!("invalid instruction: {:032b}", self.0),
                }
            }
            Type::S => match self.get_funct3() {
                0b000 => OpID::SB,
                0b001 => OpID::SH,
                0b010 => OpID::SW,
                _ => panic!("invalid funct3: {:032b}", self.0),
            },
            Type::B => match self.get_funct3() {
                0b000 => OpID::BEQ,
                0b001 => OpID::BNE,
                0b100 => OpID::BLT,
                0b101 => OpID::BGE,
                0b110 => OpID::BLTU,
                0b111 => OpID::BGEU,
                _ => panic!("invalid funct3: {:032b}", self.0),
            },
            Type::U => match opcode {
                0b0110111 => OpID::LUI,
                0b0010111 => OpID::AUIPC,
                _ => panic!("invalid instruction: {:032b}", self.0),
            },
            Type::J => OpID::JAL,
        }
    }
}
