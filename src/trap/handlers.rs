use crate::trap::{Trap, TrapFrame, read_scause};

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
