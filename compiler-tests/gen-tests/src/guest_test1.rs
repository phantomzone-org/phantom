use compiler::{CompileOpts, Phantom};

use crate::{to_u8_slice, write_test_cases};

#[repr(C)]
struct Output {
    out: u32,
}

#[repr(C)]
struct Input {
    in_a: u32,
    branch: bool,
}

#[no_mangle]
static HARDCODED_VALUE: [u32; 4] = [10, 20, 30, 40];

// Exact replica of guest-test1 program
fn run(input: &Input) -> Output {
    let in_a = input.in_a;

    let out = if input.branch {
        HARDCODED_VALUE.iter().fold(0, |acc, v| acc + in_a + *v)
    } else {
        HARDCODED_VALUE.iter().fold(0, |acc, v| acc + (in_a * *v))
    };

    Output { out }
}

/// Some test cases
fn test_cases() -> Vec<(Vec<u8>, Vec<u8>)> {
    let mut cases = vec![];
    for in_a in [10, 20, 30] {
        for branch in [true, false] {
            let input = Input { in_a, branch };
            let output = run(&input);
            cases.push((to_u8_slice(&input).to_vec(), to_u8_slice(&output).to_vec()));
        }
    }
    cases
}

pub fn write_guest_test1(file_path: &str) -> std::io::Result<()> {
    let compiler = CompileOpts::new("guest-test1");
    let elf_bytes = compiler.build();
    let phantom = Phantom::from_elf(elf_bytes);
    let test_cases = test_cases();
    write_test_cases(&phantom, test_cases.as_slice(), file_path)
}
