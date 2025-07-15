pub const CACHE_LINE_SIZE: usize = 64;

pub fn current_hart_id() -> usize {
    let hart_id: usize;
    unsafe {
        core::arch::asm!("csrr {}, mhartid", out(reg) hart_id);
    }
    hart_id
}
