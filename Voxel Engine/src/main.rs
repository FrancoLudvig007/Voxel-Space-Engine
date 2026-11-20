#[macro_use]
pub mod pyrsutils;
pub use pyrsutils::{net_utils, sec_utils, time_utils};
pub mod app;
pub mod brush;
pub mod file_browser;
pub mod math;
pub mod mesh_renderer;
pub mod sky;
pub mod terrain;
pub mod voxel_renderer;

use app::VoxelSpaceApp;
use eframe::egui;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 880.0])
            .with_title("Voxel Space Engine (Comanche-style Renderer + OBJ + Sculpt)"),
        ..Default::default()
    };
    eframe::run_native(
        "Voxel Space Engine",
        options,
        Box::new(|cc| Ok(Box::new(VoxelSpaceApp::new(cc)))),
    )
}
