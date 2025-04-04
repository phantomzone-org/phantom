use elf::{
    abi::{PF_R, PF_W, PF_X, PT_LOAD},
    segment::ProgramHeader,
};
use fhevm::instructions::Instruction;
use itertools::Itertools;
use testvm::TestVM;

mod testvm;

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

        // load all +r/+rw headers
        let hdrs: Vec<&ProgramHeader> = phdrs
            .iter()
            .filter(|p| (p.p_flags == PF_R || p.p_flags == PF_R + PF_W))
            .collect();
        let mut ram_offset = 0;
        let mut boot_ram_data = vec![0u8; 1 << 18];
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
        let boot_ram = BootMemory::new(ram_offset, 1 << 18, boot_ram_data);

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

    pub fn encrypted_interpreter(&self, input_tape: &[u8]) {
        // map .text section to collection of Instructions
        // boot_rom always has offset = 0
        assert!(self.boot_rom.data.len() % 4 == 0);
        let _instructions = self
            .boot_rom
            .data
            .chunks_exact(4)
            .map(|four_bytes| {
                let mut inst = 0u32;
                for i in 0..4 {
                    inst += (four_bytes[i] as u32) << (i * 8);
                }
                Instruction::new(inst)
            })
            .collect_vec();

        // RAM
        let ram_offset = self.boot_ram.offset;
        assert!(self.boot_ram.size % 4 == 0);
        let mut ram_with_input = self.boot_ram.data.clone();
        // read input tape
        assert!(input_tape.len() == self.input_info.size);
        ram_with_input[(self.input_info.start_addr - ram_offset)
            ..(self.input_info.start_addr + self.input_info.size - ram_offset)]
            .copy_from_slice(input_tape);
        // RAM: byte vector -> u32 vec
        let _ram_data_u32 = ram_with_input
            .chunks_exact(4)
            .map(|four_bytes| {
                let mut date_u32 = 0u32;
                for i in 0..4 {
                    date_u32 += (four_bytes[i] as u32) << (i * 8);
                }
                date_u32
            })
            .collect_vec();

        // println!("Instructions: {:?}", _instructions);
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
