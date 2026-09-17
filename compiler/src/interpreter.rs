use elf::{
    abi::{PF_R, PF_W, PF_X, PT_LOAD},
    segment::ProgramHeader,
};
use tqdm::tqdm;

use fhevm::{
    instructions::{Instruction, InstructionsParser},
    keys::{VMKeys, VMKeysPrepared},
    parameters::CryptographicParameters,
    Interpreter,
};

use fhevm::backend::{prelude::*, PhantomBackend, PhantomBootstrapModule};
use poulpy_bin_fhe::blind_rotation::CGGI;
use poulpy_core::layouts::{GLWESecret, GLWESecretPrepared, LWESecret};
#[cfg(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "fma"
))]
use poulpy_cpu_avx::{FFT64Avx, FFT64AvxRayon};
use poulpy_cpu_ref::FFT64Ref;
use poulpy_hal::{
    layouts::{Module, ScratchOwned},
    source::Source,
};
use testvm::TestVM;

mod testvm;

// RAM size default to 4KB
const RAM_SIZE: usize = 4 * 1024;

mod macros {
    macro_rules! verbose_println {
    ($($arg:tt)*) => {
        #[cfg(feature = "verbose")]
        println!($($arg)*);
    };
    }

    pub(crate) use verbose_println;
}

pub struct BootMemory {
    data: Vec<u8>,
    offset: usize,
    size: usize,
}

impl BootMemory {
    fn new(offset: usize, size: usize, data: Vec<u8>) -> Self {
        Self { data, offset, size }
    }

    pub fn data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn offset(&self) -> &usize {
        &self.offset
    }

    pub fn size(&self) -> &usize {
        &self.size
    }
}

#[derive(Clone)]
pub struct InputInfo {
    start_addr: usize,
    size: usize,
}

impl InputInfo {
    pub fn start_addr(&self) -> &usize {
        &self.start_addr
    }

    pub fn size(&self) -> &usize {
        &self.size
    }
}

#[derive(Clone)]
pub struct OutputInfo {
    start_addr: usize,
    size: usize,
}

impl OutputInfo {
    pub fn start_addr(&self) -> &usize {
        &self.start_addr
    }

    pub fn size(&self) -> &usize {
        &self.size
    }
}

/// Whether this build carries the AVX2/FMA backends (`RUSTFLAGS="-C target-feature=+avx2,+fma"`).
pub const HAS_AVX: bool = cfg!(all(
    target_arch = "x86_64",
    target_feature = "avx2",
    target_feature = "fma"
));

/// The poulpy backend the encrypted VM runs on.
///
/// Selected with `PHANTOM_BACKEND` (`fft64-ref`, `fft64-avx`, `fft64-avx-rayon`);
/// the default is the fastest available one, `fft64-avx` when the build has AVX2
/// and FMA, `fft64-ref` otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Backend {
    /// Portable backend; Phantom's own loops are multi-threaded, poulpy's are serial.
    Fft64Ref,
    /// AVX2/FMA backend; Phantom's own loops are multi-threaded, poulpy's are serial.
    Fft64Avx,
    /// AVX2/FMA backend scheduled by rayon: every parallel loop, Phantom's and
    /// poulpy's, runs on a rayon pool of `PHANTOM_THREADS` threads.
    Fft64AvxRayon,
}

impl Backend {
    pub const ENV: &'static str = "PHANTOM_BACKEND";

    pub fn name(self) -> &'static str {
        match self {
            Backend::Fft64Ref => "fft64-ref",
            Backend::Fft64Avx => "fft64-avx",
            Backend::Fft64AvxRayon => "fft64-avx-rayon",
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().replace('_', "-").as_str() {
            "fft64-ref" | "ref" => Some(Backend::Fft64Ref),
            "fft64-avx" | "avx" => Some(Backend::Fft64Avx),
            "fft64-avx-rayon" | "avx-rayon" | "rayon" => Some(Backend::Fft64AvxRayon),
            _ => None,
        }
    }

    pub fn is_available(self) -> bool {
        match self {
            Backend::Fft64Ref => true,
            Backend::Fft64Avx | Backend::Fft64AvxRayon => HAS_AVX,
        }
    }

    /// The fastest backend this build supports.
    pub fn default_backend() -> Self {
        if HAS_AVX {
            Backend::Fft64Avx
        } else {
            Backend::Fft64Ref
        }
    }

