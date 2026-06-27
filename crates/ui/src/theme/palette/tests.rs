use super::*;

#[test]
fn temas_tem_fundos_diferentes() {
    assert_ne!(Palette::clara().fundo, Palette::escura().fundo);
}

#[test]
fn sucesso_e_verde_nos_dois_temas() {
    // Verde de sucesso é o mesmo token (#16A34A) nos dois temas.
    assert_eq!(Palette::clara().sucesso, Palette::escura().sucesso);
}
