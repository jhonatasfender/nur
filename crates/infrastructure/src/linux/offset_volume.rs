//! Janela de IO: confina um Read+Write+Seek a `[start, start+len)`.
//! Usada para entregar ao `fatfs` apenas a região da partição do device.

use std::io::{Read, Result, Seek, SeekFrom, Write};

/// Recorte `[start, start+len)` sobre um dispositivo/arquivo.
pub(crate) struct OffsetVolume<T> {
    inner: T,
    start: u64,
    len: u64,
    pos: u64,
}

impl<T: Read + Write + Seek> OffsetVolume<T> {
    /// Cria a janela.
    pub(crate) fn new(inner: T, start: u64, len: u64) -> Self {
        Self {
            inner,
            start,
            len,
            pos: 0,
        }
    }

    /// Devolve o IO interno.
    pub(crate) fn into_inner(self) -> T {
        self.inner
    }

    // Quantos bytes ainda cabem da posição atual até o fim da janela.
    fn remaining(&self) -> u64 {
        self.len.saturating_sub(self.pos)
    }
}

impl<T: Read + Write + Seek> Read for OffsetVolume<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let cap = self.remaining().min(buf.len() as u64) as usize;
        if cap == 0 {
            return Ok(0);
        }
        self.inner.seek(SeekFrom::Start(self.start + self.pos))?;
        let n = self.inner.read(&mut buf[..cap])?;
        self.pos += n as u64;
        Ok(n)
    }
}

impl<T: Read + Write + Seek> Write for OffsetVolume<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let cap = self.remaining().min(buf.len() as u64) as usize;
        if cap == 0 {
            return Ok(0);
        }
        self.inner.seek(SeekFrom::Start(self.start + self.pos))?;
        let n = self.inner.write(&buf[..cap])?;
        self.pos += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

impl<T: Read + Write + Seek> Seek for OffsetVolume<T> {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        let target = match pos {
            SeekFrom::Start(n) => n as i64,
            SeekFrom::End(n) => self.len as i64 + n,
            SeekFrom::Current(n) => self.pos as i64 + n,
        };
        let clamped = target.clamp(0, self.len as i64);
        self.pos = clamped as u64;
        Ok(self.pos)
    }
}

#[cfg(test)]
mod tests;
