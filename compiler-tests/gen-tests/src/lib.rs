use compiler::Phantom;
use quote::quote;
use std::fs::File;
use std::io::Write;
use std::process::Command;

mod guest_test1;

pub use guest_test1::*;

fn to_u8_slice<T>(v: &T) -> &[u8] {
    unsafe { core::slice::from_raw_parts((v as *const T) as *const u8, core::mem::size_of::<T>()) }
}

fn from_u8_slice<T>(v: &[u8]) -> T {
    unsafe { std::ptr::read(v.as_ptr() as *const T) }.into()
}

/// Generates Rust code with static constants for the PhantomTest struct fields
fn codegen_test_cases(phantom: &Phantom, test_cases: &[(Vec<u8>, Vec<u8>)]) -> String {
    let boot_rom_offset = phantom.boot_rom().offset();
    let boot_rom_size = phantom.boot_rom().size();
    let boot_rom_data = phantom.boot_rom().data();
    let boot_rom_data_len = boot_rom_data.len();

    let boot_ram_offset = phantom.boot_ram().offset();
    let boot_ram_size = phantom.boot_ram().size();
    let boot_ram_data = phantom.boot_ram().data();
    let boot_ram_data_len = boot_ram_data.len();

    let input_start_addr = phantom.input_info().start_addr();
    let input_size = phantom.input_info().size();

    let output_start_addr = phantom.output_info().start_addr();
    let output_size = phantom.output_info().size();

    // Convert Vec<u8> to array literals for quote
    let boot_rom_bytes = boot_rom_data.iter().map(|&b| b as u8);
    let boot_ram_bytes = boot_ram_data.iter().map(|&b| b as u8);

    // Handle test_cases
    let test_cases_len = test_cases.len();

    // Generate test case data as nested arrays
    let test_case_inputs: Vec<_> = test_cases
        .iter()
        .map(|(input, _)| {
            let input_bytes = input.iter().map(|&b| b as u8);
            quote! { [#(#input_bytes),*] }
        })
        .collect();

    let test_case_outputs: Vec<_> = test_cases
        .iter()
        .map(|(_, output)| {
            let output_bytes = output.iter().map(|&b| b as u8);
            quote! { [#(#output_bytes),*] }
        })
        .collect();

    let tokens = quote! {
        // Auto-generated from Phantom struct and test cases

        // Boot ROM constants
        pub const BOOT_ROM_OFFSET: usize = #boot_rom_offset;
        pub const BOOT_ROM_SIZE: usize = #boot_rom_size;
        pub const BOOT_ROM_DATA: [u8; #boot_rom_data_len] = [
            #(#boot_rom_bytes),*
        ];

        // Boot RAM constants
        pub const BOOT_RAM_OFFSET: usize = #boot_ram_offset;
        pub const BOOT_RAM_SIZE: usize = #boot_ram_size;
        pub const BOOT_RAM_DATA: [u8; #boot_ram_data_len] = [
            #(#boot_ram_bytes),*
        ];

        // Input Info constants
        pub const INPUT_START_ADDR: usize = #input_start_addr;
        pub const INPUT_SIZE: usize = #input_size;

        // Output Info constants
        pub const OUTPUT_START_ADDR: usize = #output_start_addr;
        pub const OUTPUT_SIZE: usize = #output_size;

        // Test Cases constants
        pub const TEST_CASES_COUNT: usize = #test_cases_len;

        // Test case input data
        pub const TEST_CASE_INPUTS: &[&[u8]] = &[
            #(&(#test_case_inputs)),*
        ];

        // Test case output data
        pub const TEST_CASE_OUTPUTS: &[&[u8]] = &[
            #(&(#test_case_outputs)),*
        ];

        // Helper function to get all test cases as tuples
        pub fn get_test_cases() -> Vec<(&'static [u8], &'static [u8])> {
            TEST_CASE_INPUTS.iter()
                .zip(TEST_CASE_OUTPUTS.iter())
                .map(|(&input, &output)| (input, output))
                .collect()
        }
    };

    tokens.to_string()
}

fn write_test_cases(
    phantom: &Phantom,
    test_cases: &[(Vec<u8>, Vec<u8>)],
    file_path: &str,
) -> std::io::Result<()> {
    let code = codegen_test_cases(phantom, test_cases);
    let mut file = File::create(file_path)?;
    file.write_all(code.as_bytes())?;

    // Format the generated file with rustfmt
    let _output = Command::new("rustfmt").arg(file_path).output();

    Ok(())
}
