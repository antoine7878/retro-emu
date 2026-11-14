mod parser;

use std::path::Path;
use std::process::exit;
use std::str::Chars;
use std::vec;

use parser::{ArgError, ArgParser, argv};

#[derive(Debug)]
pub struct Args {
    pub infiles: Vec<String>,
    pub debug: bool,
    pub debug_init_file: Option<String>,
    pub serial_print: bool,
    argv: vec::IntoIter<String>,
}

impl ArgParser for Args {
    type Argv = vec::IntoIter<String>;

    fn positional(&mut self, arg: String) { self.infiles.push(arg); }

    fn argv(&mut self) -> &mut Self::Argv { &mut self.argv }

    fn flag(&mut self, c: char, it: &mut Chars) -> Result<(), ArgError> {
        match c {
            'g' => self.debug = true,
            's' => self.serial_print = true,
            'f' => self.debug_init_file = self.value(it, 'f')?,
            'h' => Self::help(),
            c => return Err(ArgError::UnknownOption(c)),
        }
        Ok(())
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            infiles: Vec::default(),
            debug: false,
            argv: argv(),
            debug_init_file: None,
            serial_print: false,
        }
    }
}

impl Args {
    pub fn parse() -> Result<Self, ArgError> {
        let mut parsed = Self::default();
        parsed.walk()?;
        parsed.process();
        parsed.check()
    }

    fn process(&mut self) {
        if self.debug_init_file.is_some() {
            self.debug = true;
        }
    }

    fn check(self) -> Result<Self, ArgError> {
        match self.infiles.len() {
            0 => Ok(self),
            1 if Path::new(&self.infiles[0]).is_file() => Ok(self),
            1 => Err(ArgError::NotAfile(self.infiles[0].clone())),
            n => Err(ArgError::BadArgumentCount(n, 1)),
        }
    }

    pub fn help() {
        eprintln!("usage: retro_emu rom.gb [-f debuf_file] [-gsh]");
        exit(0);
    }
}
