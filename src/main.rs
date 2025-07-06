use eframe::NativeOptions;
use egui::Vec2;
use std::fs;
use std::fs::create_dir;

mod app;

fn main() -> Result<(), eframe::Error> {
    create_dirs();

    let mut options = NativeOptions {
        ..Default::default()
    };
    options.viewport.min_inner_size = Some(Vec2::new(400.0, 400.0));
    options.vsync = true;
    options.hardware_acceleration = eframe::HardwareAcceleration::Preferred;

    eframe::run_native(
        "PipeWire SoundPad",
        options,
        Box::new(|cc| {
            egui_material_icons::initialize(&cc.egui_ctx);
            Ok(Box::new(app::App::new(cc)))
        }),
    )
}

fn create_dirs() {
    let config_dir_path = dirs::config_dir().unwrap().join("pwsp");
    fs::create_dir_all(&config_dir_path).ok();

    if !fs::exists(config_dir_path.join("saved_dirs"))
        .ok()
        .unwrap_or(false)
    {
        fs::File::create(config_dir_path.join("saved_dirs")).ok();
    }
    if !fs::exists(config_dir_path.join("saved_mic"))
        .ok()
        .unwrap_or(false)
    {
        fs::File::create(config_dir_path.join("saved_mic")).ok();
    }
    if !fs::exists(config_dir_path.join("saved_volume"))
        .ok()
        .unwrap_or(false)
    {
        fs::File::create(config_dir_path.join("saved_volume")).ok();
    }
}
