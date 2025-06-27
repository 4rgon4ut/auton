# RISC-V Supervisor Trap Handler with Full Context Save/Restore

.option norvc

.altmacro
.macro save_context
    # 32 GPRs + 4 CSRs = 36 registers. 36 * 8 bytes = 288 bytes.
    addi sp, sp, -288

    # gprs
    sd   ra, 0*8(sp)    # x1
    sd   gp, 2*8(sp)    # x3
    sd   tp, 3*8(sp)    # x4
    sd   t0, 4*8(sp)    # x5
    sd   t1, 5*8(sp)    # x6
    sd   t2, 6*8(sp)    # x7
    sd   s0, 7*8(sp)    # x8
    sd   s1, 8*8(sp)    # x9
    sd   a0, 9*8(sp)    # x10
    sd   a1, 10*8(sp)   # x11
    sd   a2, 11*8(sp)   # x12
    sd   a3, 12*8(sp)   # x13
    sd   a4, 13*8(sp)   # x14
    sd   a5, 14*8(sp)   # x15
    sd   a6, 15*8(sp)   # x16
    sd   a7, 16*8(sp)   # x17
    sd   s2, 17*8(sp)   # x18
    sd   s3, 18*8(sp)   # x19
    sd   s4, 19*8(sp)   # x20
    sd   s5, 20*8(sp)   # x21
    sd   s6, 21*8(sp)   # x22
    sd   s7, 22*8(sp)   # x23
    sd   s8, 23*8(sp)   # x24
    sd   s9, 24*8(sp)   # x25
    sd   s10, 25*8(sp)  # x26
    sd   s11, 26*8(sp)  # x27
    sd   t3, 27*8(sp)   # x28
    sd   t4, 28*8(sp)   # x29
    sd   t5, 29*8(sp)   # x30
    sd   t6, 30*8(sp)   # x31

    # original stack pointer
    addi t0, sp, 320
    sd   t0, 1*8(sp)

    # csrs
    csrr t0, sstatus
    sd   t0, 32*8(sp)
    csrr t0, sepc
    sd   t0, 33*8(sp)
    csrr t0, stval
    sd   t0, 34*8(sp)
    csrr t0, scause
    sd   t0, 35*8(sp)
.endm

.macro restore_context
    # `stval` and `scause` are informational and do not need to be restored.
    ld   t0, 32*8(sp)
    csrw sstatus, t0
    ld   t0, 33*8(sp)
    csrw sepc, t0

    # gprs
    ld   ra, 0*8(sp)
    ld   gp, 2*8(sp)
    ld   tp, 3*8(sp)
    ld   t0, 4*8(sp)
    ld   t1, 5*8(sp)
    ld   t2, 6*8(sp)
    ld   s0, 7*8(sp)
    ld   s1, 8*8(sp)
    ld   a0, 9*8(sp)
    ld   a1, 10*8(sp)
    ld   a2, 11*8(sp)
    ld   a3, 12*8(sp)
    ld   a4, 13*8(sp)
    ld   a5, 14*8(sp)
    ld   a6, 15*8(sp)
    ld   a7, 16*8(sp)
    ld   s2, 17*8(sp)
    ld   s3, 18*8(sp)
    ld   s4, 19*8(sp)
    ld   s5, 20*8(sp)
    ld   s6, 21*8(sp)
    ld   s7, 22*8(sp)
    ld   s8, 23*8(sp)
    ld   s9, 24*8(sp)
    ld   s10, 25*8(sp)
    ld   s11, 26*8(sp)
    ld   t3, 27*8(sp)
    ld   t4, 28*8(sp)
    ld   t5, 29*8(sp)
    ld   t6, 30*8(sp)

    # restore the original stack pointer, deallocating the trap frame
    ld   sp, 1*8(sp)
.endm

.section .text
.global alltraps
.align 2

alltraps:
    save_context

    mv      a0, sp
    call    trap_handler

    restore_context

    sret
