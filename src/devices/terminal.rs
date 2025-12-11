pub trait Terminal {
    fn write(&mut self, data: u8);
    fn read(&mut self) -> Option<u8>;
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::collections::VecDeque;

    /// 테스트용 Mock Terminal
    pub struct MockTerminal {
        pub output: Vec<u8>,
        pub input: VecDeque<u8>,
    }

    impl MockTerminal {
        pub fn new() -> Self {
            MockTerminal {
                output: Vec::new(),
                input: VecDeque::new(),
            }
        }

        /// 입력 데이터 설정
        pub fn push_input(&mut self, data: u8) {
            self.input.push_back(data);
        }

        /// 출력된 데이터를 문자열로 반환
        pub fn output_as_string(&self) -> String {
            String::from_utf8_lossy(&self.output).to_string()
        }
    }

    impl Terminal for MockTerminal {
        fn write(&mut self, data: u8) {
            self.output.push(data);
        }

        fn read(&mut self) -> Option<u8> {
            self.input.pop_front()
        }
    }

    #[test]
    fn test_mock_terminal_write() {
        let mut terminal = MockTerminal::new();

        terminal.write(b'H');
        terminal.write(b'i');

        assert_eq!(terminal.output, vec![b'H', b'i']);
        assert_eq!(terminal.output_as_string(), "Hi");
    }

    #[test]
    fn test_mock_terminal_read_empty() {
        let mut terminal = MockTerminal::new();

        assert_eq!(terminal.read(), None);
    }

    #[test]
    fn test_mock_terminal_read_with_input() {
        let mut terminal = MockTerminal::new();

        terminal.push_input(b'A');
        terminal.push_input(b'B');

        assert_eq!(terminal.read(), Some(b'A'));
        assert_eq!(terminal.read(), Some(b'B'));
        assert_eq!(terminal.read(), None);
    }

    #[test]
    fn test_mock_terminal_read_write_combined() {
        let mut terminal = MockTerminal::new();

        // 입력 설정
        terminal.push_input(b'X');

        // 읽기
        let input = terminal.read().unwrap();

        // 에코 (읽은 것을 다시 출력)
        terminal.write(input);

        assert_eq!(terminal.output_as_string(), "X");
    }
}
