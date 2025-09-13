mod app;

use std::fs;
fn main() -> Result<(), eframe::Error> {
    let settings = generate_settings();
    app::run(settings)
}

fn generate_settings() -> app::Settings {
    let config_dir_path = dirs::config_dir().unwrap_or_default().join("pwsp");
    let config_path = config_dir_path.join("pwsp.json");
    fs::create_dir_all(&config_dir_path).ok();

    let settings: app::Settings;
    if config_path.exists() {
        settings = app::settings::load_from_file(&config_path);
    } else {
        settings = app::Settings::default();
        settings.save_to_file(&config_path);
    }

    settings
}
