use elf::{
    abi::{PF_R, PF_W, PF_X, PT_LOAD},
    segment::ProgramHeader,
};
use fhevm::{
    instructions::{Instruction, InstructionsParser},
    interpreter::Interpreter,
    parameters::Parameters,
};
use itertools::Itertools;
use testvm::TestVM;

mod testvm;

const RAM_SIZE: usize = 1 << 13;

struct BootMemory {
    data: Vec<u8>,
    offset: usize,
    size: usize,
}

impl BootMemory {
    fn new(offset: usize, size: usize, data: Vec<u8>) -> Self {
        Self { data, offset, size }
    }
}

#[derive(Clone)]
struct InputInfo {
    start_addr: usize,
    size: usize,
}

#[derive(Clone)]
struct OutputInfo {
    start_addr: usize,
    size: usize,
}

/// Phantom VM: Encrypted Risc-v
pub struct Phantom {
    boot_rom: BootMemory,
    boot_ram: BootMemory,
    output_info: OutputInfo,
    input_info: InputInfo,
    _elf_bytes: Vec<u8>,
}

pub struct EncryptedVM {
    params: Parameters,
    interpreter: Interpreter,
    output_info: OutputInfo,
    ram_offset: usize,
    max_cycles: usize,
}

impl EncryptedVM {
    pub fn execute(&mut self) {
        let mut curr_cycles = 0;
        while curr_cycles < self.max_cycles {
            self.interpreter.cycle(&self.params);
            curr_cycles += 1;
        }
    }

    pub fn print_debug(&self) {
        println!(
            "PC: {}",
            self.interpreter.addr_pc.debug_as_u32(self.params.module_lwe())
        );
        println!("Registers: {:?}", self.interpreter.registers.debug_as_u32());
    }

    pub fn output_tape(&self) -> Vec<u8> {
        let mem_bytes: Vec<u8> = (&self.interpreter.memory).into();
        assert!(mem_bytes.len() == RAM_SIZE);

        let mut output = Vec::with_capacity(self.output_info.size);
        for i in 0..self.output_info.size {
            output.push(mem_bytes[(self.output_info.start_addr + i - self.ram_offset) % RAM_SIZE]);
        }
        output
    }
}

