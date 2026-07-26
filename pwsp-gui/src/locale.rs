use pwsp_lib::types::config::GuiConfig;

/// Applies the UI language: the one pinned in the config, or the system's when none is.
///
/// Safe to call at any time — `t!` reads the locale on every lookup, so the next frame
/// already renders in the new language.
pub fn apply(config: &GuiConfig) {
    let locale = config.forced_lang.clone().unwrap_or_else(system_locale);
    rust_i18n::set_locale(&locale);
}

fn system_locale() -> String {
    sys_locale::get_locale().unwrap_or_else(|| String::from("en-US"))
}

/// Locales shipped under `locales/`, in a stable order for the menu.
pub fn available() -> Vec<String> {
    let mut locales: Vec<String> = rust_i18n::available_locales!()
        .into_iter()
        .map(|locale| locale.to_string())
        .collect();
    locales.sort_unstable();
    locales
}

/// How a language names itself, which is how it should appear in a language picker
/// regardless of the language the rest of the UI is in. An unnamed locale falls back to
/// its code, so a newly shipped translation is still selectable.
pub fn display_name(code: &str) -> &str {
    match code {
        "en" => "English",
        "ru" => "Русский",
        "es" => "Español",
        "fr" => "Français",
        "zh" => "中文",
        "ar" => "العربية",
        "kz" => "Қазақша",
        "he" => "עברית",
        "pt-BR" => "Português (Brasil)",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shipped_locale_has_a_native_name() {
        for code in available() {
            assert_ne!(
                display_name(&code),
                code,
                "locale {code} falls back to its code in the picker"
            );
        }
    }
}
