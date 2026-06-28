//! Cópia raw com progresso/cancelamento e verificação por releitura.
//! Genérico sobre IO para ser testável sem dispositivo real.

use application::errors::WriteError;
use application::ports::{CancelFlag, ProgressSink, WritePhase, WriteProgress};
use std::io::{Read, Write};

const CHUNK: usize = 4 * 1024 * 1024;

/// Rotinas de cópia/verificação sobre `Read`/`Write` genéricos.
pub struct RawCopier;

impl RawCopier {
    /// Copia `total` bytes de `source` para `dest` em blocos de 4 MiB.
    ///
    /// # Errors
    /// Retorna [`WriteError::Cancelled`] se cancelado, ou [`WriteError::Io`] em falha de IO.
    pub fn copy<R: Read, W: Write>(
        source: &mut R,
        dest: &mut W,
        total: u64,
        sink: &dyn ProgressSink,
        cancel: &CancelFlag,
    ) -> Result<(), WriteError> {
        let mut buf = vec![0u8; CHUNK];
        let mut done: u64 = 0;
        loop {
            if cancel.is_cancelled() {
                return Err(WriteError::Cancelled);
            }
            let n = source
                .read(&mut buf)
                .map_err(|e| WriteError::Io(e.to_string()))?;
            if n == 0 {
                break;
            }
            dest.write_all(&buf[..n])
                .map_err(|e| WriteError::Io(e.to_string()))?;
            done += n as u64;
            sink.report(WriteProgress::new(WritePhase::Writing, done, total));
        }
        dest.flush().map_err(|e| WriteError::Io(e.to_string()))?;
        Ok(())
    }

    /// Relê `total` bytes de `written` e compara com `original`.
    ///
    /// # Errors
    /// Retorna [`WriteError::VerificationMismatch`] em divergência, [`WriteError::Cancelled`]
    /// se cancelado, ou [`WriteError::Io`] em falha de IO.
    pub fn verify<A: Read, B: Read>(
        written: &mut A,
        original: &mut B,
        total: u64,
        sink: &dyn ProgressSink,
        cancel: &CancelFlag,
    ) -> Result<(), WriteError> {
        let mut a = vec![0u8; CHUNK];
        let mut b = vec![0u8; CHUNK];
        let mut done: u64 = 0;
        while done < total {
            if cancel.is_cancelled() {
                return Err(WriteError::Cancelled);
            }
            let want = CHUNK.min((total - done) as usize);
            written
                .read_exact(&mut a[..want])
                .map_err(|e| WriteError::Io(e.to_string()))?;
            original
                .read_exact(&mut b[..want])
                .map_err(|e| WriteError::Io(e.to_string()))?;
            if a[..want] != b[..want] {
                return Err(WriteError::VerificationMismatch);
            }
            done += want as u64;
            sink.report(WriteProgress::new(WritePhase::Verifying, done, total));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