    /// Reads `PHANTOM_BACKEND`, falling back to [`Self::default_backend`].
    ///
    /// # Panics
    ///
    /// Panics on an unknown name, or on an AVX backend in a build without AVX2/FMA.
    pub fn from_env() -> Self {
        match std::env::var(Self::ENV) {
            Ok(name) => {
                let backend = Self::parse(&name).unwrap_or_else(|| {
                    panic!(
                        "{}={name:?}: expected fft64-ref, fft64-avx or fft64-avx-rayon",
                        Self::ENV
                    )
                });
                assert!(
                    backend.is_available(),
                    "{}={}: this build has no AVX2/FMA support, rebuild with RUSTFLAGS=\"-C target-feature=+avx2,+fma\"",
                    Self::ENV,
                    backend.name()
                );
                backend
            }
            Err(_) => Self::default_backend(),
        }
    }
}

/// Run-time knobs read from the environment.
#[derive(Debug, Clone, Copy)]
pub struct RunOptions {
    pub backend: Backend,
    pub threads: usize,
    pub verbose_timings: bool,
    pub debug: bool,
}

impl RunOptions {
    /// `PHANTOM_BACKEND`, `PHANTOM_THREADS` (default: physical cores),
    /// `PHANTOM_VERBOSE_TIMINGS` and `PHANTOM_DEBUG`.
    pub fn from_env() -> Self {
        let flag = |name: &str| {
            std::env::var(name)
                .map(|val| val == "1" || val.eq_ignore_ascii_case("true"))
                .unwrap_or(false)
        };
        let default_threads = num_cpus::get_physical();
        let threads = std::env::var("PHANTOM_THREADS")
            .map(|val| val.parse::<usize>().unwrap_or(default_threads))
            .unwrap_or(default_threads);
        Self {
            backend: Backend::from_env(),
            threads,
            verbose_timings: flag("PHANTOM_VERBOSE_TIMINGS"),
            debug: flag("PHANTOM_DEBUG"),
        }
    }
}

/// The encrypted VM on one backend; construct it through [`Phantom::encrypted_vm_with`].
pub struct EncryptedVMImpl<BE: PhantomBackend> {
    params: CryptographicParameters<BE>,
    sk_prepared: GLWESecretPrepared<Vec<u8>, BE>,
    key_prepared: VMKeysPrepared<Vec<u8>, CGGI, BE>,
    interpreter: Interpreter<BE>,
    output_info: OutputInfo,
    ram_offset: usize,
    max_cycles: usize,
    phantom_debug: bool,
}

