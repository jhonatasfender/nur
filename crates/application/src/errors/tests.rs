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

#[test]
fn browse_error_messages_are_in_ptbr() {
    use super::BrowseError;
    assert_eq!(
        BrowseError::NoFilesystem.to_string(),
        "este pendrive não tem uma partição legível para abrir"
    );
    assert_eq!(
        BrowseError::Mount("x".to_owned()).to_string(),
        "não foi possível montar o pendrive: x"
    );
    assert_eq!(
        BrowseError::Launch("y".to_owned()).to_string(),
        "não foi possível abrir o gerenciador: y"
    );
}
