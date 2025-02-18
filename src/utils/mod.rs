use std::io::{stdin, BufRead};

pub fn read() -> String {
    let stdin = stdin();
    let mut lines = String::new();
    stdin.lock().read_line(&mut lines).unwrap();
    lines
}
