use super::terminal::Terminal;
use std::{
    io::{Read, Write, stdout},
    sync::mpsc,
    thread,
};

pub struct StdioTerminal {
    input_rx: mpsc::Receiver<u8>,
}

impl StdioTerminal {
    pub fn new() -> (Self, thread::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel();
        let handle = thread::spawn(move || {
            let stdin = std::io::stdin();
            for byte in stdin.lock().bytes() {
                if let Ok(b) = byte {
                    if tx.send(b).is_err() {
                        break;
                    }
                }
            }
        });
        (StdioTerminal { input_rx: rx }, handle)
    }
}

impl Terminal for StdioTerminal {
    fn read(&mut self) -> Option<u8> {
        match self.input_rx.try_recv() {
            Ok(byte) => Some(byte),
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => None,
        }
    }

    fn write(&mut self, data: u8) {
        print!("{}", data as char);
        stdout().flush().unwrap();
    }
}
