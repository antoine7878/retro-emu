use std::cmp::Ordering;
use std::io::Write;

use crate::gameboy::GameBoy;

use super::{BOLD, CYAN, Debugger, RED, RESET};

const GREY: &str = "\x1b[2m";
const WHITE: &str = "\x1b[7m";
const PANEL_HEIGHT: usize = 12;
const ASM_WIDTH: usize = 32;
const REG_WIDTH: usize = 20;
const CONSOLE_HEIGHT: usize = 10;
const SHORTCUTS: [(&str, &str); 11] = [
    ("c", "Continue"),
    ("s", "Step"),
    ("t", "Trace"),
    ("P", "Pause (window)"),
    ("[ad][bw]", "Add/Delete Breakpoint/Watchpoint"),
    ("l", "List"),
    ("cls", "Clear"),
    ("r", "Reset"),
    ("⏎", "Repeat"),
    ("q", "Quit"),
    ("h", "Help"),
];

impl Debugger {
    pub fn draw(&self, game_boy: &GameBoy) {
        let mut out = String::from("\x1b[H\x1b[J");
        out += &self.disassembly_registers_panel(game_boy);
        out += &self.console_lines();
        let width = Self::terminal_size().map_or(usize::MAX, |(_, cols)| cols);
        out += &format!("\n\n{}\x1b[A\r", Self::shortcut_bar(width));
        let mut stdout = std::io::stdout().lock();
        let _ = write!(stdout, "{out}");
        let _ = stdout.flush();
    }

    pub fn draw_live(&self, game_boy: &GameBoy) {
        let body = self.disassembly_registers_panel(game_boy) + &self.console_lines();
        let mut stdout = std::io::stdout().lock();
        let _ = write!(stdout, "\x1b7\x1b[H{}\x1b[K\x1b8", body.replace('\n', "\x1b[K\n"));
        let _ = stdout.flush();
    }

    fn disassembly_registers_panel(&self, game_boy: &GameBoy) -> String {
        let asm = self.make_asm(game_boy);
        let regs = game_boy.dump_registers();
        let mut out = format!(
            "┌─{BOLD} Disassembly {RESET}{:─<w1$}┐┌─{BOLD} Registers {RESET}{:─<w2$}┐\n",
            "",
            "",
            w1 = ASM_WIDTH - 14,
            w2 = REG_WIDTH - 12
        );
        for i in 0..PANEL_HEIGHT {
            let left = asm.get(i).cloned().unwrap_or_else(|| Self::pad("", ASM_WIDTH));
            let right = Self::pad(regs.get(i).map_or("", String::as_str), REG_WIDTH);
            let (name, value) = right.split_at(2);
            let right = format!("{CYAN}{name}{RESET}{value}");
            out += &format!("│{left}││{right}│\n");
        }
        out += &format!("└{:─<ASM_WIDTH$}┘└{:─<REG_WIDTH$}┘\n", "", "");
        out
    }

    fn make_asm(&self, game_boy: &GameBoy) -> Vec<String> {
        let current = self.history.len();

        let past = self.history.iter().flat_map(|&addr| game_boy.disassembly(addr, 1));
        let next = game_boy.disassembly(game_boy.pc(), PANEL_HEIGHT - current);

        past.chain(next)
            .enumerate()
            .map(|(i, (addr, text))| {
                let bp = if self.breakpoints.contains_key(&addr) { '●' } else { ' ' };
                let line = Self::pad(&format!(" ${addr:04X}  {text}"), ASM_WIDTH - 1);
                let style = match i.cmp(&current) {
                    Ordering::Less => GREY,
                    Ordering::Equal => WHITE,
                    Ordering::Greater => "",
                };
                format!("{style}{RED}{bp}{RESET}{style}{line}{RESET}")
            })
            .collect()
    }

    fn console_lines(&self) -> String {
        let height = Self::console_height();
        let blank = height.saturating_sub(self.console.len());
        let start = self.console.len().saturating_sub(height);
        (0..height)
            .map(|i| i.checked_sub(blank).and_then(|j| self.console.get(start + j)).map_or("", String::as_str))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn pad(s: &str, width: usize) -> String { format!("{s:<width$.width$}") }

    fn shortcut_bar(width: usize) -> String {
        let mut out = String::new();
        let mut len = 0;
        for (key, desc) in SHORTCUTS {
            let item_len = key.chars().count() + 1 + desc.len() + 2;
            if len + item_len > width {
                break;
            }
            len += item_len;
            out += &format!("{WHITE}{key}{RESET} {desc}  ");
        }
        out
    }

    fn console_height() -> usize {
        Self::terminal_size().map_or(CONSOLE_HEIGHT, |(rows, _)| rows.saturating_sub(PANEL_HEIGHT + 2 + 2).max(1))
    }

    fn terminal_size() -> Option<(usize, usize)> {
        let out = std::process::Command::new("stty").arg("size").stdin(std::process::Stdio::inherit()).output().ok()?;
        let out = String::from_utf8(out.stdout).ok()?;
        let mut it = out.split_whitespace().map(|n| n.parse().ok());
        Some((it.next()??, it.next()??))
    }
}