impl<BE: PhantomBackend> EncryptedVMImpl<BE>
where
    Module<BE>: ModuleNew<BE> + PhantomBootstrapModule<CGGI, BE>,
    ScratchOwned<BE>: ScratchOwnedAlloc<BE> + ScratchOwnedBorrow<BE>,
{
    fn new(
        phantom: &Phantom,
        parser: &InstructionsParser,
        ram_data_u32: &[u32],
        max_cycles: usize,
        options: RunOptions,
    ) -> Self {
        // Initializing cryptographic parameters
        let params = CryptographicParameters::<BE>::new();
        let mut source_xs: Source = Source::new([0u8; 32]);
        let mut source_xa: Source = Source::new([0u8; 32]);
        let mut source_xe: Source = Source::new([0u8; 32]);
        let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

        // Generates a new secret-key along with the public evaluation keys.
        let module = params.module();
        let mut sk_glwe: GLWESecret<Vec<u8>, i64> = module.glwe_secret_alloc(params.rank());
        module.glwe_secret_fill_ternary_prob(&mut sk_glwe, 0.5, &mut source_xs);

        let mut sk_lwe: LWESecret<Vec<u8>, i64> = module.lwe_secret_alloc(params.n_lwe());
        module.lwe_secret_fill_binary_block(&mut sk_lwe, params.lwe_block_size(), &mut source_xs);

        let rom_size = phantom.boot_rom.size >> 2;
        let ram_size = phantom.boot_ram.size >> 2;
        let mut interpreter: Interpreter<BE> = if options.debug {
            Interpreter::new_with_debug(&params, rom_size, ram_size)
        } else {
            Interpreter::new(&params, rom_size, ram_size)
        };

        let mut sk_prepared: GLWESecretPrepared<Vec<u8>, BE> =
            module.glwe_secret_prepared_alloc(params.rank());
        module.glwe_secret_prepare(&mut sk_prepared, &sk_glwe);

        interpreter.instructions_encrypt_sk(
            module,
            parser,
            &sk_prepared,
            &mut source_xa,
            &mut source_xe,
            &mut scratch.borrow(),
        );
        let key: VMKeys<Vec<u8>, CGGI> =
            VMKeys::encrypt_sk(&params, &sk_lwe, &sk_glwe, &mut source_xa, &mut source_xe);

        let mut key_prepared: VMKeysPrepared<Vec<u8>, CGGI, BE> = VMKeysPrepared::alloc(&params);
        key_prepared.prepare(module, &key, &mut scratch.borrow());

        interpreter.ram_encrypt_sk(
            module,
            ram_data_u32,
            &sk_prepared,
            &mut source_xa,
            &mut source_xe,
            &mut scratch.borrow(),
        );

        interpreter.set_verbose_timings(options.verbose_timings);
        interpreter.set_threads(options.threads);

        Self {
            params,
            sk_prepared,
            key_prepared,
            interpreter,
            output_info: phantom.output_info.clone(),
            ram_offset: phantom.boot_ram.offset,
            max_cycles,
            phantom_debug: options.debug,
        }
    }

    fn execute(&mut self) {
        let mut scratch: ScratchOwned<BE> =
            ScratchOwned::alloc((1 << 24) * self.interpreter.threads());

        for _current_cycle in tqdm(0..self.max_cycles) {
            if self.phantom_debug {
                self.interpreter.cycle_debug(
                    self.params.module(),
                    &self.key_prepared,
                    &self.sk_prepared,
                    &mut scratch.borrow(),
                );
            } else {
                self.interpreter.cycle(
                    self.params.module(),
                    &self.key_prepared,
                    &mut scratch.borrow(),
                );
            }
        }
    }

    fn output_tape(&mut self) -> Vec<u8> {
        let mut scratch: ScratchOwned<BE> = ScratchOwned::alloc(1 << 24);

        let mut data_decrypted: Vec<u32> = vec![0u32; RAM_SIZE >> 2];
        self.interpreter.ram_decrypt(
            self.params.module(),
            &mut data_decrypted,
            &self.sk_prepared,
            &mut scratch.borrow(),
        );
        let mut mem_bytes = Vec::with_capacity(data_decrypted.len() * 4);
        for word in &data_decrypted {
            mem_bytes.push((*word & 0xFF) as u8);
            mem_bytes.push(((*word >> 8) & 0xFF) as u8);
            mem_bytes.push(((*word >> 16) & 0xFF) as u8);
            mem_bytes.push(((*word >> 24) & 0xFF) as u8);
        }

        let mut output = Vec::with_capacity(self.output_info.size);
        for i in 0..self.output_info.size {
            output.push(mem_bytes[(self.output_info.start_addr + i - self.ram_offset) % RAM_SIZE]);
        }
        output
    }
}

/// The encrypted VM, on the backend chosen at construction (see [`Backend`]).
pub enum EncryptedVM {
    Fft64Ref(Box<EncryptedVMImpl<FFT64Ref>>),
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "fma"
    ))]
    Fft64Avx(Box<EncryptedVMImpl<FFT64Avx>>),
    #[cfg(all(
        target_arch = "x86_64",
        target_feature = "avx2",
        target_feature = "fma"
    ))]
    Fft64AvxRayon(Box<EncryptedVMImpl<FFT64AvxRayon>>),
}

macro_rules! dispatch {
    ($self:expr, $vm:ident => $body:expr) => {
        match $self {
            EncryptedVM::Fft64Ref($vm) => $body,
            #[cfg(all(
                target_arch = "x86_64",
                target_feature = "avx2",
                target_feature = "fma"
            ))]
            EncryptedVM::Fft64Avx($vm) => $body,
            #[cfg(all(
                target_arch = "x86_64",
                target_feature = "avx2",
                target_feature = "fma"
            ))]
            EncryptedVM::Fft64AvxRayon($vm) => $body,
        }
    };
}

