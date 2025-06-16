OUTPUT_ARCH(riscv)
ENTRY(_start)

STACK_SIZE = 16K;

MEMORY
{
  RAM (rwx) : ORIGIN = 0x80000000, LENGTH = 128M
}

SECTIONS
{
    . = ORIGIN(RAM);

    .text : {
        KEEP(*(.init))
        *(.text .text.*)
    } >RAM

    .rodata : {
        *(.rodata .rodata.*)
    } >RAM

    .data : {
        . = ALIGN(8);
        PROVIDE(_global_pointer = .);
        *(.data .data.*)
    } >RAM

    .bss : {
        . = ALIGN(8);
        PROVIDE(_bss_start = .);
        *(.bss .bss.*)
        PROVIDE(_bss_end = .);
    } >RAM

    . = . + STACK_SIZE;
    . = ALIGN(8);
    PROVIDE(_stack_top = .);
}