#[derive(Debug)]
pub enum Trap {
    Interrupt(Interrupt),
    Exception(Exception),
}

#[derive(Debug)]
#[repr(usize)]
pub enum Interrupt {
    SupervisorSoft = 1,
    SupervisorTimer = 5,
    SupervisorExternal = 9,
    // TODO: add more interrupts as needed
}

#[derive(Debug)]
#[repr(usize)]
pub enum Exception {
    InstructionMisaligned = 0,
    InstructionFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadFault = 5,
    StoreFault = 7,
    UserEcall = 8,
    SupervisorEcall = 9,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    StorePageFault = 15,
    // TODO: add more exceptions as needed
}

impl TryFrom<usize> for Trap {
    type Error = &'static str;

    fn try_from(cause: usize) -> Result<Self, Self::Error> {
        const INTERRUPT_MASK: usize = 1 << (usize::BITS - 1);
        let code = cause & !INTERRUPT_MASK;

        if cause & INTERRUPT_MASK != 0 {
            let interrupt = match code {
                1 => Interrupt::SupervisorSoft,
                5 => Interrupt::SupervisorTimer,
                9 => Interrupt::SupervisorExternal,
                _ => return Err("Unknown interrupt code"),
            };
            Ok(Trap::Interrupt(interrupt))
        } else {
            let exception = match code {
                0 => Exception::InstructionMisaligned,
                1 => Exception::InstructionFault,
                2 => Exception::IllegalInstruction,
                3 => Exception::Breakpoint,
                5 => Exception::LoadFault,
                7 => Exception::StoreFault,
                8 => Exception::UserEcall,
                9 => Exception::SupervisorEcall,
                12 => Exception::InstructionPageFault,
                13 => Exception::LoadPageFault,
                15 => Exception::StorePageFault,
                _ => return Err("Unknown exception code"),
            };
            Ok(Trap::Exception(exception))
        }
    }
}

#[repr(C)]
#[derive(Debug)]
pub struct TrapFrame {
    gprs: [usize; 32],
    // TODO: add more fields as needed
    // pub sstatus: usize,
    // pub sepc: usize,
}

#[inline(always)]
pub fn read_scause() -> usize {
    let cause: usize;
    unsafe {
        core::arch::asm!("csrr {}, scause", out(reg) cause);
    }
    cause
}

#[unsafe(no_mangle)]
pub extern "C" fn trap_handler(frame: &mut TrapFrame) -> ! {
    let cause = read_scause();

    match Trap::try_from(cause) {
        Ok(trap) => match trap {
            Trap::Interrupt(interrupt) => {
                panic!("Interrupt: {:?}", interrupt);
            }
            Trap::Exception(exception) => {
                panic!("Exception: {:?}", exception);
            }
        },
        Err(e) => {
            panic!("{}", e);
        }
    }
}
