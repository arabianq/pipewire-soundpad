use crate::gui::SoundpadGui;
use egui::{
    Align, AtomExt, Button, CollapsingHeader, Color32, CursorIcon, Layout, RichText, ScrollArea,
    Sense, TextEdit, Ui, Vec2,
};
use egui_dnd::dnd;
use egui_material_icons::icons::*;
use pwsp_lib::types::{
    config::{GuiConfig, SortOrder},
    gui::{AppState, AudioPlayerState},
};
use rust_i18n::t;
use std::{cmp::Ordering, path::Path, path::PathBuf};

pub(crate) enum FileAction {
    Play(PathBuf, bool),
    StopAndPlay(u32, PathBuf, bool),
    AssignHotkey(PathBuf),
}

impl SoundpadGui {
    pub fn draw_body(&mut self, ui: &mut Ui) {
        let left_panel_width = self
            .config
            .left_panel_width
            .max(100.0)
            .min(ui.available_width() - 100.0);
        let dirs_size = Vec2::new(left_panel_width, ui.available_height() - 40.0);

        ui.horizontal(|ui| {
            self.draw_dirs(ui, dirs_size);

            let (rect, response) = ui.allocate_at_least(
                Vec2::new(ui.spacing().item_spacing.x, ui.available_height()),
                Sense::click_and_drag(),
            );

            if ui.is_rect_visible(rect) {
                let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
                ui.painter().vline(rect.center().x, rect.y_range(), stroke);
            }

            let vertical_separator_response =
                response.on_hover_and_drag_cursor(CursorIcon::ResizeHorizontal);

            if vertical_separator_response.dragged() {
                self.config.left_panel_width += vertical_separator_response.drag_delta().x;
                self.config.left_panel_width = self.config.left_panel_width.clamp(100.0, 500.0);
            }

            if vertical_separator_response.drag_stopped() {
                self.config.save_to_file().ok();
            }

            let files_size = Vec2::new(ui.available_width(), ui.available_height() - 40.0);
            self.draw_files(ui, files_size);
        });
    }

