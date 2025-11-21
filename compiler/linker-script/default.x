ENTRY(_start)

MEMORY
{
    /* Define different memory regions for code and data memory */
    FLASH (rx)  : ORIGIN = 0x00000000, LENGTH = 256K  /* Code memory */
    RAM   (rwx) : ORIGIN = 0x40000, LENGTH = 4K /* Data memory */
}

SECTIONS
{
  /* stack pointer is set to max memory limit */
  _stack_start = ORIGIN(RAM) + LENGTH(RAM);

  .text : ALIGN(4)
  {
    KEEP(*(.init));
    . = ALIGN(4);
    KEEP(*(.init.rust));
    *(.text .text.*);

    . = ALIGN(4);
  } > FLASH AT> FLASH

  .inpdata : ALIGN(4)
  {
    KEEP(*(.inpdata))
  } > RAM AT> RAM

  .outdata : ALIGN(4)
  {
    KEEP(*(.outdata))
  } > RAM AT> RAM

  .rodata : ALIGN(4)
  {
     . = ALIGN(4);

    *(.srodata .srodata.*);
    *(.rodata .rodata.*);

    /* ${ARCH_WIDTH}-byte align the end (VMA) of this section.
       This is required by LLD to ensure the LMA of the following .data
       section will have the correct alignment. */
    . = ALIGN(4);
  } > RAM AT> RAM

  .data : ALIGN(4)
  {
    . = ALIGN(4);

    /* Must be called __global_pointer$ for linker relaxations to work.

       Must point to address where .sdata starts offset by 2048 bytes
       for best optimisation

       ref: https://gnu-mcu-eclipse.github.io/arch/riscv/programmer/#the-gp-global-pointer-register
       ref: https://groups.google.com/a/groups.riscv.org/g/sw-dev/c/60IdaZj27dY
     */
    PROVIDE(__global_pointer$ = . + 0x800);
    *(.sdata .sdata.* .sdata2 .sdata2.*);
    *(.data .data.*);
  } > RAM AT> RAM

  .bss (NOLOAD) : ALIGN(4)
  {
    . = ALIGN(4);
    *(.sbss .sbss.* .bss .bss.*);
    _end = .;
  } > RAM

  /* fake output .got section */
  /* Dynamic relocations are unsupported. This section is only used to detect
     relocatable code in the input files and raise an error if relocatable code
     is found */
  .got (INFO) :
  {
    KEEP(*(.got .got.*));
  }

  /DISCARD/ :
  {
    *(.comment)
    *(.comment.*)
    *(.debug)
    *(.debug.*)
    *(.eh_frame)
    *(.eh_frame.*)
    *(.eh_frame_hdr)
    *(.eh_frame_hdr.*)
    *(.riscv.attributes)
    *(.riscv.attributes.*)
  }
}

/* Do not exceed this mark in the error messages above                                    | */

/* # Other checks */
ASSERT(SIZEOF(.got) == 0, "
ERROR(riscv-rt): .got section detected in the input files. Dynamic relocations are not
supported. If you are linking to C code compiled using the `cc` crate then modify your
build script to compile the C code _without_ the -fPIC flag. See the documentation of
the `cc::Build.pic` method for details.");

/* Do not exceed this mark in the error messages above                                    | */
