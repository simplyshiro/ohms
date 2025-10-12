use eframe::egui;

use ohms::OhmsApp;

fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Ohms")
            .with_app_id("com.github.simplyshiro.ohms")
            .with_inner_size([480.0, 240.0])
            .with_resizable(false),
        ..Default::default()
    };

    eframe::run_native(
        "Ohms",
        native_options,
        Box::new(|cc| Ok(Box::new(OhmsApp::new(cc)))),
    )
}
