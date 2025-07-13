.option norvc

.equ TRAP_FRAME_SIZE, 288

.altmacro
.macro save_context
    # 32 GPRs + 4 CSRs = 36 registers. 36 * 8 bytes = 288 bytes.
    addi sp, sp, -TRAP_FRAME_SIZE

    # gprs
    sd   ra, 1*8(sp)    # x1
    sd   t0, 5*8(sp)    # x5 t0 stored prior the order to reuse it for sscratch

    csrr t0, sscratch   # save original sp (x2) from sscratch
    sd   t0, 2*8(sp)    # x2

    sd   gp, 3*8(sp)    # x3
    sd   tp, 4*8(sp)    # x4
    sd   t1, 6*8(sp)    # x6
    sd   t2, 7*8(sp)    # x7
    sd   s0, 8*8(sp)    # x8
    sd   s1, 9*8(sp)    # x9
    sd   a0, 10*8(sp)   # x10
    sd   a1, 11*8(sp)   # x11
    sd   a2, 12*8(sp)   # x12
    sd   a3, 13*8(sp)   # x13
    sd   a4, 14*8(sp)   # x14
    sd   a5, 15*8(sp)   # x15
    sd   a6, 16*8(sp)   # x16
    sd   a7, 17*8(sp)   # x17
    sd   s2, 18*8(sp)   # x18
    sd   s3, 19*8(sp)   # x19
    sd   s4, 20*8(sp)   # x20
    sd   s5, 21*8(sp)   # x21
    sd   s6, 22*8(sp)   # x22
    sd   s7, 23*8(sp)   # x23
    sd   s8, 24*8(sp)   # x24
    sd   s9, 25*8(sp)   # x25
    sd   s10, 26*8(sp)  # x26
    sd   s11, 27*8(sp)  # x27
    sd   t3, 28*8(sp)   # x28
    sd   t4, 29*8(sp)   # x29
    sd   t5, 30*8(sp)   # x30
    sd   t6, 31*8(sp)   # x31

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
    ld   ra, 1*8(sp)    # x1
                        # x2 skipped as sp will be resotred from sscratch
    ld   gp, 3*8(sp)    # x3
    ld   tp, 4*8(sp)    # x4
    ld   t0, 5*8(sp)    # x5
    ld   t1, 6*8(sp)    # x6
    ld   t2, 7*8(sp)    # x7
    ld   s0, 8*8(sp)    # x8
    ld   s1, 9*8(sp)    # x9
    ld   a0, 10*8(sp)   # x10
    ld   a1, 11*8(sp)   # x11
    ld   a2, 12*8(sp)   # x12
    ld   a3, 13*8(sp)   # x13
    ld   a4, 14*8(sp)   # x14
    ld   a5, 15*8(sp)   # x15
    ld   a6, 16*8(sp)   # x16
    ld   a7, 17*8(sp)   # x17
    ld   s2, 18*8(sp)   # x18
    ld   s3, 19*8(sp)   # x19
    ld   s4, 20*8(sp)   # x20
    ld   s5, 21*8(sp)   # x21
    ld   s6, 22*8(sp)   # x22
    ld   s7, 23*8(sp)   # x23
    ld   s8, 24*8(sp)   # x24
    ld   s9, 25*8(sp)   # x25
    ld   s10, 26*8(sp)  # x26
    ld   s11, 27*8(sp)  # x27
    ld   t3, 28*8(sp)   # x28
    ld   t4, 29*8(sp)   # x29
    ld   t5, 30*8(sp)   # x30
    ld   t6, 31*8(sp)   # x31

    # deallocate trap frame
    addi sp, sp, TRAP_FRAME_SIZE
.endm

.section .text
.global alltraps
.align 2

alltraps:

    # swap sp <> sscratch
    csrrw sp, sscratch, sp
    save_context

    mv      a0, sp
    call    trap_handler

    restore_context
    # swap back to restore original sp
    csrrw sp, sscratch, sp

    sret