impl EncryptedVM {
    pub fn backend(&self) -> Backend {
        match self {
            EncryptedVM::Fft64Ref(_) => Backend::Fft64Ref,
            #[cfg(all(
                target_arch = "x86_64",
                target_feature = "avx2",
                target_feature = "fma"
            ))]
            EncryptedVM::Fft64Avx(_) => Backend::Fft64Avx,
            #[cfg(all(
                target_arch = "x86_64",
                target_feature = "avx2",
                target_feature = "fma"
            ))]
            EncryptedVM::Fft64AvxRayon(_) => Backend::Fft64AvxRayon,
        }
    }

    pub fn execute(&mut self) {
        dispatch!(self, vm => vm.execute())
    }

    pub fn output_tape(&mut self) -> Vec<u8> {
        dispatch!(self, vm => vm.output_tape())
    }
}

/// Phantom VM: Encrypted Risc-v
pub struct Phantom {
    boot_rom: BootMemory,
    boot_ram: BootMemory,
    output_info: OutputInfo,
    input_info: InputInfo,
    _elf_bytes: Option<Vec<u8>>,
}

impl Phantom {
    pub fn from_elf(elf_bytes: Vec<u8>) -> Self {
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
        // macros::verbose_println!("ROM SIZE: {} bytes", txthdr.p_memsz);

        // load all +r/+rw headers
        let hdrs: Vec<&ProgramHeader> = phdrs
            .iter()
            .filter(|p| p.p_flags == PF_R || p.p_flags == PF_R + PF_W)
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
            _elf_bytes: Some(elf_bytes),
        }
    }

    pub fn boot_rom(&self) -> &BootMemory {
        &self.boot_rom
    }

    pub fn boot_ram(&self) -> &BootMemory {
        &self.boot_ram
    }

    pub fn input_info(&self) -> &InputInfo {
        &self.input_info
    }

    pub fn output_info(&self) -> &OutputInfo {
        &self.output_info
    }

    /// Encrypts the program and `input_tape` on the backend and with the
    /// options read from the environment (see [`RunOptions::from_env`]).
    pub fn encrypted_vm(&self, input_tape: &[u8], max_cycles: usize) -> EncryptedVM {
        self.encrypted_vm_with(input_tape, max_cycles, RunOptions::from_env())
    }

    /// Encrypts the program and `input_tape` with explicit options.
    pub fn encrypted_vm_with(
        &self,
        input_tape: &[u8],
        max_cycles: usize,
        options: RunOptions,
    ) -> EncryptedVM {
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

        // // setup RAM
        let ram_offset: usize = self.boot_ram.offset;
        let mut ram_with_input: Vec<u8> = self.boot_ram.data.clone();
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
            .collect::<Vec<u32>>();

        macros::verbose_println!(
            "Phantom backend: {} ({} threads)",
            options.backend.name(),
            options.threads
        );

        match options.backend {
            Backend::Fft64Ref => EncryptedVM::Fft64Ref(Box::new(EncryptedVMImpl::new(
                self,
                &parser,
                &ram_data_u32,
                max_cycles,
                options,
            ))),
            #[cfg(all(
                target_arch = "x86_64",
                target_feature = "avx2",
                target_feature = "fma"
            ))]
            Backend::Fft64Avx => EncryptedVM::Fft64Avx(Box::new(EncryptedVMImpl::new(
                self,
                &parser,
                &ram_data_u32,
                max_cycles,
                options,
            ))),
            #[cfg(all(
                target_arch = "x86_64",
                target_feature = "avx2",
                target_feature = "fma"
            ))]
            Backend::Fft64AvxRayon => {
                // poulpy's rayon backend schedules on the global pool; size it
                // once to the requested thread count (a later call is a no-op).
                let _ = rayon::ThreadPoolBuilder::new()
                    .num_threads(options.threads)
                    .build_global();
                EncryptedVM::Fft64AvxRayon(Box::new(EncryptedVMImpl::new(
                    self,
                    &parser,
                    &ram_data_u32,
                    max_cycles,
                    options,
                )))
            }
            #[cfg(not(all(
                target_arch = "x86_64",
                target_feature = "avx2",
                target_feature = "fma"
            )))]
            other => panic!(
                "{} is not available in this build (no AVX2/FMA)",
                other.name()
            ),
        }
    }

    pub fn test_vm(&self, max_cycles: usize) -> TestVM {
        TestVM::init(
            &self.boot_rom,
            &self.boot_ram,
            &self.input_info,
            &self.output_info,
            max_cycles,
        )
    }
}
