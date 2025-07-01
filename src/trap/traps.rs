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
    pub gprs: [usize; 32], // 256
    pub sstatus: usize,    // 264
    pub sepc: usize,       // 272
    pub stval: usize,      // 280
    pub scause: usize,     // 288
}

impl core::fmt::Display for TrapFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "--- TrapFrame ---")?;
        writeln!(f, "  sstatus: {:#x}", self.sstatus)?;
        writeln!(f, "  sepc:    {:#x}", self.sepc)?;
        writeln!(f, "  stval:   {:#x}", self.stval)?;
        writeln!(f, "  scause:  {:#x}", self.scause)?;
        writeln!(f, "  Registers:")?;

        // Print General Purpose Registers (GPRs) in a structured way
        // x0 is hardwired to zero, so we typically start from x1 (ra)
        // Let's print them in rows for readability
        for i in 0..32 {
            // Print 4 registers per line
            if i % 4 == 0 {
                write!(f, "    ")?;
            }
            write!(f, "x{:02}: {:#018x}  ", i, self.gprs[i])?;
            if i % 4 == 3 || i == 31 {
                // End of line or last register
                writeln!(f)?;
            }
        }
        writeln!(f, "-----------------")
    }
}
