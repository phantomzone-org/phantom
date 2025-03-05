use compiler::{interpreter::TestVM, CompileOpts};

fn main() {
    let compiler = CompileOpts::new("guest");
    let elf_bytes = compiler.build();
    let mut vm = TestVM::init(elf_bytes);

    while vm.is_exec() {
        vm.run();
    }
}
