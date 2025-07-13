# S-Mode trap handling

.option norvc

# Trap Frame Layout
.equ TRAP_FRAME_RA,      1*8
.equ TRAP_FRAME_SP,      2*8
.equ TRAP_FRAME_GP,      3*8
.equ TRAP_FRAME_TP,      4*8
.equ TRAP_FRAME_T0,      5*8
.equ TRAP_FRAME_T1,      6*8
.equ TRAP_FRAME_T2,      7*8
.equ TRAP_FRAME_S0,      8*8
.equ TRAP_FRAME_S1,      9*8
.equ TRAP_FRAME_A0,      10*8
.equ TRAP_FRAME_A1,      11*8
.equ TRAP_FRAME_A2,      12*8
.equ TRAP_FRAME_A3,      13*8
.equ TRAP_FRAME_A4,      14*8
.equ TRAP_FRAME_A5,      15*8
.equ TRAP_FRAME_A6,      16*8
.equ TRAP_FRAME_A7,      17*8
.equ TRAP_FRAME_S2,      18*8
.equ TRAP_FRAME_S3,      19*8
.equ TRAP_FRAME_S4,      20*8
.equ TRAP_FRAME_S5,      21*8
.equ TRAP_FRAME_S6,      22*8
.equ TRAP_FRAME_S7,      23*8
.equ TRAP_FRAME_S8,      24*8
.equ TRAP_FRAME_S9,      25*8
.equ TRAP_FRAME_S10,     26*8
.equ TRAP_FRAME_S11,     27*8
.equ TRAP_FRAME_T3,      28*8
.equ TRAP_FRAME_T4,      29*8
.equ TRAP_FRAME_T5,      30*8
.equ TRAP_FRAME_T6,      31*8
.equ TRAP_FRAME_SSTATUS, 32*8
.equ TRAP_FRAME_SEPC,    33*8
.equ TRAP_FRAME_STVAL,   34*8
.equ TRAP_FRAME_SCAUSE,  35*8
.equ TRAP_FRAME_SIZE,    36*8

.altmacro
.macro save_context
    # 32 GPRs + 4 CSRs = 36 registers. 36 * 8 bytes = 288 bytes.
    addi sp, sp, -TRAP_FRAME_SIZE

    # gprs
    sd   ra, TRAP_FRAME_RA(sp)      # x1
    sd   t0, TRAP_FRAME_T0(sp)      # x5 t0 stored prior the order to reuse it for sscratch

    csrr t0, sscratch               # save original sp (x2) from sscratch
    sd   t0, TRAP_FRAME_SP(sp)      # x2

    sd   gp, TRAP_FRAME_GP(sp)      # x3
    sd   tp, TRAP_FRAME_TP(sp)      # x4
    sd   t1, TRAP_FRAME_T1(sp)      # x6
    sd   t2, TRAP_FRAME_T2(sp)      # x7
    sd   s0, TRAP_FRAME_S0(sp)      # x8
    sd   s1, TRAP_FRAME_S1(sp)      # x9
    sd   a0, TRAP_FRAME_A0(sp)      # x10
    sd   a1, TRAP_FRAME_A1(sp)      # x11
    sd   a2, TRAP_FRAME_A2(sp)      # x12
    sd   a3, TRAP_FRAME_A3(sp)      # x13
    sd   a4, TRAP_FRAME_A4(sp)      # x14
    sd   a5, TRAP_FRAME_A5(sp)      # x15
    sd   a6, TRAP_FRAME_A6(sp)      # x16
    sd   a7, TRAP_FRAME_A7(sp)      # x17
    sd   s2, TRAP_FRAME_S2(sp)      # x18
    sd   s3, TRAP_FRAME_S3(sp)      # x19
    sd   s4, TRAP_FRAME_S4(sp)      # x20
    sd   s5, TRAP_FRAME_S5(sp)      # x21
    sd   s6, TRAP_FRAME_S6(sp)      # x22
    sd   s7, TRAP_FRAME_S7(sp)      # x23
    sd   s8, TRAP_FRAME_S8(sp)      # x24
    sd   s9, TRAP_FRAME_S9(sp)      # x25
    sd   s10, TRAP_FRAME_S10(sp)    # x26
    sd   s11, TRAP_FRAME_S11(sp)    # x27
    sd   t3, TRAP_FRAME_T3(sp)      # x28
    sd   t4, TRAP_FRAME_T4(sp)      # x29
    sd   t5, TRAP_FRAME_T5(sp)      # x30
    sd   t6, TRAP_FRAME_T6(sp)      # x31

    # csrs
    csrr t0, sstatus
    sd   t0, TRAP_FRAME_SSTATUS(sp)

    csrr t0, sepc
    sd   t0, TRAP_FRAME_SEPC(sp)

    csrr t0, stval
    sd   t0, TRAP_FRAME_STVAL(sp)

    csrr t0, scause
    sd   t0, TRAP_FRAME_SCAUSE(sp)
.endm

.macro restore_context
    # `stval` and `scause` are informational and do not need to be restored.
    ld   t0, TRAP_FRAME_SSTATUS(sp)
    csrw sstatus, t0

    ld   t0, TRAP_FRAME_SEPC(sp)
    csrw sepc, t0

    # gprs
    ld   ra, TRAP_FRAME_RA(sp)      # x1
                                    # x2 skipped as sp will be resotred from sscratch
    ld   gp, TRAP_FRAME_GP(sp)      # x3
    ld   tp, TRAP_FRAME_TP(sp)      # x4
    ld   t0, TRAP_FRAME_T0(sp)      # x5
    ld   t1, TRAP_FRAME_T1(sp)      # x6
    ld   t2, TRAP_FRAME_T2(sp)      # x7
    ld   s0, TRAP_FRAME_S0(sp)      # x8
    ld   s1, TRAP_FRAME_S1(sp)      # x9
    ld   a0, TRAP_FRAME_A0(sp)      # x10
    ld   a1, TRAP_FRAME_A1(sp)      # x11
    ld   a2, TRAP_FRAME_A2(sp)      # x12
    ld   a3, TRAP_FRAME_A3(sp)      # x13
    ld   a4, TRAP_FRAME_A4(sp)      # x14
    ld   a5, TRAP_FRAME_A5(sp)      # x15
    ld   a6, TRAP_FRAME_A6(sp)      # x16
    ld   a7, TRAP_FRAME_A7(sp)      # x17
    ld   s2, TRAP_FRAME_S2(sp)      # x18
    ld   s3, TRAP_FRAME_S3(sp)      # x19
    ld   s4, TRAP_FRAME_S4(sp)      # x20
    ld   s5, TRAP_FRAME_S5(sp)      # x21
    ld   s6, TRAP_FRAME_S6(sp)      # x22
    ld   s7, TRAP_FRAME_S7(sp)      # x23
    ld   s8, TRAP_FRAME_S8(sp)      # x24
    ld   s9, TRAP_FRAME_S9(sp)      # x25
    ld   s10, TRAP_FRAME_S10(sp)    # x26
    ld   s11, TRAP_FRAME_S11(sp)    # x27
    ld   t3, TRAP_FRAME_T3(sp)      # x28
    ld   t4, TRAP_FRAME_T4(sp)      # x29
    ld   t5, TRAP_FRAME_T5(sp)      # x30
    ld   t6, TRAP_FRAME_T6(sp)      # x31

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
