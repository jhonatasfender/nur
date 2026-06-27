use super::*;

#[test]
fn saves_png_of_simple_image() {
    let image = egui::ColorImage::filled([4, 4], egui::Color32::from_rgb(10, 20, 30));
    let dest = std::env::temp_dir().join("nur_capture_test.png");
    let _ = std::fs::remove_file(&dest);
    Capturer::save_png(&image, &dest).unwrap();
    let meta = std::fs::metadata(&dest).unwrap();
    assert!(meta.len() > 0);
    std::fs::remove_file(&dest).unwrap();
}

#[test]
fn manual_destination_increments_and_numbers() {
    let mut cap = Capturer {
        auto: None,
        auto_requested: false,
        frames: 0,
        counter: 0,
        last_msg: None,
    };
    let p1 = cap.next_destination();
    let p2 = cap.next_destination();
    assert_ne!(p1, p2);
    assert!(p1.to_string_lossy().contains("001"));
    assert!(p2.to_string_lossy().contains("002"));
}
