/* RISC-V Linker Script for SIS Kernel
 * 
 * Research-backed memory layout following RISC-V conventions and optimization studies.
 * Designed for QEMU virt machine and future Vikram 3201 hardware support.
 *
 * Memory Layout (based on RISC-V conventions):
 * 0x8000_0000 - Kernel load address (standard RISC-V)
 * 0x8020_0000 - Kernel stack region  
 * 0x8040_0000 - Kernel heap region
 * 0xC000_0000 - Device MMIO region (3GB mark)
 *
 * Research Basis:
 * - RISC-V memory layout conventions
 * - QEMU virt machine memory map
 * - Cache-aligned section optimization
 */

OUTPUT_ARCH(riscv)
ENTRY(_start)

/* Memory regions for QEMU virt machine */
MEMORY
{
    /* Main RAM starts at 2GB (0x8000_0000) in QEMU virt */
    RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 128M
}

/* Section definitions */
SECTIONS
{
    /* Kernel load address - standard RISC-V location */
    . = 0x80000000;
    
    /* Boot section - must be first for proper execution */
    .text.boot : {
        *(.text.boot)
        . = ALIGN(4096);  /* Page-aligned for MMU setup */
    } >RAM
    
    /* Main text section - executable code */
    .text : {
        *(.text .text.*)
        . = ALIGN(4096);  /* Page boundary for better cache performance */
    } >RAM
    
    /* Read-only data */
    .rodata : {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)  /* Small read-only data */
        . = ALIGN(4096);
    } >RAM
    
    /* Initialized data */
    .data : {
        *(.data .data.*)
        *(.sdata .sdata.*)      /* Small data section */
        . = ALIGN(8);           /* 8-byte alignment for 64-bit */
    } >RAM
    
    /* BSS section - zero-initialized data */
    .bss : {
        _bss_start = .;
        *(.bss .bss.*)
        *(.sbss .sbss.*)        /* Small BSS section */
        . = ALIGN(8);
        _bss_end = .;
    } >RAM
    
    /* Exception/interrupt vector table (if needed) */
    .vectors : {
        . = ALIGN(4096);        /* Page-aligned for isolation */
        *(.vectors)
    } >RAM
    
    /* Stack region allocation */
    . = ALIGN(4096);
    _stack_start = .;
    . += 0x20000;               /* 128KB total stack space */
    _stack_top = .;
    
    /* Heap region starts after stack */
    . = ALIGN(4096);
    _heap_start = .;
    . += 0x1000000;             /* 16MB heap space */
    _heap_end = .;
    
    /* Kernel end marker */
    _kernel_end = .;
    
    /* Device tree blob space (reserved) */
    . = ALIGN(4096);
    _dtb_space = .;
    . += 0x10000;               /* 64KB for device tree */
    
    /* Discard sections we don't need */
    /DISCARD/ : {
        *(.comment)
        *(.debug*)
        *(.note*)
        *(.eh_frame*)
    }
}

/* Symbols for kernel memory management */
_text_start = 0x80000000;
_text_end = ADDR(.rodata);
_data_start = ADDR(.data);
_data_end = ADDR(.bss);

/* Memory layout validation */
ASSERT(_stack_top < 0x82000000, "Stack region exceeds 32MB")
ASSERT(_heap_end < 0x88000000, "Heap region exceeds reasonable bounds")
ASSERT(_kernel_end < 0x90000000, "Kernel size exceeds 256MB")