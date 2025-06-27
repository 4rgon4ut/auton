use crate::trap::{Trap, TrapFrame};

#[unsafe(no_mangle)]
pub extern "C" fn trap_handler(frame: &mut TrapFrame) -> ! {
    match Trap::try_from(frame.scause) {
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
