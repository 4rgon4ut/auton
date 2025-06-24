# src/asm/trap.S
.section .text
.global trap_handler


trap_handler:

    # # TODO
    # mv      a0, sp
    # call    rust_supervisor_trap_handler

    sret