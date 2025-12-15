pub mod bus;
pub mod cpu;
pub mod csr;
pub mod decoder;
pub mod devices;
pub mod elf;

pub use bus::Bus;
pub use cpu::Cpu;
pub use csr::Csr;

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    };
}
