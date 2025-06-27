.altmacro
.macro save_gprs
    # Allocate space on the stack for the trap frame.
    # 32 registers * 8 bytes/register = 256 bytes.
    # This maintains the required 16-byte stack alignment.
    addi sp, sp, -256

    # Save all gprs except x0 (zero).
    sd   ra, 0*8(sp)    # x1
    # x2 (sp) is handled separately below
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

    # Save the original stack pointer. We calculate its value *before*
    # we allocated the trap frame and store it in the x2 slot.
    addi t0, sp, 256
    sd   t0, 1*8(sp)
.endm

.macro restore_gprs
    # restore all gprs from the trap frame on the stack
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

    # Restore the original stack pointer. This also deallocates the trap frame.
    ld   sp, 1*8(sp)
.endm

.section .text
.global trap_handler

strap_handler:

    save_gprs

    mv      a0, sp
    call    trap_handler

    restore_gprs
    sret