#[macro_use]
pub mod pyrsutils;
pub mod editor_ui;
pub mod entities;
pub mod math3d;
pub mod renderer;
pub mod textures;

use editor_ui::DescentEditorApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 860.0])
            .with_title("Descent 2 Map Editor & 6DOF Engine (Modular OOP Rust)"),
        ..Default::default()
    };
    eframe::run_native(
        "Descent 2 Map Editor",
        options,
        Box::new(|cc| Ok(Box::new(DescentEditorApp::new(cc)))),
    )
}