impl Phantom {
    pub fn init(elf_bytes: Vec<u8>) -> Self {
        let elf = elf::ElfBytes::<elf::endian::LittleEndian>::minimal_parse(&elf_bytes).unwrap();

        let phdrs: Vec<ProgramHeader> = elf
            .segments()
            .unwrap()
            .iter()
            .filter(|ph| ph.p_type == PT_LOAD)
            .collect();

        // .text section: +rx
        let txthdr = phdrs
            .iter()
            .find(|p| p.p_flags == PF_R + PF_X)
            .expect("Program header for .text not found");
        assert!(
            txthdr.p_filesz == txthdr.p_memsz,
            ".text phdr: contains uninitiliased values"
        );
        assert!(
            txthdr.p_vaddr == 0,
            ".text phdr: .text section must start from 0 offset"
        );
        let boot_rom = BootMemory::new(
            txthdr.p_vaddr as usize,
            txthdr.p_memsz as usize,
            elf_bytes[txthdr.p_offset as usize..(txthdr.p_offset + txthdr.p_memsz) as usize]
                .to_vec(),
        );

        println!("ROM SIZE: {}", txthdr.p_memsz);

        // load all +r/+rw headers
        let hdrs: Vec<&ProgramHeader> = phdrs
            .iter()
            .filter(|p| (p.p_flags == PF_R || p.p_flags == PF_R + PF_W))
            .collect();
        let mut ram_offset = 0;
        let mut boot_ram_data = vec![0u8; RAM_SIZE];
        if hdrs.len() > 0 {
            ram_offset = hdrs[0].p_vaddr as usize;
            // load ram with .inpdata,.rodata,.data.,etc.
            hdrs.iter().for_each(|ph| {
                // assert!(
                //     ph.p_filesz == ph.p_memsz,
                //     "Header contains uninitialised values (most probably .bss exists)"
                // );
                if ph.p_memsz > 0 && ph.p_filesz == ph.p_memsz {
                    let vaddr = ph.p_vaddr as usize;
                    boot_ram_data
                        [(vaddr - ram_offset)..(vaddr + (ph.p_memsz as usize) - ram_offset)]
                        .copy_from_slice(
                            &elf_bytes[ph.p_offset as usize..(ph.p_memsz + ph.p_offset) as usize],
                        );
                }
            });
        }
        let boot_ram = BootMemory::new(ram_offset, RAM_SIZE, boot_ram_data);
        // println!("RAM OFFSET: {}", ram_offset);

        // gather input information
        let inpdata_sec = elf
            .section_header_by_name(".inpdata")
            .expect(".inpdata section does not exist")
            .expect(".inpdata section does not exist");
        let input_info = InputInfo {
            start_addr: inpdata_sec.sh_addr as usize,
            size: inpdata_sec.sh_size as usize,
        };

        // gather output information
        let outdata_sec = elf
            .section_header_by_name(".outdata")
            .expect(".outdata section does not exist")
            .expect(".outdata section does not exist");
        let output_info = OutputInfo {
            start_addr: outdata_sec.sh_addr as usize,
            size: outdata_sec.sh_size as usize,
        };

        // println!(
        //     ".inpdata section: size={}, flag={}, v_addr={}, values={:?}",
        //     inpdata_sec.sh_size,
        //     inpdata_sec.sh_flags,
        //     inpdata_sec.sh_addr,
        //     &elf_bytes[inpdata_sec.sh_offset as usize
        //         ..(inpdata_sec.sh_offset + inpdata_sec.sh_size) as usize]
        // );

        // println!(
        //     ".outdata section: size={}, flag={}, v_addr={}, values={:?}",
        //     outdata_sec.sh_size,
        //     outdata_sec.sh_flags,
        //     outdata_sec.sh_addr,
        //     &elf_bytes[outdata_sec.sh_offset as usize
        //         ..(outdata_sec.sh_offset + outdata_sec.sh_size) as usize]
        // );

        Phantom {
            boot_rom,
            boot_ram,
            output_info,
            input_info,
            _elf_bytes: elf_bytes,
        }
    }

    pub fn encrypted_vm(&self, input_tape: &[u8], max_cycles: usize) -> EncryptedVM {
        // map .text section to collection of Instructions
        // boot_rom always has offset = 0
        assert!(self.boot_rom.data.len() % 4 == 0);
        let mut parser = InstructionsParser::new();
        self.boot_rom
            .data
            .chunks_exact(4)
            .map(|four_bytes| {
                let mut inst = 0u32;
                for i in 0..4 {
                    inst += (four_bytes[i] as u32) << (i * 8);
                }
                Instruction::new(inst)
            })
            .for_each(|i| parser.add(i));

        // setup RAM
        let ram_offset = self.boot_ram.offset;
        assert!(self.boot_ram.size % 4 == 0);
        let mut ram_with_input = self.boot_ram.data.clone();
        // read input tape
        assert!(input_tape.len() == self.input_info.size);
        ram_with_input[(self.input_info.start_addr - ram_offset)
            ..(self.input_info.start_addr + self.input_info.size - ram_offset)]
            .copy_from_slice(input_tape);
        // RAM: byte vector -> u32 vec
        let ram_data_u32 = ram_with_input
            .chunks_exact(4)
            .map(|four_bytes| {
                let mut date_u32 = 0u32;
                for i in 0..4 {
                    date_u32 += (four_bytes[i] as u32) << (i * 8);
                }
                date_u32
            })
            .collect_vec();

        // Initialize interpreter
        let params = Parameters::new();
        let mut interpreter = Interpreter::new(&params);
        interpreter.init_pc(&params);
        interpreter.init_instructions(parser);
        interpreter.init_registers(&vec![0u32; 32]);
        interpreter.init_memory(&ram_data_u32);

        // interpreter.cycle(&params);

        // println!("Instructions: {:?}", _instructions);

        EncryptedVM {
            params: params,
            interpreter,
            output_info: self.output_info.clone(),
            ram_offset: self.boot_ram.offset,
            max_cycles,
        }
    }

    pub fn test_vm(&self) -> TestVM {
        TestVM::init(
            &self.boot_rom,
            &self.boot_ram,
            &self.input_info,
            &self.output_info,
        )
    }
}
