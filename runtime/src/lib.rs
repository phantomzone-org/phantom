#![no_std]

use core::alloc::{GlobalAlloc, Layout};

mod alloc;

// RISC-V assembly to start the program

// _start section
//     1. sets global pointer in gp register. __global_pointer$ symbol is declared in linker script (default.x)
//     2. sets the stack pointer in sp register and fp register. stack pointer is stored in _stack_start symbol in linker script.
//     2. "blez x10, run" jumps to `run` section if x10 register is zero. Nexus's implemntation calls stack size before calling
//         this line. I haven't replicated it here because I didn't know why calling stack size in necessary in the first place.
//         But it'll definitely cause some issue so beware.
// run section:
//     1. directly jumps to "_start_rust" symbol.
//     2. Notice that "_start_rust" is linked to the "main" function defined in "/guest/src/main"
//
core::arch::global_asm!(
    ".option nopic
    .section .init, \"ax\"
    .global _start

_start:
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop

    la sp, _stack_start
    mv fp, sp

    blez x10, run

    unimp

run: 
    jal ra, _start_rust
    unimp

    "
);

#[cfg(target_arch = "riscv32")]

pub fn println(v: &str) {
    let v_addr = v.as_ptr().addr();
    let v_len = v.len();
    unsafe {
        core::arch::asm!("ecall", in("a0") v_addr, in("a1") v_len);
    }
}

/// Expose "_start_rust" symbol in binary.
///
/// The function calls "main" function declared in "guest/src/main"
#[doc(hidden)]
#[link_section = ".init.rust"]
#[export_name = "_start_rust"]
pub unsafe extern "C" fn start_rust(a0: u32, a1: u32, a2: u32) -> u32 {
    extern "Rust" {
        fn main(a0: u32, a1: u32, a2: u32) -> u32;
    }
    main(a0, a1, a2)
}

/// Following code is taken from nexus-zkvm which itself refers to risc-v rt

#[export_name = "error: rt appears more than once"]
#[doc(hidden)]
pub static __ONCE__: () = ();

struct Heap;

#[global_allocator]
static HEAP: Heap = Heap;

// This trivial allocate will always expand the heap, and never
// deallocates. This should be fine for small programs.

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        alloc::sys_alloc_aligned(layout.size(), layout.align())
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}
