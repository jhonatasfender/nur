//! Núcleo de domínio do Nur: modelos e value objects puros, sem IO.

mod byte_size;
mod caminho_dispositivo;
mod rotulo_volume;

pub use byte_size::ByteSize;
pub use caminho_dispositivo::CaminhoDispositivo;
pub use rotulo_volume::{RotuloInvalido, RotuloVolume};
