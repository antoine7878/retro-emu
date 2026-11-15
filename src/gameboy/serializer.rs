use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::hash::{BuildHasher, Hash};
use std::time::Duration;

#[derive(Debug)]
pub struct SerdeError {
    name: String,
    msg: String,
}

impl SerdeError {
    pub fn new<T>(name: &str, msg: &str) -> Result<T, Self> {
        Err(SerdeError { name: name.to_string(), msg: msg.to_string() })
    }
}

impl fmt::Display for SerdeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Deserialization of tag {} failed: {}", self.name, self.msg)
    }
}

impl Error for SerdeError {}

#[derive(Default)]
pub struct StateWriter {
    buf: Vec<u8>,
}

impl StateWriter {
    pub fn write(&mut self, bytes: &[u8]) { self.buf.extend_from_slice(bytes); }
    pub fn into_bytes(self) -> Vec<u8> { self.buf }
}

#[derive(Default)]
pub struct StateReader<'a> {
    buf: &'a [u8],
}

impl<'a> StateReader<'a> {
    pub fn new(buf: &'a [u8]) -> Self { Self { buf } }
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], SerdeError> {
        if self.buf.len() < n {
            return SerdeError::new("StateReader", "unexpected end of data");
        }
        let (head, tail) = self.buf.split_at(n);
        self.buf = tail;
        Ok(head)
    }

    pub fn take_array<const N: usize>(&mut self) -> Result<[u8; N], SerdeError> {
        Ok(self.take(N)?.try_into().unwrap())
    }

    pub fn read_into(&mut self, dst: &mut [u8]) -> Result<(), SerdeError> {
        dst.copy_from_slice(self.take(dst.len())?);
        Ok(())
    }
}

pub trait SaveState {
    fn save(&self, w: &mut StateWriter);
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError>;
}

impl SaveState for bool {
    fn save(&self, w: &mut StateWriter) { w.write(&(*self as u8).to_be_bytes()); }
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        *self = match r.take(1)?[0] {
            0 => false,
            1 => true,
            _ => return SerdeError::new("bool", "invalid value"),
        };
        Ok(())
    }
}

impl SaveState for u8 {
    fn save(&self, w: &mut StateWriter) { w.write(&self.to_be_bytes()); }
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        *self = r.take(1)?[0];
        Ok(())
    }
}

impl SaveState for u16 {
    fn save(&self, w: &mut StateWriter) { w.write(&self.to_be_bytes()); }
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        *self = u16::from_be_bytes(r.take_array()?);
        Ok(())
    }
}

impl SaveState for u32 {
    fn save(&self, w: &mut StateWriter) { w.write(&self.to_be_bytes()); }
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        *self = u32::from_be_bytes(r.take_array()?);
        Ok(())
    }
}

impl SaveState for u64 {
    fn save(&self, w: &mut StateWriter) { w.write(&self.to_be_bytes()); }
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        *self = u64::from_be_bytes(r.take_array()?);
        Ok(())
    }
}

impl SaveState for usize {
    fn save(&self, w: &mut StateWriter) { w.write(&(*self as u64).to_be_bytes()); }
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        *self = u64::from_be_bytes(r.take_array()?) as usize;
        Ok(())
    }
}

impl<const N: usize> SaveState for [u8; N] {
    fn save(&self, w: &mut StateWriter) { w.write(self); }
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> { r.read_into(self) }
}

impl<const N: usize, const M: usize, const O: usize> SaveState for [[[u8; N]; M]; O] {
    fn save(&self, w: &mut StateWriter) { w.write(self.as_flattened().as_flattened()) }
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        r.read_into(self.as_flattened_mut().as_flattened_mut())
    }
}

impl<const N: usize, const M: usize> SaveState for [[u8; N]; M] {
    fn save(&self, w: &mut StateWriter) { w.write(self.as_flattened()) }
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> { r.read_into(self.as_flattened_mut()) }
}

impl SaveState for Vec<u8> {
    fn save(&self, w: &mut StateWriter) {
        self.len().save(w);
        w.write(self);
    }
    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        let mut len = 0_usize;
        len.load(r)?;
        if len != self.len() {
            return SerdeError::new("Vec<u8>", "length mismatch");
        }
        r.read_into(self)
    }
}

impl<T: SaveState> SaveState for Option<T> {
    fn save(&self, w: &mut StateWriter) {
        match self {
            Some(t) => {
                1u8.save(w);
                t.save(w);
            }
            None => 0u8.save(w),
        }
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        match (r.take(1)?[0], self) {
            (0, None) => Ok(()),
            (1, Some(t)) => t.load(r),
            (0 | 1, _) => SerdeError::new("Option", "presence mismatch with loaded cartridge"),
            _ => SerdeError::new("Option", "invalid tag"),
        }
    }
}

impl SaveState for String {
    fn save(&self, w: &mut StateWriter) {
        self.len().save(w);
        w.write(self.as_bytes());
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        let mut len = 0_usize;
        len.load(r)?;
        let Ok(s) = std::str::from_utf8(r.take(len)?) else {
            return SerdeError::new("String", "invalid utf-8");
        };
        self.clear();
        self.push_str(s);
        Ok(())
    }
}

impl SaveState for Duration {
    fn save(&self, w: &mut StateWriter) {
        self.as_secs().save(w);
        self.subsec_nanos().save(w);
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        let mut secs = 0_u64;
        let mut nanos = 0_u32;
        secs.load(r)?;
        nanos.load(r)?;
        if nanos >= 1_000_000_000 {
            return SerdeError::new("Duration", "invalid nanoseconds");
        }
        *self = Duration::new(secs, nanos);
        Ok(())
    }
}

impl<T, S> SaveState for HashSet<T, S>
where
    T: SaveState + Default + Eq + Hash + Ord,
    S: BuildHasher,
{
    fn save(&self, w: &mut StateWriter) {
        let mut items: Vec<&T> = self.iter().collect();
        items.sort();
        items.len().save(w);
        items.into_iter().for_each(|t| t.save(w));
    }

    fn load(&mut self, r: &mut StateReader) -> Result<(), SerdeError> {
        let mut len = 0_usize;
        len.load(r)?;
        self.clear();
        for _ in 0..len {
            let mut item = T::default();
            item.load(r)?;
            if !self.insert(item) {
                return SerdeError::new("HashSet", "duplicate element");
            }
        }
        Ok(())
    }
}
