use super::terminal::Terminal;
use std::io::{Write, stdout};

pub struct StdioTerminal;

impl Terminal for StdioTerminal {
    fn read(&mut self) -> Option<u8> {
        None
    }

    fn write(&mut self, data: u8) {
        print!("{}", data as char);
        stdout().flush().unwrap();
    }
}
