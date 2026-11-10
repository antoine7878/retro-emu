use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::fs::read_to_string;
use std::io::Write;

use crate::args::Args;

use crate::gbc::{AccessKind, GameBoy, MemAccess, Monitor, memory_location, memory_range};

pub const PROMPT: &str = "\x1b[1;32m(gbdb)\x1b[0m ";
const GREY: &str = "\x1b[2m";
const WHITE: &str = "\x1b[7m";
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const RED: &str = "\x1b[31m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const PANEL_HEIGHT: usize = 12;
const HISTORY_LEN: usize = 3;
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Watchpoint {
    start: u16,
    end: u16,
    value: Option<u8>,
    on: AccessKind,
}

impl fmt::Display for Watchpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{CYAN}${:04X}{RESET}", self.start)?;
        if self.start != self.end {
            write!(f, "..{CYAN}${:04X}{RESET}", self.end)?;
        }
        write!(f, " on {}", self.on)?;
        Ok(())
        // if let Some(value) = self.value { writeln!(f, "to {value}") } else { writeln!(f, "change") }
    }
}

#[derive(Default)]
pub struct Debugger {
    breakpoints: HashMap<u16, usize>,
    watchpoints: HashMap<Watchpoint, usize>,
    step_budget: Option<u32>,
    skip_break: bool,
    console: Vec<String>,
    history: VecDeque<u16>,
    initial_cmds: Vec<String>,
}

pub enum StopReason {
    Breakpoint(u16),
    Watchpoint(MemAccess),
    Step,
    Pause,
}

impl fmt::Display for StopReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StopReason::Step => write!(f, "{YELLOW}Step{RESET}"),
            StopReason::Pause => write!(f, "{YELLOW}Paused{RESET}"),
            StopReason::Breakpoint(addr) => {
                write!(f, "{YELLOW}Breakpoint{RESET} at {CYAN}${addr:04X}{RESET}",)
            }
            StopReason::Watchpoint(MemAccess { addr, value, on }) => {
                write!(
                    f,
                    "{YELLOW}Watchpoint{RESET} {on} value 0x{value:02X} at {CYAN}${addr:04X}{RESET} in {}",
                    memory_location(*addr)
                )
            }
        }
    }
}

impl Monitor for Debugger {
    type Stop = StopReason;

    fn check(&mut self, pc: u16) -> Option<Self::Stop> {
        let reason = self.stop_reason(pc);
        if reason.is_none() {
            if self.history.len() == HISTORY_LEN {
                self.history.pop_front();
            }
            self.history.push_back(pc);
        }
        reason
    }

    fn check_accesses(&mut self, mut accesses: impl Iterator<Item = MemAccess>) -> Option<Self::Stop> {
        accesses
            .find(|a| self.watchpoints.keys().any(|w| w.on == a.on && (w.start..=w.end).contains(&a.addr)))
            .map(StopReason::Watchpoint)
    }
}

pub enum DebugAction {
    Continue,
    Step,
    Trace(u32),
    Stay,
    Reset,
}