    fn draw_dirs(&mut self, ui: &mut Ui, area_size: Vec2) {
        ui.vertical(|ui| {
            ui.set_min_width(area_size.x);
            ui.set_min_height(area_size.y);

            ScrollArea::vertical().id_salt(0).show(ui, |ui| {
                ui.set_min_width(area_size.x);

                let mut dirs = std::mem::take(&mut self.app_state.dirs);
                let mut dir_to_open = None;

                dnd(ui, "dnd_directories").show_vec(&mut dirs, |ui, item, handle, _state| {
                    let path = item;
                    ui.horizontal(|ui| {
                        handle.ui(ui, |ui| {
                            ui.label(ICON_DRAG_INDICATOR.codepoint);
                        });
                        let name = path
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| path.to_string_lossy().to_string());

                        let mut dir_button =
                            Button::new(RichText::new(name.clone()).atom_max_width(area_size.x))
                                .frame(false);

                        if let Some(current_dir) = &self.app_state.current_dir
                            && current_dir.eq(&*path)
                        {
                            dir_button = dir_button.selected(true);
                        }

                        let dir_button_response = ui.add(dir_button);
                        if dir_button_response.clicked() {
                            dir_to_open = Some(path.clone());
                        }

                        let delete_dir_button = Button::new(ICON_DELETE).frame(false);
                        let delete_dir_button_response =
                            ui.add_sized([18.0, 18.0], delete_dir_button);
                        if delete_dir_button_response.clicked() {
                            self.app_state.dirs_to_remove.insert(path.clone());
                        }

                        // Context menu
                        dir_button_response.context_menu(|ui| {
                            if ui
                                .button(format!(
                                    "{} {}",
                                    ICON_OPEN_IN_NEW.codepoint,
                                    t!("gui.context.dirs.open")
                                ))
                                .clicked()
                            {
                                dir_to_open = Some(path.clone());
                            }

                            if ui
                                .button(format!(
                                    "{} {}",
                                    ICON_OPEN_IN_BROWSER.codepoint,
                                    t!("gui.context.dirs.open_in_fm")
                                ))
                                .clicked()
                                && let Err(e) = opener::open(&path)
                            {
                                eprintln!("Failed to open file manager: {}", e);
                            }

                            ui.separator();

                            if ui
                                .button(format!(
                                    "{} {}",
                                    ICON_DELETE.codepoint,
                                    t!("gui.context.dirs.remove")
                                ))
                                .clicked()
                            {
                                self.app_state.dirs_to_remove.insert(path.clone());
                            }

                            ui.separator();
                            ui.label(t!("gui.context.dirs.sort_by"));

                            let current_order = self
                                .config
                                .dirs_settings
                                .get(path)
                                .map(|s| s.sort_order)
                                .unwrap_or_default();
                            let mut new_order = None;

                            if ui
                                .radio(
                                    current_order == SortOrder::AlphabeticalAsc,
                                    t!("gui.sort.alpha_asc"),
                                )
                                .clicked()
                            {
                                new_order = Some(SortOrder::AlphabeticalAsc);
                            }
                            if ui
                                .radio(
                                    current_order == SortOrder::AlphabeticalDesc,
                                    t!("gui.sort.alpha_desc"),
                                )
                                .clicked()
                            {
                                new_order = Some(SortOrder::AlphabeticalDesc);
                            }
                            if ui
                                .radio(
                                    current_order == SortOrder::DateModifiedNewest,
                                    t!("gui.sort.date_newest"),
                                )
                                .clicked()
                            {
                                new_order = Some(SortOrder::DateModifiedNewest);
                            }
                            if ui
                                .radio(
                                    current_order == SortOrder::DateModifiedOldest,
                                    t!("gui.sort.date_oldest"),
                                )
                                .clicked()
                            {
                                new_order = Some(SortOrder::DateModifiedOldest);
                            }

                            if let Some(order) = new_order {
                                self.config
                                    .dirs_settings
                                    .entry(path.clone())
                                    .or_default()
                                    .sort_order = order;
                                self.config.save_to_file().ok();
                                self.app_state.dir_cache.remove(path);
                                self.open_dir(path);
                            }
                        });
                    });
                });
                self.app_state.dirs = dirs;

                if let Some(path) = dir_to_open {
                    self.open_dir(&path);
                }

                ui.horizontal(|ui| {
                    let add_dirs_button = Button::new(ICON_ADD).frame(false);
                    let add_dirs_button_response = ui.add_sized([18.0, 18.0], add_dirs_button);
                    if add_dirs_button_response.clicked() {
                        self.add_dirs();
                    }
                });

                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    let play_file_button = Button::new(t!("gui.play_file_button"));
                    let play_file_button_response = ui.add(play_file_button);
                    if play_file_button_response.clicked() {
                        self.open_file();
                    }
                });
            });
        });
    }

    fn draw_files_search_field(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            let search_field_response = ui.add_sized(
                [ui.available_width(), 22.0],
                TextEdit::singleline(&mut self.app_state.search_query)
                    .hint_text(t!("gui.search_placeholder")),
            );

            if self.app_state.force_focus_search {
                search_field_response.request_focus();
                self.app_state.force_focus_search = false;
            }

            self.app_state.search_field_id = Some(search_field_response.id);
        });
    }

    fn draw_files_list(&mut self, ui: &mut Ui, area_size: Vec2) {
        ScrollArea::vertical().id_salt(1).show(ui, |ui| {
            ui.set_min_width(area_size.x);
            ui.set_min_height(area_size.y);

            ui.vertical(|ui| {
                let mut actions = Vec::new();
                let files = self.get_filtered_files();
                for entry_path in files {
                    Self::draw_tree_node(
                        ui,
                        entry_path,
                        &self.config,
                        &mut self.app_state,
                        &self.audio_player_state,
                        &mut actions,
                    );
                }

                for action in actions {
                    match action {
                        FileAction::Play(path, concurrent) => self.play_file(&path, concurrent),
                        FileAction::StopAndPlay(id, path, concurrent) => {
                            self.stop(Some(id));
                            self.play_file(&path, concurrent);
                        }
                        FileAction::AssignHotkey(path) => {
                            self.app_state.assigning_hotkey_for_file = Some(path);
                            self.app_state.hotkey_capture_active = true;
                        }
                    }
                }
            });
        });
    }

    fn draw_files(&mut self, ui: &mut Ui, area_size: Vec2) {
        ui.vertical(|ui| {
            self.draw_files_search_field(ui);
            ui.separator();
            self.draw_files_list(ui, area_size);
        });
    }

    fn draw_tree_node_dir(
        ui: &mut Ui,
        path: std::path::PathBuf,
        config: &GuiConfig,
        app_state: &mut AppState,
        audio_player_state: &AudioPlayerState,
        actions: &mut Vec<FileAction>,
    ) {
        let dir_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        CollapsingHeader::new(dir_name)
            .id_salt(&path)
            .show(ui, |ui| {
                let children = if let Some(cached) = app_state.dir_cache.get(&path) {
                    cached.clone()
                } else {
                    let mut read = Vec::new();
                    if let Ok(entries) = std::fs::read_dir(&path) {
                        for entry in entries.filter_map(|e| e.ok()) {
                            let child_path = entry.path();
                            if !child_path.is_dir()
                                && !crate::gui::SUPPORTED_EXTENSIONS.contains(
                                    &child_path
                                        .extension()
                                        .unwrap_or_default()
                                        .to_str()
                                        .unwrap_or_default(),
                                ) {
                                    continue;
                                }
                            read.push(child_path);
                        }
                    }
                    let sort_order = config.get_sort_order(&path);
                    read.sort_by(|a, b| {
                        let a_is_dir = a.is_dir();
                        let b_is_dir = b.is_dir();
                        if a_is_dir && !b_is_dir {
                            Ordering::Less
                        } else if !a_is_dir && b_is_dir {
                            Ordering::Greater
                        } else {
                            sort_order.compare(a, b)
                        }
                    });
                    app_state.dir_cache.insert(path.clone(), read.clone());
                    read
                };

                let search_query = app_state.search_query.to_lowercase();
                let search_query = search_query.trim();

                for child in children {
                    if !child.is_dir()
                        && !search_query.is_empty() {
                            let file_name = child
                                .file_name()
                                .unwrap_or_default()
                                .to_string_lossy()
                                .to_string();
                            if !file_name.to_lowercase().contains(search_query) {
                                continue;
                            }
                        }
                    Self::draw_tree_node(ui, child, config, app_state, audio_player_state, actions);
                }
            });
    }

    fn draw_tree_node_file(
        ui: &mut Ui,
        path: std::path::PathBuf,
        app_state: &mut AppState,
        audio_player_state: &AudioPlayerState,
        actions: &mut Vec<FileAction>,
    ) {
        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        ui.horizontal(|ui| {
            // Hotkey badge
            let mut hotkey_badge = None;
            for slot in &app_state.hotkey_config.slots {
                if slot.action.name == "play"
                    && let Some(file_path_str) = slot.action.args.get("file_path")
                    && Path::new(file_path_str) == path
                {
                    if let Some(chord) = &slot.key_chord {
                        hotkey_badge = Some(format!("[{}]", chord));
                    } else {
                        hotkey_badge = Some(format!("[{}]", slot.slot));
                    }
                    break;
                }
            }

            if let Some(badge) = &hotkey_badge {
                ui.label(
                    RichText::new(badge)
                        .small()
                        .monospace()
                        .color(Color32::from_rgb(100, 200, 100)),
                );
            }

            let file_button_text = RichText::new(&file_name);

            let file_button = Button::new(file_button_text).frame(false).truncate();
            let file_button_response = ui.add(file_button);
            if file_button_response.clicked() {
                ui.input(|i| {
                    if i.modifiers.ctrl {
                        actions.push(FileAction::Play(path.clone(), true));
                    } else if i.modifiers.shift
                        && let Some(last_track) = audio_player_state.tracks.last()
                    {
                        actions.push(FileAction::StopAndPlay(last_track.id, path.clone(), true));
                    } else {
                        actions.push(FileAction::Play(path.clone(), false));
                    }
                });
            }

            // Context menu
            file_button_response.context_menu(|ui| {
                if ui
                    .button(format!(
                        "{} {}",
                        ICON_BOLT.codepoint,
                        t!("gui.context.files.play_solo")
                    ))
                    .clicked()
                {
                    actions.push(FileAction::Play(path.clone(), false));
                }

                if ui
                    .button(format!(
                        "{} {}",
                        ICON_ADD.codepoint,
                        t!("gui.context.files.add_new")
                    ))
                    .clicked()
                {
                    actions.push(FileAction::Play(path.clone(), true));
                }

                if ui
                    .button(format!(
                        "{} {}",
                        ICON_SWAP_HORIZ.codepoint,
                        t!("gui.context.files.replace_last")
                    ))
                    .clicked()
                    && let Some(last_track) = audio_player_state.tracks.last()
                {
                    actions.push(FileAction::StopAndPlay(last_track.id, path.clone(), true));
                }

                ui.separator();

                if ui
                    .button(format!(
                        "{} {}",
                        ICON_OPEN_IN_BROWSER.codepoint,
                        t!("gui.context.files.show_in_fm")
                    ))
                    .clicked()
                    && let Err(e) = opener::reveal(&path)
                {
                    eprintln!("Failed to open file manager: {}", e);
                }

                ui.separator();

                if ui
                    .button(format!(
                        "{} {}",
                        ICON_KEYBOARD.codepoint,
                        t!("gui.context.files.asign_hotkey")
                    ))
                    .clicked()
                {
                    actions.push(FileAction::AssignHotkey(path.clone()));
                    ui.close();
                }

                ui.separator();

                if ui
                    .button(format!(
                        "{} {}",
                        ICON_FILE_COPY.codepoint,
                        t!("gui.context.files.copy_cli_command")
                    ))
                    .clicked()
                {
                    ui.ctx().copy_text(format!(
                        "pwsp-cli action play \"{}\"",
                        path.to_string_lossy()
                            .replace('\\', "\\\\")
                            .replace('"', "\\\"")
                    ));
                }
            });
        });
    }

    fn draw_tree_node(
        ui: &mut Ui,
        path: std::path::PathBuf,
        config: &GuiConfig,
        app_state: &mut AppState,
        audio_player_state: &AudioPlayerState,
        actions: &mut Vec<FileAction>,
    ) {
        if path.is_dir() {
            Self::draw_tree_node_dir(ui, path, config, app_state, audio_player_state, actions);
        } else {
            Self::draw_tree_node_file(ui, path, app_state, audio_player_state, actions);
        }
    }
}
