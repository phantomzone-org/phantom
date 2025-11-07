use std::{
    fs,
    io::{self, Write},
    path::Path,
    process::Command,
};

pub mod interpreter;

pub use interpreter::Phantom;

pub struct CompileOpts {
    program: String,
}

impl CompileOpts {
    /// Pass `guest` string to the compiler.
    pub fn new(program: &str) -> CompileOpts {
        CompileOpts {
            program: program.to_string(),
        }
    }

    pub fn build(&self) -> Vec<u8> {
        // set compilation target to riscv32i
        let target = "riscv32i-unknown-none-elf";
        let profile = "release";

        // Direct path to linker file for the rust compiler
        let linker_path = {
            let manifest_dir = env!("CARGO_MANIFEST_DIR");
            fs::canonicalize(Path::new(manifest_dir).join("linker-script/default.x")).unwrap()
        };

        let rust_flags = [
            "-C",
            // specify the linker path
            &format!("link-arg=-T{}", linker_path.to_str().unwrap()),
            "-C",
            "panic=abort",
        ];
        let envs = vec![("CARGO_ENCODED_RUSTFLAGS", rust_flags.join("\x1f"))];

        // Destination for outputs
        let destination = "/tmp/vm-experiments";

        let cargo_bin = std::env::var("CARGO").unwrap();
        let mut cmd = Command::new(cargo_bin);
        // Compile /guest/main using cargo
        cmd.envs(envs).args([
            "build",
            "--profile",
            profile,
            "--target",
            target,
            "--package",
            self.program.as_str(),
            "--bin",
            self.program.as_str(),
            "--target-dir",
            destination,
            // "--verbose",
        ]);
        let out = cmd.output().unwrap();

        if !out.status.success() {
            io::stderr().write_all(&out.stderr).unwrap();
            panic!("Compilation failed!")
        }

        // Compilation succeded
        // Read the ELF file from destination: /tmp/vm-experiments
        let elf_path = format!(
            "{}/{}/{}/{}",
            destination,
            target,
            profile,
            self.program.as_str()
        );
        let elf_data = std::fs::read(std::path::Path::new(&elf_path)).unwrap();
        elf_data
    }
}