impl Debugger {
    pub fn new(args: &Args) -> Option<Self> {
        if !args.debug {
            return None;
        }

        let initial_cmds = args
            .debug_init_file
            .as_ref()
            .map(|file| {
                read_to_string(file)
                    .unwrap()
                    .lines()
                    .filter(|l| !l.trim_start().starts_with("#"))
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();
        let mut dbg = Self { initial_cmds, ..Default::default() };
        dbg.reset();
        Some(dbg)
    }

    pub fn reset(&mut self) -> DebugAction {
        self.step_budget = None;
        self.skip_break = false;
        self.console.clear();
        self.console.push(format!("{RED}Reset{RESET}"));
        self.history.clear();
        for cmd in &self.initial_cmds.clone() {
            self.handle_command(cmd.as_str());
        }
        DebugAction::Reset
    }

    fn stop_reason(&mut self, pc: u16) -> Option<StopReason> {
        if let Some(n) = &mut self.step_budget {
            if *n == 0 {
                self.step_budget = None;
                return Some(StopReason::Step);
            }
            *n -= 1;
        }
        if std::mem::take(&mut self.skip_break) {
            return None;
        }
        self.breakpoints.contains_key(&pc).then_some(StopReason::Breakpoint(pc))
    }

    pub fn resume(&mut self) { self.skip_break = true; }

    pub fn handle_command(&mut self, cmd: &str) -> DebugAction {
        self.log(format!("{PROMPT}{cmd}"));
        let it = &mut cmd.split_whitespace();
        match it.next() {
            Some("r") => return self.reset(),
            Some("c") => return self.continu(),
            Some("s") => return self.step(it),
            Some("t") => return self.trace(it),
            Some("ab") => self.add_breakpoint(it),
            Some("db") => self.delete_breakpoing(it),
            Some("aw") => self.add_watchpoint(it),
            Some("dw") => self.delete_watchpoint(it),
            Some("l") => self.list_points(),
            Some("q") => std::process::exit(0),
            Some("h") => self.help(),
            Some("cls") => self.console.clear(),
            Some(other) => self.log(format!("{RED}unknown command: {other}{RESET}")),
            None => (),
        }
        DebugAction::Stay
    }

    fn continu(&mut self) -> DebugAction {
        self.resume();
        DebugAction::Continue
    }

    fn step<'a, It: Iterator<Item = &'a str>>(&mut self, it: &mut It) -> DebugAction {
        let n = Self::parse_int(it);
        self.step_budget = Some(n);
        self.resume();
        DebugAction::Step
    }

    fn trace<'a, It: Iterator<Item = &'a str>>(&mut self, it: &mut It) -> DebugAction {
        let n = Self::parse_int(it);
        self.resume();
        DebugAction::Trace(n)
    }

    pub fn set_step_budget(&mut self, n: u32) { self.step_budget = Some(n); }

    fn add_breakpoint<'a, It: Iterator<Item = &'a str>>(&mut self, it: &mut It) {
        let max_id = *self.breakpoints.values().max().unwrap_or(&0) + 1;
        let Some(addr) = Self::parse_addr(it) else { return };
        if self.breakpoints.insert(addr, max_id).is_none() {
            self.log(format!("{YELLOW}Breakpoint{RESET} {max_id} added: {CYAN}${addr:04X}{RESET}"));
        }
    }

    fn delete_breakpoing<'a, It: Iterator<Item = &'a str>>(&mut self, it: &mut It) {
        let point_id = Self::parse_int(it) as usize;
        let Some((&point, _)) = self.breakpoints.iter().find(|&(_, id)| id == &point_id) else { return };
        if let Some(id) = self.breakpoints.remove(&point) {
            self.log(format!("{YELLOW}Breakpoint{RESET} {id} removed at {CYAN}${point:04X}{RESET}"));
        }
    }

    fn add_watchpoint<'a, It: Iterator<Item = &'a str>>(&mut self, it: &mut It) {
        let mut max_id = *self.watchpoints.values().max().unwrap_or(&0) + 1;
        let ons = match it.next() {
            Some("r" | "read") => [AccessKind::Read, AccessKind::Read],
            Some("w" | "write") => [AccessKind::Write, AccessKind::Write],
            Some("rw" | "wr") => [AccessKind::Read, AccessKind::Write],
            _ => return self.log(format!("{YELLOW}Watchpoint{RESET} {RED}not added{RESET}")),
        };
        let Some(range) = it.next() else {
            return self.log(format!("{YELLOW}Watchpoint{RESET} {RED}not added{RESET}"));
        };
        let mut locations = range.split("..").map(Self::parse_location);
        let Some(Some((start, first_end))) = locations.next() else {
            return self.log(format!("{YELLOW}Watchpoint{RESET} {RED}not added{RESET}"));
        };
        let end = match locations.next() {
            None => first_end,
            Some(Some((_, end))) => end,
            Some(None) => return self.log(format!("{YELLOW}Watchpoint{RESET} {RED}not added{RESET}")),
        };
        for on in ons {
            let point = Watchpoint { start, end, on, value: None };
            if self.watchpoints.insert(point, max_id).is_none() {
                max_id += 1;
                self.log(format!("{YELLOW}Watchpoint{RESET} {max_id} added: {point}"));
            }
        }
    }

    fn delete_watchpoint<'a, It: Iterator<Item = &'a str>>(&mut self, it: &mut It) {
        let point_id = Self::parse_int(it) as usize;
        let Some((&point, _)) = self.watchpoints.iter().find(|&(_, id)| id == &point_id) else { return };
        if let Some(id) = self.watchpoints.remove(&point) {
            self.log(format!("{YELLOW}Watchpoint{RESET} {id} removed: {point}"));
        }
    }

    fn list_points(&mut self) {
        let mut breakpoints: Vec<_> = self.breakpoints.iter().map(|(&point, &id)| (id, point)).collect();
        breakpoints.sort_by_key(|&(id, _)| id);
        self.log(format!("{BOLD}{YELLOW}Breakpoints{RESET}{BOLD}:{RESET}"));
        for (id, point) in breakpoints {
            self.log(format!("  {id}: {CYAN}${point:04X}{RESET}"));
        }
        let mut watchpoints: Vec<_> = self.watchpoints.iter().map(|(&point, &id)| (id, point)).collect();
        watchpoints.sort_by_key(|&(id, _)| id);
        self.log(format!("{BOLD}{YELLOW}Watchpoints{RESET}{BOLD}:{RESET}"));
        for (id, point) in watchpoints {
            self.log(format!("  {id}: {point}"));
        }
    }

    fn help(&mut self) {
        self.log(format!("{BOLD}Commands:{RESET}"));
        self.log(format!("  {CYAN}r                    {RESET}reset"));
        self.log(format!("  {CYAN}c                    {RESET}continue"));
        self.log(format!("  {CYAN}s [n]                {RESET}step n instructions (default 1)"));
        self.log(format!("  {CYAN}t [n]                {RESET}trace n inst/frame (default 1)"));
        self.log(format!("  {CYAN}ab $<addr>           {RESET}add breakpoint at hex addr"));
        self.log(format!("  {CYAN}db <id>              {RESET}delete breakpoint by id"));
        self.log(format!("  {CYAN}aw r|w <a>[..<b>]    {RESET}add read/write watchpoint ($hex or name: LCDC, WRAM..)"));
        self.log(format!("  {CYAN}dw <id>              {RESET}delete watchpoint by id"));
        self.log(format!("  {CYAN}l                    {RESET}list breakpoints & watchpoints"));
        self.log(format!("  {CYAN}cls                  {RESET}clear the console"));
        self.log(format!("  {CYAN}h                    {RESET}show this help"));
        self.log(format!("  {CYAN}q                    {RESET}quit"));
        self.log(format!("  {CYAN}<enter>              {RESET}repeat last cmd / stop trace"));
        self.log(format!("  {CYAN}P (game window)      {RESET}pause execution"));
    }

    pub fn log(&mut self, line: String) { self.console.push(line); }

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

    fn parse_addr<'a, It: Iterator<Item = &'a str>>(it: &mut It) -> Option<u16> {
        it.next()?.strip_prefix('$').and_then(|hex| u16::from_str_radix(hex, 16).ok())
    }

    fn parse_location(s: &str) -> Option<(u16, u16)> {
        match s.strip_prefix('$') {
            Some(hex) => u16::from_str_radix(hex, 16).ok().map(|addr| (addr, addr)),
            None => memory_range(s),
        }
    }

    fn parse_int<'a, It: Iterator<Item = &'a str>>(it: &mut It) -> u32 {
        it.next().and_then(|s| s.parse().ok()).unwrap_or(1)
    }
}
