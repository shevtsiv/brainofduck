trait CharReader {
    fn read_char(&mut self) -> char;
}

trait CharWriter {
    fn write_char(&mut self, char_to_write: char);
}

const ARRAY_MAX_SIZE: usize = 30_000;

struct Interpreter {
    array: [u8; ARRAY_MAX_SIZE],
    pointer: usize,
}

impl Interpreter {
    fn interpret(
        &mut self,
        program: &str,
        reader: &mut impl CharReader,
        writer: &mut impl CharWriter,
    ) {
        let mut cycle_stack = vec![];
        let mut i = 0;
        while i < program.len() {
            match program.chars().nth(i).unwrap() {
                '>' => {
                    self.pointer = self.pointer.wrapping_add(1);
                    if self.pointer == ARRAY_MAX_SIZE {
                        self.pointer = 0;
                    }
                }
                '<' => {
                    self.pointer = if let Some(decremented_pointer) = self.pointer.checked_sub(1) {
                        decremented_pointer
                    } else {
                        ARRAY_MAX_SIZE - 1
                    };
                }
                '+' => self.array[self.pointer] = self.array[self.pointer].wrapping_add(1),
                '-' => self.array[self.pointer] = self.array[self.pointer].wrapping_sub(1),
                '.' => writer.write_char(self.array[self.pointer] as char),
                ',' => self.array[self.pointer] = reader.read_char() as u8,
                '[' => {
                    if self.array[self.pointer] != 0 {
                        cycle_stack.push(i);
                    } else {
                        // skip the cycle
                        let mut unclosed_brackets_counter = 0;
                        loop {
                            match program.chars().nth(i) {
                                Some(smb) => match smb {
                                    '[' => unclosed_brackets_counter += 1,
                                    ']' => unclosed_brackets_counter -= 1,
                                    _ => {}
                                },
                                None => panic!("Invalid cycle has been found!"),
                            }
                            if unclosed_brackets_counter == 0 {
                                break;
                            }
                            i += 1;
                        }
                    }
                }
                ']' => {
                    i = match cycle_stack.pop() {
                        Some(open_bracket_index) => open_bracket_index,
                        None => panic!("Invalid cycle has been found!"),
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
}

pub fn main() {
    let mut interpreter = Interpreter {
        array: [0; ARRAY_MAX_SIZE],
        pointer: 0,
    };
    let input = "++>++<[->+<]>.";

    #[derive(Default)]
    struct R;
    impl CharReader for R {
        fn read_char(&mut self) -> char {
            let mut read_from_keyboard_char = [0u8];
            use std::io::{stdin, Read};
            stdin().read(&mut read_from_keyboard_char).unwrap();
            return read_from_keyboard_char[0] as char;
        }
    }
    #[derive(Default)]
    struct W {
        output_capture: String,
    };
    impl CharWriter for W {
        fn write_char(&mut self, char_to_write: char) {
            self.output_capture.push(char_to_write);
        }
    }
    let mut output_stream = W::default();
    let mut input_stream = R::default();
    interpreter.interpret(input, &mut input_stream, &mut output_stream);
    println!(
        "Result: {}",
        output_stream.output_capture.bytes().next().unwrap()
    );
}

#[cfg(test)]
mod tests {
    use crate::{CharReader, CharWriter, ARRAY_MAX_SIZE};

    fn interpret(input: &str) -> String {
        use super::Interpreter;
        let mut interpreter = Interpreter {
            array: [0; ARRAY_MAX_SIZE],
            pointer: 0,
        };
        #[derive(Default)]
        struct TestReader;
        impl CharReader for TestReader {
            fn read_char(&mut self) -> char {
                unimplemented!()
            }
        }
        #[derive(Default)]
        struct TestWriter {
            output_capture: String,
        };
        impl CharWriter for TestWriter {
            fn write_char(&mut self, char_to_write: char) {
                self.output_capture.push(char_to_write);
            }
        }
        let mut output_stream = TestWriter::default();
        let mut input_stream = TestReader::default();
        interpreter.interpret(input, &mut input_stream, &mut output_stream);
        output_stream.output_capture
    }

    #[test]
    pub fn hello_world() {
        let hello_world_program = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.
                           >---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
        assert_eq!("Hello World!\n", interpret(hello_world_program).as_str());
    }

    #[test]
    pub fn sum() {
        let sum_2_and_3 = "++>+++<[->+<]>.";
        assert_eq!(interpret(sum_2_and_3).bytes().next().unwrap(), 5);
    }

    #[test]
    pub fn pointer_underflow() {
        let increment_with_underflow = "<+.";
        assert_eq!(
            interpret(increment_with_underflow).bytes().next().unwrap(),
            1
        );
    }

    #[test]
    pub fn pointer_overflow() {
        let increment_with_overflow = "+++.<.+.>.";
        let output = interpret(increment_with_overflow);
        let mut bytes = output.bytes();
        assert_eq!(bytes.next().unwrap(), 3);
        assert_eq!(bytes.next().unwrap(), 0);
        assert_eq!(bytes.next().unwrap(), 1);
        assert_eq!(bytes.next().unwrap(), 3);
    }

    #[test]
    #[should_panic]
    pub fn invalid_loop_closing() {
        let invalid_loop_closing_program = "]";
        interpret(invalid_loop_closing_program);
    }
}
