use super::*;

#[test]
fn salva_png_de_imagem_simples() {
    let imagem = egui::ColorImage::filled([4, 4], egui::Color32::from_rgb(10, 20, 30));
    let destino = std::env::temp_dir().join("nur_captura_teste.png");
    let _ = std::fs::remove_file(&destino);
    Capturador::salvar_png(&imagem, &destino).unwrap();
    let meta = std::fs::metadata(&destino).unwrap();
    assert!(meta.len() > 0);
    std::fs::remove_file(&destino).unwrap();
}

#[test]
fn destino_manual_incrementa_e_numera() {
    let mut cap = Capturador {
        auto: None,
        auto_solicitado: false,
        frames: 0,
        contador: 0,
        ultima_msg: None,
    };
    let p1 = cap.proximo_destino();
    let p2 = cap.proximo_destino();
    assert_ne!(p1, p2);
    assert!(p1.to_string_lossy().contains("001"));
    assert!(p2.to_string_lossy().contains("002"));
}
