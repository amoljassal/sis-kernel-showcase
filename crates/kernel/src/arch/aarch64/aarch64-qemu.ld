ENTRY(_start)

PHDRS
{
  text PT_LOAD FLAGS((1 << 0) | (1 << 2)); /* R | X */
  data PT_LOAD FLAGS((1 << 0) | (1 << 1)); /* R | W */
}

SECTIONS
{
  /* QEMU virt base address for Image */
  __image_base = 0x40000000;
  . = __image_base;

  /* Place 64-byte Linux Image header at start */
  .head : ALIGN(0x1000) {
    KEEP(*(.text._head))
  } :text

  /* Place code at image_base + 0x80000 as per text_offset */
  . = __image_base + 0x00080000;
  .text : ALIGN(0x1000) {
    KEEP(*(.text._start))
    *(.text*)
  } :text

  .rodata : ALIGN(0x1000) {
    *(.rodata*)
  } :text

  .data : ALIGN(0x1000) {
    *(.data*)
  } :data

  .bss : ALIGN(0x1000) {
    *(.bss*)
    *(COMMON)
  } :data
}

