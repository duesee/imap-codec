#![allow(dead_code)]

use std::io::Write;

pub const COLOR_SERVER: &str = "\x1b[34m";
pub const COLOR_CLIENT: &str = "\x1b[31m";
pub const RESET: &str = "\x1b[0m";

#[derive(Clone, Copy, Debug)]
pub enum Role {
    Client,
    Server,
}

pub fn read_more(buffer: &mut Vec<u8>, role: Role) {
    let prompt = if buffer.is_empty() {
        match role {
            Role::Client => "C: ",
            Role::Server => "S: ",
        }
    } else {
        ".. "
    };

    let line = read_line(prompt, role);

    // If `read_line` produces an empty string, standard input has been closed.
    if line.is_empty() || line.trim() == "exit" {
        println!("Exiting.");
        std::process::exit(0);
    }

    buffer.extend_from_slice(line.as_bytes());
}

fn read_line(prompt: &str, role: Role) -> String {
    match role {
        Role::Client => print!("{}{COLOR_CLIENT}", prompt),
        Role::Server => print!("{}{COLOR_SERVER}", prompt),
    }

    std::io::stdout().flush().unwrap();

    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();

    print!("{RESET}");

    // If `Stdin::read_line` produces an empty string, standard input has been closed.
    if line.is_empty() {
        return line;
    }

    // Ensure `CRLF` line ending of resulting string.
    // Line ending of `line` can be one of:
    // - `CRLF` on Windows
    // - `LF` on Unix-like
    // - none when EOF of standard input is reached
    if line.ends_with("\r\n") {
        return line;
    }
    if line.ends_with('\n') {
        line.pop();
    }
    line + "\r\n"
}
