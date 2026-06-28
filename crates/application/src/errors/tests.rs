use super::{IsoError, WriteError};

#[test]
fn write_error_messages_are_in_ptbr() {
    assert_eq!(WriteError::Unauthorized.to_string(), "autorização negada");
    assert_eq!(WriteError::Cancelled.to_string(), "operação cancelada");
    assert_eq!(
        WriteError::DeviceTooSmall.to_string(),
        "o dispositivo é menor que a imagem"
    );
}

#[test]
fn iso_error_wraps_message() {
    assert_eq!(
        IsoError::Io("x".to_owned()).to_string(),
        "falha ao ler a ISO: x"
    );
}
