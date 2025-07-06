use std::fs;

mod app;

fn main() -> Result<(), eframe::Error> {
    create_dirs();
    app::run()
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
