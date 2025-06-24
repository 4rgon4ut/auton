.option norvc
.section .init


.global _start

_start:
    .cfi_startproc

    csrr    a0, mhartid
    bnez    a0, hart_jail

setup_pointers:
    csrw    satp, zero

.option push
.option norelax
    la      gp, _global_pointer
.option pop

    la      sp, _stack_top

    la      t0, _bss_start
    la      t1, _bss_end
    bgeu    t0, t1, bss_clear_done  # skip if BSS size is zero

bss_clear_loop:
    sd      zero, (t0)
    addi    t0, t0, 8
    bltu    t0, t1, bss_clear_loop

bss_clear_done:
    # pass


# delegate_traps:
#     li      t0, -1
#     csrw    medeleg, t0
#     csrw    mideleg, t0

#     la      t0, supervisor_trap_handler
#     csrw    stvec, t0

prepare_s_mode_transition:
    csrr    t0, mstatus
    li      t1, 0b1100000000000     # mpp bitmask (12:11)
    not     t1, t1                  # invert bits ( --> 0b1110011111111111 )
    and     t0, t0, t1             # clear mpp bits

    li      t1, 0b0100000000000     # (12:11) = 01 (s-mode)
    or      t0, t0, t1              # set the bits

    csrw    mstatus, t0             # set mstatus to s-mode

jump_to_kernel:
    csrr a0, mhartid
    li   a1, 0

    la   t0, kmain
    csrw mepc, t0

    mret

    .cfi_endproc

# ---
hart_jail:
    wfi
    j   hart_jail