pub mod cpu;
pub mod decoder;
pub mod devices;

pub use cpu::Cpu;

pub const DRAM_BASE: u32 = 0x80000000;

#[macro_export]
macro_rules! debug_log {
    ($($arg:tt)*) => {
        if cfg!(debug_assertions) {
            println!($($arg)*);
        }
    };
}
