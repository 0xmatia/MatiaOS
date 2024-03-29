/*
 * File: linker.ld
 * Project: RpiOS
 * File Created: Tuesday, 26th October 2021 5:49:15 pm
 * Author: Elad Matia (elad.matia@gmail.com)
 */

kernel_addr_in_memory = 0x80000;
dram_start_addr       = 0x0;

ENTRY(kernel_addr_in_memory)

/*
* Segments are marked PT_LOAD below so that the ELF file provides virtual and physical addresses.
* It doesn't mean all of them need actually be loaded. 
*/

PHDRS
{
    segment_boot_core_stack PT_LOAD FLAGS(6); /* 6 == RW */
    segment_code            PT_LOAD FLAGS(5); /* 5 == RX */
    segment_data            PT_LOAD FLAGS(6); /* 6 == RW */
}

SECTIONS
{
    . = dram_start_addr;

    /*
    This is the "start" of the stack section for the boot core.
    It grows downwards (0x7999-0x0000) + 0x8000
    */
    .boot_core_stack (NOLOAD) : 
    {
       . += kernel_addr_in_memory; 
        __boot_core_stack_end_exclusive = .;
    } :segment_boot_core_stack

    /* Code section. */
	__binary_start = .;
    .text :
    {
        KEEP(*(.text._start)) /* found in boot.s */
        *(.text._start_arguments) /* constants */
        *(.text._start_rust)  /* rust entry point*/
        *(.text*) /* the rest */
    } :segment_code

    .rodata : ALIGN(8) { *(.rodata*) } :segment_code

    .got : ALIGN(8) { *(.got) } :segment_code

    .data : { *(.data*) } :segment_data

	. = ALIGN(8);

	
    /* bsses */
    .bss (NOLOAD) : ALIGN(16)
    {
        __bss_start = .;
        *(.bss*)
        . = ALIGN(16);
        __bss_end_exclusive = .;
    } :segment_data
}
