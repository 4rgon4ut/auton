.option norvc
.section .init

.type start, @function
.global start

start:
    .cfi_startproc

    csrr    t0, mhartid
    bnez    t0, hart_jail

boot_hart_init:
    csrw    satp, zero

.option push
.option norelax
    la      gp, _global_pointer
.option pop

    la      sp, _stack_top

    la      t0, _bss_start
    la      t1, _bss_end
    bgeu    t0, t1, bss_clear_done  # Skip if BSS size is zero or invalid

bss_clear_loop:
    sd      zero, (t0)
    addi    t0, t0, 8
    bltu    t0, t1, bss_clear_loop

bss_clear_done:
    csrr    a0, mhartid
    li      a1, 0

    #    Set the return address to kmain
    la      t0, kmain
    csrw    mepc, t0

    #    Set the Previous Privilege mode in mstatus to Supervisor (S-Mode)
    #    MPP bits (12:11) = 01 for Supervisor
    csrr    t0, mstatus
    li      t1, ~(3 << 11)          # Create a mask to clear MPP bits
    and     t0, t0, t1
    li      t1, 1 << 11             # Set MPP to 01 (S-Mode)
    or      t0, t0, t1
    csrw    mstatus, t0

    mret

    .cfi_endproc


hart_jail:
    wfi
    j   hart_jail