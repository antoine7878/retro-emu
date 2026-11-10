use std::io::{Read, Write};
use std::process::{Command, Stdio};

const CTRL_A: u8 = 0x01;
const CTRL_C: u8 = 0x03;
const CTRL_D: u8 = 0x04;
const CTRL_E: u8 = 0x05;
const BACKSPACE: u8 = 0x08;
const ESC: u8 = 0x1b;
const DEL: u8 = 0x7f;

#[derive(Default)]
pub struct Prompt {
    history: Vec<String>,
}

impl Prompt {
    pub fn read_line(&mut self, prompt: &str) -> Option<String> {
        stty(&["-icanon", "-echo", "-isig", "min", "0", "time", "0"]);
        while std::io::stdin().read(&mut [0; 64]).is_ok_and(|n| n > 0) {}
        stty(&["min", "1"]);
        let line = self.edit(prompt);
        stty(&["icanon", "isig"]);
        println!();
        if let Some(line) = &line
            && !line.trim().is_empty()
            && self.history.last() != Some(line)
        {
            self.history.push(line.clone());
        }
        line
    }

    fn edit(&self, prompt: &str) -> Option<String> {
        let mut stdin = std::io::stdin().lock();
        let mut next = || {
            let mut byte = [0];
            stdin.read_exact(&mut byte).ok().map(|_| byte[0])
        };
        let mut buf: Vec<u8> = Vec::new();
        let mut cursor = 0;
        let mut hist = self.history.len();

        loop {
            redraw(prompt, &buf, cursor);
            match next()? {
                b'\r' | b'\n' => return Some(String::from_utf8_lossy(&buf).into_owned()),
                CTRL_C => return None,
                CTRL_D if buf.is_empty() => return None,
                CTRL_A => cursor = 0,
                CTRL_E => cursor = buf.len(),
                DEL | BACKSPACE if cursor > 0 => {
                    cursor -= 1;
                    buf.remove(cursor);
                }
                ESC => match (next()?, next()?) {
                    (b'[' | b'O', b'A') if hist > 0 => {
                        hist -= 1;
                        buf = self.history[hist].clone().into_bytes();
                        cursor = buf.len();
                    }
                    (b'[' | b'O', b'B') if hist < self.history.len() => {
                        hist += 1;
                        buf = self.history.get(hist).map_or_else(Vec::new, |s| s.clone().into_bytes());
                        cursor = buf.len();
                    }
                    (b'[' | b'O', b'C') => cursor = (cursor + 1).min(buf.len()),
                    (b'[' | b'O', b'D') => cursor = cursor.saturating_sub(1),
                    (b'[' | b'O', b'H') => cursor = 0,
                    (b'[' | b'O', b'F') => cursor = buf.len(),
                    (b'[', b'3') if next()? == b'~' && cursor < buf.len() => {
                        buf.remove(cursor);
                    }
                    _ => {}
                },
                c @ 0x20..0x7f => {
                    buf.insert(cursor, c);
                    cursor += 1;
                }
                _ => {}
            }
        }
    }
}

fn redraw(prompt: &str, buf: &[u8], cursor: usize) {
    let mut out = format!("\r\x1b[K{prompt}{}", String::from_utf8_lossy(buf));
    if cursor < buf.len() {
        out += &format!("\x1b[{}D", buf.len() - cursor);
    }
    let mut stdout = std::io::stdout().lock();
    let _ = stdout.write_all(out.as_bytes());
    let _ = stdout.flush();
}

pub fn restore_terminal() { stty(&["icanon", "echo", "isig"]); }

fn stty(args: &[&str]) { let _ = Command::new("stty").args(args).stdin(Stdio::inherit()).status(); }
